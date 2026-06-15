//! Diagnostics Service Module
//!
//! Servicio de diagnóstico de red centralizado.
//! Usado por comandos y system tray.
//!
//! By LOUST (www.loust.pro)

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Instant;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Estado de salud de la red
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkHealth {
    Excellent, // < 20ms
    Good,      // < 50ms
    Fair,      // < 100ms
    Poor,      // < 200ms
    Bad,       // >= 200ms
    Down,      // Sin conexión
}

impl std::fmt::Display for NetworkHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkHealth::Excellent => write!(f, "Excelente"),
            NetworkHealth::Good => write!(f, "Buena"),
            NetworkHealth::Fair => write!(f, "Regular"),
            NetworkHealth::Poor => write!(f, "Lenta"),
            NetworkHealth::Bad => write!(f, "Mala"),
            NetworkHealth::Down => write!(f, "Sin conexión"),
        }
    }
}

/// Punto de fallo en la red
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailurePoint {
    None,
    Adapter,
    Router,
    Isp,
    Dns,
}

/// Resultado del diagnóstico completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub health: NetworkHealth,
    pub failure_point: FailurePoint,
    pub adapter_ok: bool,
    pub adapter_name: String,
    pub router_ok: bool,
    pub router_latency_ms: f64,
    pub isp_ok: bool,
    pub isp_latency_ms: f64,
    pub dns_ok: bool,
    pub dns_latency_ms: f64,
    pub recommendation: String,
}

/// Resultado de quick check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickCheckResult {
    pub connected: bool,
    pub latency_ms: f64,
    pub message: String,
}

/// Ejecutar diagnóstico rápido
pub fn quick_check() -> QuickCheckResult {
    // Ping a Cloudflare (1.1.1.1) que es muy rápido
    match ping_host("1.1.1.1") {
        Some(latency) if latency > 0.0 => {
            let message = if latency < 50.0 {
                format!("Conexión OK ({:.0}ms)", latency)
            } else if latency < 100.0 {
                format!("Conexión aceptable ({:.0}ms)", latency)
            } else {
                format!("Conexión lenta ({:.0}ms)", latency)
            };

            QuickCheckResult {
                connected: true,
                latency_ms: latency,
                message,
            }
        }
        _ => QuickCheckResult {
            connected: false,
            latency_ms: 0.0,
            message: "Sin conexión a internet".to_string(),
        },
    }
}

/// Ejecutar diagnóstico completo de 4 fases
pub fn run_full_diagnostic() -> DiagnosticResult {
    let mut result = DiagnosticResult {
        health: NetworkHealth::Down,
        failure_point: FailurePoint::Adapter,
        adapter_ok: false,
        adapter_name: String::new(),
        router_ok: false,
        router_latency_ms: 0.0,
        isp_ok: false,
        isp_latency_ms: 0.0,
        dns_ok: false,
        dns_latency_ms: 0.0,
        recommendation: String::new(),
    };

    // Fase 1: Verificar adaptador
    log::info!("Diagnóstico Fase 1: Verificando adaptador...");
    match check_adapter() {
        Some(adapter) => {
            result.adapter_ok = true;
            result.adapter_name = adapter;
        }
        None => {
            result.recommendation =
                "No hay adaptador de red activo. Verifica tu conexión física.".to_string();
            return result;
        }
    }

    // Fase 2: Verificar router (gateway)
    log::info!("Diagnóstico Fase 2: Verificando router...");
    match get_gateway() {
        Some(gateway) => {
            if let Some(latency) = ping_host(&gateway) {
                if latency > 0.0 {
                    result.router_ok = true;
                    result.router_latency_ms = latency;
                    result.failure_point = FailurePoint::Isp;
                } else {
                    result.recommendation =
                        "No se puede alcanzar el router. Verifica tu conexión local.".to_string();
                    return result;
                }
            } else {
                result.recommendation = "El router no responde. Reinicia tu router.".to_string();
                return result;
            }
        }
        None => {
            result.recommendation =
                "No se detectó gateway. Verifica la configuración de red.".to_string();
            return result;
        }
    }

    // Fase 3: Verificar ISP (conectividad externa)
    log::info!("Diagnóstico Fase 3: Verificando ISP...");
    if let Some(latency) = ping_host("1.1.1.1") {
        if latency > 0.0 {
            result.isp_ok = true;
            result.isp_latency_ms = latency;
            result.failure_point = FailurePoint::Dns;
        } else {
            result.recommendation = "Sin conexión a internet. Contacta a tu ISP.".to_string();
            return result;
        }
    } else {
        result.recommendation =
            "Sin respuesta del ISP. Verifica si hay problemas con tu proveedor.".to_string();
        return result;
    }

    // Fase 4: Verificar DNS
    log::info!("Diagnóstico Fase 4: Verificando DNS...");
    if let Some(latency) = check_dns_resolution() {
        if latency > 0.0 {
            result.dns_ok = true;
            result.dns_latency_ms = latency;
            result.failure_point = FailurePoint::None;
        } else {
            result.recommendation =
                "DNS no funciona correctamente. Considera cambiar a Cloudflare (1.1.1.1)."
                    .to_string();
            return result;
        }
    } else {
        result.recommendation =
            "Error de resolución DNS. Prueba limpiar caché DNS o cambiar servidor.".to_string();
        return result;
    }

    // Calcular salud general
    let max_latency = result
        .router_latency_ms
        .max(result.isp_latency_ms)
        .max(result.dns_latency_ms);

    result.health = if max_latency < 20.0 {
        NetworkHealth::Excellent
    } else if max_latency < 50.0 {
        NetworkHealth::Good
    } else if max_latency < 100.0 {
        NetworkHealth::Fair
    } else if max_latency < 200.0 {
        NetworkHealth::Poor
    } else {
        NetworkHealth::Bad
    };

    result.recommendation = match result.health {
        NetworkHealth::Excellent => "Tu conexión está funcionando perfectamente. 🚀".to_string(),
        NetworkHealth::Good => "Tu conexión está bien. No se requieren cambios.".to_string(),
        NetworkHealth::Fair => "Conexión aceptable. Considera optimizar tu DNS.".to_string(),
        NetworkHealth::Poor => {
            "Conexión lenta. Prueba cambiar DNS y aplicar optimizaciones.".to_string()
        }
        NetworkHealth::Bad => {
            "Conexión muy lenta. Verifica tu ISP o aplica perfil agresivo.".to_string()
        }
        NetworkHealth::Down => "Sin conexión.".to_string(),
    };

    log::info!("Diagnóstico completo: {:?}", result.health);
    result
}

/// Verificar adaptador de red activo
fn check_adapter() -> Option<String> {
    let ps_script = r#"
        Get-NetAdapter | Where-Object Status -eq 'Up' | 
        Select-Object -First 1 -ExpandProperty Name
    "#;

    run_powershell(ps_script).ok().filter(|s| !s.is_empty())
}

/// Obtener gateway por defecto
fn get_gateway() -> Option<String> {
    let ps_script = r#"
        (Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Select-Object -First 1).NextHop
    "#;

    run_powershell(ps_script).ok().filter(|s| !s.is_empty())
}

/// Ping a un host y retornar latencia en ms
fn ping_host(host: &str) -> Option<f64> {
    let _start = Instant::now();

    let ps_script = format!(
        r#"
        $ping = Test-Connection -ComputerName {} -Count 1 -ErrorAction SilentlyContinue
        if ($ping) {{ $ping.ResponseTime }} else {{ -1 }}
        "#,
        host
    );

    run_powershell(&ps_script)
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|&lat| lat >= 0.0)
}

/// Verificar resolución DNS
fn check_dns_resolution() -> Option<f64> {
    let start = Instant::now();

    let ps_script = r#"
        try {
            $result = Resolve-DnsName -Name "google.com" -DnsOnly -ErrorAction Stop
            if ($result) { "OK" } else { "FAIL" }
        } catch {
            "FAIL"
        }
    "#;

    match run_powershell(ps_script) {
        Ok(result) if result.trim() == "OK" => Some(start.elapsed().as_millis() as f64),
        _ => None,
    }
}

/// Ejecutar comando PowerShell
fn run_powershell(command: &str) -> Result<String, String> {
    #[cfg(windows)]
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", command])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Error: {}", e))?;

    #[cfg(not(windows))]
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", command])
        .output()
        .map_err(|e| format!("Error: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
