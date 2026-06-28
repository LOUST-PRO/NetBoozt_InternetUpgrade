//! Diagnostic commands module
//!
//! Comandos Tauri para diagnóstico de red.

use serde::{Deserialize, Serialize};
#[cfg(not(windows))]
use std::fs;
#[cfg(not(windows))]
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
#[cfg(not(windows))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(not(windows))]
fn linux_find_program(program: &str) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            let candidate = directory.join(program);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    for prefix in [
        "/usr/local/sbin",
        "/usr/local/bin",
        "/usr/sbin",
        "/usr/bin",
        "/sbin",
        "/bin",
    ] {
        let candidate = Path::new(prefix).join(program);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

#[cfg(not(windows))]
fn linux_command(program: &str) -> Command {
    if let Some(path) = linux_find_program(program) {
        Command::new(path)
    } else {
        Command::new(program)
    }
}

#[cfg(not(windows))]
fn linux_capture(program: &str, args: &[&str]) -> Option<String> {
    let output = linux_command(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(not(windows))]
fn linux_spawn(program: &str, args: &[&str]) -> Result<(), String> {
    linux_command(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Error iniciando {}: {}", program, e))
}

#[cfg(not(windows))]
fn linux_primary_adapter() -> Option<String> {
    if let Some(route) = linux_capture("ip", &["route", "show", "default"]) {
        let mut parts = route.split_whitespace();
        while let Some(token) = parts.next() {
            if token == "dev" {
                if let Some(device) = parts.next() {
                    return Some(device.to_string());
                }
            }
        }
    }

    let devices = linux_capture(
        "nmcli",
        &["-t", "-f", "DEVICE,TYPE,STATE", "device", "status"],
    )?;

    for line in devices.lines() {
        let mut fields = line.split(':');
        let device = fields.next().unwrap_or("").trim();
        let device_type = fields.next().unwrap_or("").trim();
        let state = fields.next().unwrap_or("").trim().to_lowercase();

        if device.is_empty() || !matches!(device_type, "ethernet" | "wifi") {
            continue;
        }

        if matches!(state.as_str(), "connected" | "connecting") {
            return Some(device.to_string());
        }
    }

    None
}

#[cfg(not(windows))]
fn linux_active_connection(adapter: &str) -> Option<String> {
    let connection = linux_capture(
        "nmcli",
        &["-t", "-g", "GENERAL.CONNECTION", "device", "show", adapter],
    )?;

    let connection = connection.lines().next().unwrap_or("").trim();
    if connection.is_empty() || connection == "--" {
        None
    } else {
        Some(connection.to_string())
    }
}

#[cfg(not(windows))]
fn linux_open_network_settings() -> Result<String, String> {
    if linux_find_program("nm-connection-editor").is_some() {
        linux_spawn("nm-connection-editor", &[])?;
        return Ok("Editor de conexiones de red iniciado".to_string());
    }

    if linux_find_program("gnome-control-center").is_some() {
        linux_spawn("gnome-control-center", &["network"])?;
        return Ok("Ajustes de red iniciados".to_string());
    }

    if linux_find_program("plasma-open-settings").is_some() {
        linux_spawn("plasma-open-settings", &["kcm_networkmanagement"])?;
        return Ok("Ajustes de red iniciados".to_string());
    }

    if linux_find_program("systemsettings").is_some() {
        linux_spawn("systemsettings", &["kcm_networkmanagement"])?;
        return Ok("Ajustes de red iniciados".to_string());
    }

    Err("No se encontró una herramienta gráfica de red compatible".to_string())
}

#[cfg(not(windows))]
fn linux_open_system_settings() -> Result<String, String> {
    if linux_find_program("gnome-control-center").is_some() {
        linux_spawn("gnome-control-center", &[])?;
        return Ok("Ajustes del sistema iniciados".to_string());
    }

    if linux_find_program("plasma-open-settings").is_some() {
        linux_spawn("plasma-open-settings", &[])?;
        return Ok("Ajustes del sistema iniciados".to_string());
    }

    if linux_find_program("systemsettings").is_some() {
        linux_spawn("systemsettings", &[])?;
        return Ok("Ajustes del sistema iniciados".to_string());
    }

    Err("No se encontró una herramienta de ajustes del sistema compatible".to_string())
}

#[cfg(not(windows))]
fn linux_open_terminal() -> Result<String, String> {
    for terminal in [
        "ptyxis",
        "kgx",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "kitty",
        "alacritty",
        "foot",
        "xterm",
    ] {
        if linux_find_program(terminal).is_some() {
            linux_spawn(terminal, &[])?;
            return Ok("Terminal del sistema iniciado".to_string());
        }
    }

    Err("No se encontró un emulador de terminal compatible".to_string())
}

#[cfg(not(windows))]
fn linux_write_network_snapshot() -> Result<PathBuf, String> {
    let mut content = String::from("NetBoozt Linux Network Snapshot\n\n");

    if let Some(adapter) = linux_primary_adapter() {
        content.push_str(&format!("Adaptador principal: {}\n\n", adapter));
    }

    let sections = [
        ("ip addr", linux_capture("ip", &["addr", "show"])),
        ("ip route", linux_capture("ip", &["route", "show"])),
        (
            "nmcli device status",
            linux_capture(
                "nmcli",
                &[
                    "-t",
                    "-f",
                    "DEVICE,TYPE,STATE,CONNECTION",
                    "device",
                    "status",
                ],
            ),
        ),
        ("resolvectl dns", linux_capture("resolvectl", &["dns"])),
    ];

    for (title, body) in sections {
        content.push_str(&format!("## {}\n", title));
        content.push_str(body.as_deref().unwrap_or("No disponible"));
        content.push_str("\n\n");
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = std::env::temp_dir().join(format!("netboozt-network-snapshot-{}.txt", timestamp));

    fs::write(&path, content)
        .map_err(|e| format!("No se pudo escribir el resumen de red: {}", e))?;

    Ok(path)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FailurePoint {
    None,
    Adapter,
    Router,
    Isp,
    Dns,
    Internet,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkHealth {
    Excellent,
    Good,
    Fair,
    Poor,
    Bad,
    Down,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub health: NetworkHealth,
    pub score: f64,
    pub failure_point: FailurePoint,
    pub adapter_ok: bool,
    pub adapter_name: String,
    pub router_ok: bool,
    pub router_latency_ms: f64,
    pub isp_ok: bool,
    pub isp_latency_ms: f64,
    pub dns_ok: bool,
    pub dns_latency_ms: f64,
    pub internet_ok: bool,
    pub internet_latency_ms: f64,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResult {
    pub success: bool,
    pub host: String,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsHealthResult {
    pub provider: String,
    pub server: String,
    pub online: bool,
    pub latency_ms: f64,
    pub resolves_correctly: bool,
}

/// Ejecuta diagnóstico completo de 5 fases
#[tauri::command]
pub async fn run_full_diagnostic() -> Result<DiagnosticResult, String> {
    log::info!("Iniciando diagnóstico de red de 5 fases...");

    let mut result = DiagnosticResult {
        health: NetworkHealth::Down,
        score: 0.0,
        failure_point: FailurePoint::Adapter,
        adapter_ok: false,
        adapter_name: String::new(),
        router_ok: false,
        router_latency_ms: 0.0,
        isp_ok: false,
        isp_latency_ms: 0.0,
        dns_ok: false,
        dns_latency_ms: 0.0,
        internet_ok: false,
        internet_latency_ms: 0.0,
        recommendation: String::new(),
    };

    // Phase 1: Check adapter
    log::info!("Fase 1: Verificando adaptador de red...");
    let adapter = check_adapter().await?;
    if adapter.is_none() {
        result.recommendation = "No hay adaptador de red activo. Verifica tu conexión.".to_string();
        log::warn!("Fase 1 FALLIDA: Sin adaptador activo");
        return Ok(result);
    }

    result.adapter_ok = true;
    result.adapter_name = adapter.unwrap();
    log::info!("Fase 1 OK: Adaptador '{}'", result.adapter_name);

    // Phase 2: Check router (gateway) usando TCP ping
    log::info!("Fase 2: Verificando router/gateway...");
    let gateway = get_gateway().await?;
    if let Some(gw) = gateway {
        if let Some(latency) = tcp_ping_internal(&gw, 80)
            .await
            .or(tcp_ping_internal(&gw, 443).await)
            .or(ping_host_internal(&gw).await)
        {
            result.router_ok = true;
            result.router_latency_ms = latency;
            result.failure_point = FailurePoint::Isp;
            log::info!("Fase 2 OK: Gateway {} responde en {:.1}ms", gw, latency);
        } else {
            result.recommendation =
                "No se puede alcanzar el router. Verifica tu conexión local.".to_string();
            log::warn!("Fase 2 FALLIDA: No responde gateway {}", gw);
            return Ok(result);
        }
    } else {
        // Sin gateway = conexión directa al ISP (raro pero posible)
        log::info!("Fase 2: Sin gateway detectado, saltando al ISP...");
        result.router_ok = true;
        result.router_latency_ms = 0.0;
        result.failure_point = FailurePoint::Isp;
    }

    // Phase 3: Check ISP usando TCP ping al puerto 53 (más preciso)
    log::info!("Fase 3: Verificando conexión ISP...");
    if let Some(latency) = tcp_ping_internal("1.1.1.1", 53).await {
        result.isp_ok = true;
        result.isp_latency_ms = latency;
        result.failure_point = FailurePoint::Dns;
        log::info!("Fase 3 OK: ISP responde en {:.1}ms", latency);
    } else if let Some(latency) = ping_host_internal("1.1.1.1").await {
        result.isp_ok = true;
        result.isp_latency_ms = latency;
        result.failure_point = FailurePoint::Dns;
        log::info!(
            "Fase 3 OK (fallback ping): ISP responde en {:.1}ms",
            latency
        );
    } else {
        result.recommendation = "Sin conexión a internet. Contacta a tu ISP.".to_string();
        log::warn!("Fase 3 FALLIDA: Sin conexión externa");
        return Ok(result);
    }

    // Phase 4: Check DNS usando TCP ping al puerto 53
    log::info!("Fase 4: Verificando servicio DNS...");
    // Probar el DNS que debería estar configurado (o Cloudflare por defecto)
    if let Some(latency) = tcp_ping_internal("1.1.1.1", 53).await {
        result.dns_ok = true;
        result.dns_latency_ms = latency;
        result.failure_point = FailurePoint::Internet;
        log::info!("Fase 4 OK: DNS responde en {:.1}ms", latency);
    } else {
        result.recommendation =
            "DNS no responde. Prueba cambiar a Cloudflare (1.1.1.1).".to_string();
        log::warn!("Fase 4 FALLIDA: DNS no responde");
        return Ok(result);
    }

    // Phase 5: Check Internet (HTTP request real)
    log::info!("Fase 5: Verificando conectividad a internet...");
    if let Some(latency) = check_internet_connectivity().await {
        result.internet_ok = true;
        result.internet_latency_ms = latency;
        result.failure_point = FailurePoint::None;
        log::info!("Fase 5 OK: Internet responde en {:.1}ms", latency);
    } else {
        result.recommendation =
            "DNS funciona pero no hay acceso a internet. Posible problema con tu ISP.".to_string();
        log::warn!("Fase 5 FALLIDA: Sin acceso a internet");
        result.failure_point = FailurePoint::Internet;
    }

    // Calculate health based on weighted score
    // Thresholds REALISTAS basados en conexiones típicas:
    // - Router: <5ms = 100, <20ms = 95, <50ms = 85, <100ms = 70, >100ms = linear decay
    // - ISP: <20ms = 100, <50ms = 95, <100ms = 85, <200ms = 70
    // - DNS: <20ms = 100, <40ms = 95, <80ms = 85, <150ms = 70
    // - Internet: <100ms = 100, <200ms = 90, <300ms = 80, <500ms = 65

    // Función de scoring más justa que no penaliza latencias normales
    fn latency_to_score(latency: f64, excellent: f64, good: f64, fair: f64, poor: f64) -> f64 {
        if latency <= 0.0 {
            return 100.0;
        }
        if latency <= excellent {
            return 100.0;
        }
        if latency <= good {
            return 95.0 - ((latency - excellent) / (good - excellent)) * 5.0;
        }
        if latency <= fair {
            return 90.0 - ((latency - good) / (fair - good)) * 10.0;
        }
        if latency <= poor {
            return 80.0 - ((latency - fair) / (poor - fair)) * 15.0;
        }
        // Más allá de poor, decaimiento linear hasta 0
        (65.0 - ((latency - poor) / poor) * 30.0).max(0.0)
    }

    let router_score = if result.router_latency_ms == 0.0 {
        100.0
    } else {
        latency_to_score(result.router_latency_ms, 5.0, 20.0, 50.0, 100.0)
    };
    let isp_score = latency_to_score(result.isp_latency_ms, 20.0, 50.0, 100.0, 200.0);
    let dns_score = latency_to_score(result.dns_latency_ms, 20.0, 40.0, 80.0, 150.0);
    let internet_score = if result.internet_ok {
        latency_to_score(result.internet_latency_ms, 100.0, 200.0, 350.0, 500.0)
    } else {
        40.0 // Si DNS funciona pero internet no, dar puntos base reducidos
    };

    // Promedio ponderado: Router 10%, ISP 15%, DNS 25%, Internet 50%
    // Internet tiene más peso porque es lo que realmente importa al usuario
    let score =
        (router_score * 0.10) + (isp_score * 0.15) + (dns_score * 0.25) + (internet_score * 0.50);
    result.score = score.round();

    result.health = if score >= 85.0 {
        NetworkHealth::Excellent
    } else if score >= 70.0 {
        NetworkHealth::Good
    } else if score >= 50.0 {
        NetworkHealth::Fair
    } else if score >= 25.0 {
        NetworkHealth::Poor
    } else {
        NetworkHealth::Bad
    };

    // Recomendación más inteligente basada en qué fase es más lenta
    let latency_data = [
        ("router", result.router_latency_ms, 50.0),
        ("ISP", result.isp_latency_ms, 80.0),
        ("DNS", result.dns_latency_ms, 50.0),
    ];

    let slowest: Option<(&str, f64, f64)> = latency_data
        .iter()
        .filter(|(_, lat, threshold)| *lat > *threshold)
        .max_by(|a, b| (a.1 / a.2).partial_cmp(&(b.1 / b.2)).unwrap())
        .copied();

    result.recommendation = match result.health {
        NetworkHealth::Excellent => format!(
            "🚀 Tu conexión está funcionando perfectamente. Score: {:.0}/100",
            score
        ),
        NetworkHealth::Good => format!("✅ Tu conexión está bien. Score: {:.0}/100", score),
        NetworkHealth::Fair => {
            if let Some((component, lat, _)) = slowest {
                format!(
                    "⚠️ Conexión aceptable. {} un poco lento ({:.0}ms). Score: {:.0}/100",
                    component, lat, score
                )
            } else {
                format!("⚠️ Conexión aceptable. Score: {:.0}/100", score)
            }
        }
        NetworkHealth::Poor => format!(
            "🔴 Conexión lenta. Verifica tu red. Score: {:.0}/100",
            score
        ),
        NetworkHealth::Bad => format!(
            "❌ Conexión muy lenta. Contacta a tu ISP. Score: {:.0}/100",
            score
        ),
        NetworkHealth::Down => "❌ Sin conexión.".to_string(),
    };

    log::info!("Diagnóstico completo: {:?}, score: {:.1}, latencias: R={:.0}ms I={:.0}ms D={:.0}ms Net={:.0}ms", 
               result.health, score, result.router_latency_ms, result.isp_latency_ms, result.dns_latency_ms, result.internet_latency_ms);
    Ok(result)
}

/// Check rápido de conectividad
#[tauri::command]
pub async fn quick_check() -> Result<(bool, String), String> {
    if let Some(latency) = ping_host_internal("1.1.1.1").await {
        if latency < 50.0 {
            Ok((true, format!("Conexión OK ({:.0}ms)", latency)))
        } else {
            Ok((true, format!("Conexión lenta ({:.0}ms)", latency)))
        }
    } else {
        Ok((false, "Sin conexión a internet".to_string()))
    }
}

/// Comando Tauri para hacer ping a un host
#[tauri::command]
pub async fn ping_host(host: String) -> Result<PingResult, String> {
    match ping_host_internal(&host).await {
        Some(latency) => Ok(PingResult {
            success: true,
            host,
            latency_ms: Some(latency),
            error: None,
        }),
        None => Ok(PingResult {
            success: false,
            host,
            latency_ms: None,
            error: Some("Host inalcanzable".to_string()),
        }),
    }
}

/// DNS providers para health check
const DNS_HEALTH_PROVIDERS: &[(&str, &str)] = &[
    ("Cloudflare", "1.1.1.1"),
    ("Google", "8.8.8.8"),
    ("Quad9", "9.9.9.9"),
    ("OpenDNS", "208.67.222.222"),
    ("AdGuard", "94.140.14.14"),
    ("CleanBrowsing", "185.228.168.9"),
];

/// Comando Tauri para verificar salud de DNS
#[tauri::command]
pub async fn check_dns_health() -> Result<Vec<DnsHealthResult>, String> {
    let mut results = Vec::new();

    for (provider, server) in DNS_HEALTH_PROVIDERS {
        let start = Instant::now();

        // Verificar si el servidor responde a ping
        let online = ping_host_internal(server).await.is_some();
        let ping_latency = start.elapsed().as_millis() as f64;

        // Verificar si resuelve DNS correctamente
        let resolves = if online {
            resolve_dns_with_server("google.com", server).await
        } else {
            false
        };

        results.push(DnsHealthResult {
            provider: provider.to_string(),
            server: server.to_string(),
            online,
            latency_ms: if online { ping_latency } else { 0.0 },
            resolves_correctly: resolves,
        });
    }

    // Ordenar por latencia (menores primero)
    results.sort_by(|a, b| {
        if !a.online && !b.online {
            std::cmp::Ordering::Equal
        } else if !a.online {
            std::cmp::Ordering::Greater
        } else if !b.online {
            std::cmp::Ordering::Less
        } else {
            a.latency_ms
                .partial_cmp(&b.latency_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    Ok(results)
}

/// Resultado del check de un DNS individual
#[derive(Debug, serde::Serialize)]
pub struct SingleDnsHealthResult {
    pub latency_ms: f64,
    pub success: bool,
    pub resolves: bool,
}

/// Check de salud de un servidor DNS específico usando TCP ping al puerto 53
/// Esto es mucho más preciso que nslookup para medir latencia real
#[tauri::command]
pub async fn check_single_dns_health(server: String) -> Result<SingleDnsHealthResult, String> {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    let start = Instant::now();

    // Primero intentar TCP ping al puerto 53 (DNS) - método más rápido y preciso
    let addr: SocketAddr = format!("{}:53", server)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;

    let tcp_result = TcpStream::connect_timeout(&addr, Duration::from_millis(2000));
    let latency_ms = start.elapsed().as_millis() as f64;

    let success = tcp_result.is_ok();

    // Si TCP falló, intentar resolución DNS real como fallback
    let resolves = if !success {
        resolve_dns_with_server("google.com", &server).await
    } else {
        true // Si TCP funcionó, asumimos que puede resolver
    };

    Ok(SingleDnsHealthResult {
        latency_ms: if success {
            latency_ms
        } else if resolves {
            start.elapsed().as_millis() as f64
        } else {
            999.0
        },
        success: success || resolves,
        resolves,
    })
}

/// Función interna para resolver DNS usando un servidor específico
async fn resolve_dns_with_server(domain: &str, dns_server: &str) -> bool {
    #[cfg(windows)]
    let output = Command::new("nslookup")
        .args([domain, dns_server])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    #[cfg(not(windows))]
    let output = linux_command("nslookup")
        .args([domain, dns_server])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Verificar que se obtuvo una respuesta válida
            stdout.contains("Address") && !stdout.contains("can't find")
        }
        _ => false,
    }
}

async fn check_adapter() -> Result<Option<String>, String> {
    #[cfg(windows)]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-NetAdapter | Where-Object Status -eq 'Up' | Select-Object -First 1 -ExpandProperty Name",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    #[cfg(not(windows))]
    {
        return Ok(linux_primary_adapter());
    }

    #[cfg(windows)]
    {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            Ok(None)
        } else {
            Ok(Some(name))
        }
    }
}

async fn get_gateway() -> Result<Option<String>, String> {
    #[cfg(windows)]
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Select-Object -First 1).NextHop",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    #[cfg(not(windows))]
    {
        let route = linux_capture("ip", &["route", "show", "default"]).unwrap_or_default();
        let gateway = route
            .split_whitespace()
            .skip_while(|token| *token != "via")
            .nth(1)
            .unwrap_or("")
            .to_string();

        if gateway.is_empty() {
            Ok(None)
        } else {
            Ok(Some(gateway))
        }
    }

    #[cfg(windows)]
    {
        let gateway = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if gateway.is_empty() {
            Ok(None)
        } else {
            Ok(Some(gateway))
        }
    }
}

/// Función interna para ping (usada por diagnósticos)
async fn ping_host_internal(host: &str) -> Option<f64> {
    let start = Instant::now();

    #[cfg(windows)]
    let output = Command::new("ping")
        .args(["-n", "1", "-w", "2000", host])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    #[cfg(not(windows))]
    let output = linux_command("ping")
        .args(["-c", "1", "-W", "2", host])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse latency from ping output
        if let Some(time_str) = stdout.split("time=").nth(1) {
            if let Some(ms_str) = time_str.split("ms").next() {
                if let Ok(ms) = ms_str.trim().replace('<', "").parse::<f64>() {
                    return Some(ms);
                }
            }
        }
        // Fallback to measured time
        Some(start.elapsed().as_millis() as f64)
    } else {
        None
    }
}

async fn resolve_dns(domain: &str) -> Option<f64> {
    let start = Instant::now();

    #[cfg(windows)]
    let output = Command::new("nslookup")
        .args([domain])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    #[cfg(not(windows))]
    let output = linux_command("nslookup").args([domain]).output().ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("Address") {
            return Some(start.elapsed().as_millis() as f64);
        }
    }
    None
}

/// TCP ping interno - mucho más preciso para medir latencia
async fn tcp_ping_internal(host: &str, port: u16) -> Option<f64> {
    use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let start = Instant::now();

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = match addr_str.to_socket_addrs() {
        Ok(mut addrs) => addrs.next()?,
        Err(_) => return None,
    };

    let timeout = Duration::from_millis(2000);
    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(_) => Some(start.elapsed().as_secs_f64() * 1000.0),
        Err(_) => None,
    }
}

/// Verificar conectividad real a internet (HTTP HEAD request)
async fn check_internet_connectivity() -> Option<f64> {
    // Intentar conexión TCP al puerto 80 de varios hosts confiables
    let hosts = [
        ("www.google.com", 80),
        ("www.cloudflare.com", 80),
        ("www.microsoft.com", 80),
    ];

    for (host, port) in hosts.iter() {
        if let Some(latency) = tcp_ping_internal(host, *port).await {
            return Some(latency);
        }
    }

    // Fallback: verificar si podemos resolver y conectar
    #[cfg(windows)]
    {
        let start = Instant::now();
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Invoke-WebRequest -Uri 'http://www.msftconnecttest.com/connecttest.txt' -TimeoutSec 5 -UseBasicParsing).StatusCode",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim() == "200" {
                return Some(start.elapsed().as_millis() as f64);
            }
        }
    }

    None
}

/// Ejecutar el solucionador de problemas de red de Windows
#[tauri::command]
pub async fn run_windows_network_troubleshooter() -> Result<String, String> {
    log::info!("Ejecutando solucionador de problemas de red de Windows...");

    #[cfg(windows)]
    {
        Command::new("msdt.exe")
            .args(["/id", "NetworkDiagnosticsNetworkAdapter"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Error iniciando troubleshooter: {}", e))?;

        Ok(
            "Solucionador de problemas de red iniciado. Sigue las instrucciones en pantalla."
                .to_string(),
        )
    }

    #[cfg(not(windows))]
    {
        match linux_open_network_settings().or_else(|_| linux_open_system_settings()) {
            Ok(message) => Ok(format!(
                "Se abrió la herramienta de red del sistema. {}",
                message
            )),
            Err(err) => Err(format!(
                "No se encontró un asistente gráfico de red en este Linux: {}",
                err
            )),
        }
    }
}

/// Abrir herramienta del sistema de Windows
#[tauri::command]
pub async fn open_system_tool(tool: String) -> Result<String, String> {
    log::info!("Abriendo herramienta del sistema: {}", tool);

    #[cfg(windows)]
    {
        use std::process::Command as StdCommand;

        // Verificar que sea una herramienta permitida
        let allowed_tools = vec![
            "ncpa.cpl",     // Conexiones de red
            "devmgmt.msc",  // Administrador de dispositivos
            "cmd",          // Command Prompt
            "powershell",   // PowerShell
            "control",      // Panel de control
            "services.msc", // Servicios
            "eventvwr.msc", // Visor de eventos
            "resmon",       // Monitor de recursos
            "perfmon",      // Monitor de rendimiento
        ];

        if !allowed_tools.contains(&tool.as_str()) {
            return Err(format!("Herramienta no permitida: {}", tool));
        }

        StdCommand::new("cmd")
            .args(["/c", "start", "", &tool])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Error abriendo {}: {}", tool, e))?;

        Ok(format!("Herramienta {} iniciada", tool))
    }

    #[cfg(not(windows))]
    {
        match tool.as_str() {
            "network_connections" | "ncpa.cpl" => linux_open_network_settings(),
            "device_settings" | "devmgmt.msc" => linux_open_system_settings(),
            "terminal" | "cmd" | "powershell" => linux_open_terminal(),
            "network_snapshot" | "cmd /k ipconfig /all" => {
                let snapshot = linux_write_network_snapshot()?;
                let snapshot_path = snapshot.to_string_lossy().to_string();

                if linux_find_program("xdg-open").is_some() {
                    linux_spawn("xdg-open", &[snapshot_path.as_str()])?;
                    Ok("Resumen de red abierto en tu visor predeterminado".to_string())
                } else {
                    Ok(format!("Resumen de red guardado en {}", snapshot.display()))
                }
            }
            _ => Err(format!("Herramienta no permitida: {}", tool)),
        }
    }
}

/// Resultado de resolución DNS
#[derive(Debug, serde::Serialize)]
pub struct DnsResolutionResult {
    pub domain: String,
    pub success: bool,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

/// Medir tiempo de resolución DNS para un dominio usando el DNS del sistema
#[tauri::command]
pub async fn measure_dns_resolution(domain: String) -> Result<DnsResolutionResult, String> {
    match resolve_dns(&domain).await {
        Some(latency) => Ok(DnsResolutionResult {
            domain,
            success: true,
            latency_ms: Some(latency),
            error: None,
        }),
        None => Ok(DnsResolutionResult {
            domain,
            success: false,
            latency_ms: None,
            error: Some("No se pudo resolver el dominio".to_string()),
        }),
    }
}

/// Reset del stack de red de Windows
#[tauri::command]
pub async fn reset_network_stack() -> Result<String, String> {
    log::info!("Reseteando stack de red...");

    #[cfg(windows)]
    {
        // Resetear Winsock
        let _ = Command::new("netsh")
            .args(["winsock", "reset"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        // Resetear IP
        let _ = Command::new("netsh")
            .args(["int", "ip", "reset"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        // Flush DNS
        let _ = Command::new("ipconfig")
            .args(["/flushdns"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        // Renovar IP
        let _ = Command::new("ipconfig")
            .args(["/release"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        let _ = Command::new("ipconfig")
            .args(["/renew"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        Ok("Stack de red reseteado. Puede ser necesario reiniciar.".to_string())
    }

    #[cfg(not(windows))]
    {
        let mut actions = Vec::new();

        if linux_find_program("resolvectl").is_some() {
            if linux_command("resolvectl")
                .args(["flush-caches"])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                actions.push("caché DNS limpiada".to_string());
            }
        }

        if let Some(adapter) = linux_primary_adapter() {
            if linux_find_program("nmcli").is_some() {
                let reapplied = linux_command("nmcli")
                    .args(["device", "reapply", adapter.as_str()])
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false);

                if reapplied {
                    actions.push(format!("configuración reaplicada sobre {}", adapter));
                } else if let Some(connection) = linux_active_connection(&adapter) {
                    if linux_command("nmcli")
                        .args([
                            "connection",
                            "up",
                            connection.as_str(),
                            "ifname",
                            adapter.as_str(),
                        ])
                        .output()
                        .map(|output| output.status.success())
                        .unwrap_or(false)
                    {
                        actions.push(format!("conexión {} reactivada", connection));
                    }
                }
            }
        }

        if actions.is_empty() {
            Err("No se pudo aplicar un reset de red seguro en este Linux".to_string())
        } else {
            Ok(format!(
                "Reset Linux aplicado: {}. Si el problema persiste, reconecta la interfaz o reinicia NetworkManager.",
                actions.join(", ")
            ))
        }
    }
}
