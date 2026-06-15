//! Optimizer commands module
//!
//! Comandos Tauri para optimizaciones TCP/IP.
//!
//! By LOUST (www.loust.pro)

use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(not(windows))]
use std::path::Path;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(not(windows))]
fn linux_command(program: &str) -> Command {
    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            let candidate = directory.join(program);
            if candidate.exists() {
                return Command::new(candidate);
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
            return Command::new(candidate);
        }
    }

    Command::new(program)
}

#[cfg(not(windows))]
fn linux_sysctl_get(key: &str) -> Option<String> {
    let output = linux_command("sysctl").args(["-n", key]).output().ok()?;
    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(not(windows))]
fn linux_tcp_1323_opts() -> u32 {
    let window_scaling = linux_sysctl_get("net.ipv4.tcp_window_scaling").as_deref() == Some("1");
    let timestamps = linux_sysctl_get("net.ipv4.tcp_timestamps").as_deref() == Some("1");

    u32::from(window_scaling) + (u32::from(timestamps) << 1)
}

#[cfg(not(windows))]
fn linux_current_settings() -> TcpSettings {
    let autotuning = if linux_sysctl_get("net.ipv4.tcp_moderate_rcvbuf").as_deref() == Some("1") {
        "enabled"
    } else {
        "disabled"
    }
    .to_string();

    let rss = if linux_sysctl_get("net.core.rps_sock_flow_entries")
        .map(|value| value != "0")
        .unwrap_or(false)
    {
        "enabled"
    } else {
        "kernel-managed"
    }
    .to_string();

    let rsc = "kernel-managed".to_string();

    let ecn = match linux_sysctl_get("net.ipv4.tcp_ecn").as_deref() {
        Some("0") => "disabled",
        Some("1") => "enabled",
        Some("2") => "passive",
        Some(_) => "custom",
        None => "unknown",
    }
    .to_string();

    let timestamps = if linux_sysctl_get("net.ipv4.tcp_timestamps").as_deref() == Some("1") {
        "enabled"
    } else {
        "disabled"
    }
    .to_string();

    let congestion_provider = linux_sysctl_get("net.ipv4.tcp_congestion_control")
        .unwrap_or_else(|| "unknown".to_string());

    let fast_open = if linux_sysctl_get("net.ipv4.tcp_fastopen")
        .map(|value| value != "0")
        .unwrap_or(false)
    {
        "enabled"
    } else {
        "disabled"
    }
    .to_string();

    let hystart = match linux_sysctl_get("net.ipv4.tcp_slow_start_after_idle").as_deref() {
        Some("0") => "optimized",
        Some("1") => "default",
        Some(_) => "custom",
        None => "kernel-managed",
    }
    .to_string();

    let prr = if linux_sysctl_get("net.ipv4.tcp_recovery").is_some() {
        "enabled"
    } else {
        "kernel-managed"
    }
    .to_string();

    let pacing = linux_sysctl_get("net.core.default_qdisc")
        .unwrap_or_else(|| "unknown".to_string());

    let rack = match linux_sysctl_get("net.ipv4.tcp_recovery").as_deref() {
        Some("0") => "disabled",
        Some(_) => "enabled",
        None => "unknown",
    }
    .to_string();

    TcpSettings {
        autotuning,
        rss,
        rsc,
        ecn,
        timestamps,
        chimney: "not-applicable".to_string(),
        congestion_provider,
        fast_open,
        hystart,
        prr,
        pacing,
        initial_rto: "kernel default".to_string(),
        rack,
        nagle_disabled: false,
        network_throttling_disabled: false,
        tcp_1323_opts: linux_tcp_1323_opts(),
    }
}

#[cfg(not(windows))]
fn linux_sysctl_set(key: &str, value: &str) -> Result<(), String> {
    let assignment = format!("{}={}", key, value);
    let output = linux_command("sysctl")
        .args(["-w", &assignment])
        .output()
        .map_err(|e| format!("Error ejecutando sysctl: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[cfg(not(windows))]
fn linux_available_congestion_controls() -> Vec<String> {
    linux_sysctl_get("net.ipv4.tcp_available_congestion_control")
        .unwrap_or_default()
        .split_whitespace()
        .map(|value| value.to_string())
        .collect()
}

#[cfg(not(windows))]
fn linux_preferred_congestion(profile: &str) -> Option<String> {
    let available = linux_available_congestion_controls();
    let preferred = if matches!(profile, "Balanced" | "Aggressive") {
        ["bbr", "cubic"]
    } else {
        ["cubic", "bbr"]
    };

    preferred
        .into_iter()
        .find(|candidate| available.iter().any(|value| value == candidate))
        .map(|value| value.to_string())
}

#[cfg(not(windows))]
fn linux_apply_sysctl(
    key: &str,
    value: &str,
    label: &str,
    applied: &mut Vec<String>,
) -> Result<(), String> {
    match linux_sysctl_set(key, value) {
        Ok(()) => {
            applied.push(label.to_string());
            Ok(())
        }
        Err(err) => {
            let lower = err.to_lowercase();
            if lower.contains("permission denied")
                || lower.contains("operation not permitted")
                || lower.contains("read-only")
            {
                Err(format!(
                    "Se requieren permisos root para modificar {}: {}",
                    key, err
                ))
            } else {
                log::warn!("Linux sysctl {}={} no aplicado: {}", key, value, err);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TcpSettings {
    pub autotuning: String,
    pub rss: String,
    pub rsc: String,
    pub ecn: String,
    pub timestamps: String,
    pub chimney: String,
    pub congestion_provider: String,
    // Campos adicionales para optimizaciones avanzadas
    pub fast_open: String,
    pub hystart: String,
    pub prr: String,
    pub pacing: String,
    pub initial_rto: String,
    pub rack: String,
    // Nuevas optimizaciones del CLI Python
    pub nagle_disabled: bool,
    pub network_throttling_disabled: bool,
    pub tcp_1323_opts: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OptimizationProfile {
    Conservative,
    Balanced,
    Aggressive,
}

/// Información de una optimización individual
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptimizationInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub risk_level: String,
    pub applied: bool,
}

/// Obtiene configuración TCP actual
#[tauri::command]
pub async fn get_current_settings() -> Result<TcpSettings, String> {
    get_current_settings_internal()
}

/// Función interna para obtener settings (usable desde tray)
pub fn get_current_settings_internal() -> Result<TcpSettings, String> {
    #[cfg(not(windows))]
    {
        return Ok(linux_current_settings());
    }

    #[cfg(windows)]
    {
        // Usamos PowerShell para forzar UTF-8 output
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; netsh int tcp show global",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Log para debug
        log::info!("TCP Settings - netsh output length: {} bytes", stdout.len());

        // Función que busca por múltiples claves (español e inglés)
        // Busca de forma case-insensitive y extrae el valor después del ':'
        // Para evitar falsos positivos, preferimos que la línea COMIENCE con la clave
        fn extract_value_multi(text: &str, keys: &[&str]) -> String {
            for line in text.lines() {
                let line_trimmed = line.trim();
                let line_lower = line_trimmed.to_lowercase();

                for key in keys {
                    let key_lower = key.to_lowercase();
                    // Remover los dos puntos del key si los tiene para buscar
                    let key_clean = key_lower.trim_end_matches(':');

                    // Preferir match al inicio de línea, o contiene si tiene más contexto
                    let matches = if key_clean.len() < 15 {
                        // Para claves cortas como "Fast Open", verificar inicio de línea
                        line_lower.starts_with(&key_clean)
                    } else {
                        // Para claves largas, contains está bien
                        line_lower.contains(&key_clean)
                    };

                    if matches {
                        if let Some(value) = line_trimmed.split(':').nth(1) {
                            let v = value.trim();
                            if !v.is_empty() {
                                return v.to_string();
                            }
                        }
                    }
                }
            }
            "unknown".to_string()
        }

        // Parsear los valores usando nombres en español e inglés
        // RSS - "Receive-Side Scaling State" en inglés, "Estado de escalado de lado de recepción" en español
        let rss = extract_value_multi(
            &stdout,
            &[
                "Receive-Side Scaling State",
                "escalado de lado de recepción",
                "Estado de escalado",
            ],
        );

        // Autotuning - "Receive Window Auto-Tuning Level" en inglés
        let autotuning = extract_value_multi(
            &stdout,
            &[
                "Receive Window Auto-Tuning Level",
                "ajuste automático de ventana",
                "Nivel de ajuste",
            ],
        );

        // RSC - "Receive Segment Coalescing State" en inglés
        let rsc = extract_value_multi(
            &stdout,
            &[
                "Receive Segment Coalescing State",
                "fusión de segmento de recepción",
                "Estado de fusión",
            ],
        );

        // ECN
        let ecn = extract_value_multi(&stdout, &["ECN Capability", "Funcionalidad de ECN"]);

        // Timestamps
        let timestamps = extract_value_multi(
            &stdout,
            &["RFC 1323 Timestamps", "Marcas de hora RFC 1323"],
        );

        // Chimney (deprecado en Windows 10+)
        let chimney = extract_value_multi(&stdout, &["Chimney Offload State"]);

        // Congestion Provider
        let congestion_provider = extract_value_multi(
            &stdout,
            &[
                "Add-On Congestion Control Provider",
                "Proveedor de control de congestión",
                "control de congestión",
            ],
        );

        // Fast Open - hay que evitar "Reserva Fast Open", buscamos inicio de línea
        let fast_open = extract_value_multi(&stdout, &["Fast Open"]);

        // HyStart - aparece como "HyStart" directamente
        let hystart = extract_value_multi(&stdout, &["HyStart"]);

        // PRR - Proportional Rate Reduction
        let prr = extract_value_multi(
            &stdout,
            &["Proportional Rate Reduction", "tasa proporcional"],
        );

        // Pacing Profile
        let pacing = extract_value_multi(&stdout, &["Pacing Profile", "Perfil de velocidad"]);

        // Initial RTO
        let initial_rto_raw = extract_value_multi(&stdout, &["Initial RTO", "RTO inicial"]);

        // Log valores extraídos para debugging
        log::info!("TCP Settings parsed - RSS: {}, RSC: {}, Autotuning: {}, ECN: {}, FastOpen: {}, HyStart: {}, PRR: {}",
            rss, rsc, autotuning, ecn, fast_open, hystart, prr);

        // Formatear initial_rto
        let initial_rto = if initial_rto_raw != "unknown" {
            if initial_rto_raw.ends_with("ms") {
                initial_rto_raw
            } else {
                format!("{}ms", initial_rto_raw)
            }
        } else {
            "3000ms".to_string()
        };

        Ok(TcpSettings {
            autotuning,
            rss,
            rsc,
            ecn,
            timestamps,
            chimney,
            congestion_provider,
            fast_open,
            hystart,
            prr,
            pacing,
            initial_rto,
            rack: "via PRR".to_string(), // RACK-TLP se maneja junto con PRR en Windows
            // Nuevas optimizaciones
            nagle_disabled: get_registry_dword("TcpNoDelay").map_or(false, |v| v == 1),
            network_throttling_disabled: get_registry_dword("NetworkThrottlingIndex")
                .map_or(false, |v| v == 0xffffffff),
            tcp_1323_opts: get_registry_dword("Tcp1323Opts").unwrap_or(0),
        })
    }
}

/// Aplica perfil de optimización (comando Tauri)
#[tauri::command]
pub async fn apply_profile(profile: OptimizationProfile) -> Result<Vec<String>, String> {
    let profile_str = match profile {
        OptimizationProfile::Conservative => "Conservative",
        OptimizationProfile::Balanced => "Balanced",
        OptimizationProfile::Aggressive => "Aggressive",
    };
    apply_profile_internal(profile_str)
}

/// Función interna para aplicar perfil (usable desde tray)
pub fn apply_profile_internal(profile: &str) -> Result<Vec<String>, String> {
    let mut applied = Vec::new();

    log::info!("Aplicando perfil de optimización: {}", profile);

    #[cfg(not(windows))]
    {
        match profile {
            "Conservative" => {
                linux_apply_sysctl(
                    "net.ipv4.tcp_moderate_rcvbuf",
                    "1",
                    "TCP autotuning enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_window_scaling",
                    "1",
                    "Window scaling enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.core.default_qdisc",
                    "fq_codel",
                    "Default qdisc set to fq_codel",
                    &mut applied,
                )?;
            }
            "Balanced" => {
                linux_apply_sysctl(
                    "net.ipv4.tcp_moderate_rcvbuf",
                    "1",
                    "TCP autotuning enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_window_scaling",
                    "1",
                    "Window scaling enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_ecn",
                    "1",
                    "ECN enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_timestamps",
                    "1",
                    "TCP timestamps enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.core.default_qdisc",
                    "fq",
                    "Default qdisc set to fq",
                    &mut applied,
                )?;

                if let Some(congestion) = linux_preferred_congestion(profile) {
                    linux_apply_sysctl(
                        "net.ipv4.tcp_congestion_control",
                        &congestion,
                        &format!("Congestion control set to {}", congestion),
                        &mut applied,
                    )?;
                }
            }
            "Aggressive" => {
                linux_apply_sysctl(
                    "net.ipv4.tcp_moderate_rcvbuf",
                    "1",
                    "TCP autotuning enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_window_scaling",
                    "1",
                    "Window scaling enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_ecn",
                    "1",
                    "ECN enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_timestamps",
                    "1",
                    "TCP timestamps enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.core.default_qdisc",
                    "fq",
                    "Default qdisc set to fq",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_fastopen",
                    "3",
                    "TCP Fast Open enabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_slow_start_after_idle",
                    "0",
                    "Slow-start after idle disabled",
                    &mut applied,
                )?;
                linux_apply_sysctl(
                    "net.ipv4.tcp_mtu_probing",
                    "1",
                    "TCP MTU probing enabled",
                    &mut applied,
                )?;

                if let Some(congestion) = linux_preferred_congestion(profile) {
                    linux_apply_sysctl(
                        "net.ipv4.tcp_congestion_control",
                        &congestion,
                        &format!("Congestion control set to {}", congestion),
                        &mut applied,
                    )?;
                }
            }
            _ => return Err(format!("Perfil desconocido: {}", profile)),
        }

        if applied.is_empty() {
            return Err(
                "No se aplicaron optimizaciones Linux; faltan permisos o soporte del kernel"
                    .to_string(),
            );
        }

        log::info!(
            "Perfil Linux {} aplicado: {} optimizaciones",
            profile,
            applied.len()
        );
        return Ok(applied);
    }

    #[cfg(windows)]
    {
        match profile {
            "Conservative" => {
                // Solo optimizaciones seguras
                run_netsh("int tcp set global rss=enabled")?;
                applied.push("RSS enabled".to_string());

                run_netsh("int tcp set global rsc=enabled")?;
                applied.push("RSC enabled".to_string());

                run_netsh("int tcp set global autotuninglevel=normal")?;
                applied.push("Autotuning normal".to_string());
            }
            "Balanced" => {
                // Incluye ECN y timestamps
                run_netsh("int tcp set global rss=enabled")?;
                applied.push("RSS enabled".to_string());

                run_netsh("int tcp set global rsc=enabled")?;
                applied.push("RSC enabled".to_string());

                run_netsh("int tcp set global autotuninglevel=normal")?;
                applied.push("Autotuning normal".to_string());

                run_netsh("int tcp set global ecncapability=enabled")?;
                applied.push("ECN enabled".to_string());

                run_netsh("int tcp set global timestamps=enabled")?;
                applied.push("Timestamps enabled".to_string());

                // Registry settings (require admin)
                if set_registry_dword("EnableHyStart", 1).is_ok() {
                    applied.push("HyStart++ enabled".to_string());
                }

                if set_registry_dword("EnablePrr", 1).is_ok() {
                    applied.push("PRR enabled".to_string());
                }
            }
            "Aggressive" => {
                // Todas las optimizaciones
                run_netsh("int tcp set global rss=enabled")?;
                applied.push("RSS enabled".to_string());

                run_netsh("int tcp set global rsc=enabled")?;
                applied.push("RSC enabled".to_string());

                run_netsh("int tcp set global autotuninglevel=normal")?;
                applied.push("Autotuning normal".to_string());

                run_netsh("int tcp set global ecncapability=enabled")?;
                applied.push("ECN enabled".to_string());

                run_netsh("int tcp set global timestamps=enabled")?;
                applied.push("Timestamps enabled".to_string());

                let _ = run_netsh("int tcp set global chimney=disabled");
                applied.push("Chimney disabled".to_string());

                // Registry settings
                if set_registry_dword("EnableHyStart", 1).is_ok() {
                    applied.push("HyStart++ enabled".to_string());
                }

                if set_registry_dword("EnablePrr", 1).is_ok() {
                    applied.push("PRR enabled".to_string());
                }

                if set_registry_dword("EnableTFO", 1).is_ok() {
                    applied.push("TCP Fast Open enabled".to_string());
                }

                if set_registry_dword("EnableWsd", 0).is_ok() {
                    applied.push("TCP Pacing enabled".to_string());
                }

                if set_registry_dword("TcpInitialRto", 1000).is_ok() {
                    applied.push("Initial RTO reduced".to_string());
                }

                // === NUEVAS OPTIMIZACIONES DEL CLI PYTHON ===

                // Network Throttling Index: Elimina el delay artificial de 10ms
                // https://docs.microsoft.com/en-us/windows/win32/api/qos2/
                // Valor 0xFFFFFFFF = sin throttling
                if set_registry_dword("NetworkThrottlingIndex", 0xffffffff).is_ok() {
                    applied.push(
                        "Network Throttling disabled (10ms delay removed)".to_string(),
                    );
                }

                // Nagle Algorithm: Desactivar para reducir latencia en gaming/realtime
                // TcpNoDelay=1 = Nagle desactivado
                if set_registry_dword("TcpNoDelay", 1).is_ok() {
                    applied.push("Nagle Algorithm disabled (lower latency)".to_string());
                }

                // TCP Window Scaling (Tcp1323Opts)
                // Bit 0 (valor 1): Window scaling
                // Bit 1 (valor 2): Timestamps
                // Valor 3 = ambos habilitados
                if set_registry_dword("Tcp1323Opts", 3).is_ok() {
                    applied.push(
                        "TCP Window Scaling + Timestamps (Tcp1323Opts=3)".to_string(),
                    );
                }
            }
            _ => {
                return Err(format!("Perfil desconocido: {}", profile));
            }
        }

        log::info!(
            "Perfil {} aplicado: {} optimizaciones",
            profile,
            applied.len()
        );
        Ok(applied)
    }
}

/// Restaura valores por defecto (comando Tauri)
#[tauri::command]
pub async fn reset_to_defaults() -> Result<Vec<String>, String> {
    reset_to_defaults_internal()
}

/// Función interna para resetear (usable desde tray)
pub fn reset_to_defaults_internal() -> Result<Vec<String>, String> {
    let mut reset = Vec::new();

    log::info!("Restaurando configuración TCP/IP por defecto...");

    #[cfg(not(windows))]
    {
        linux_apply_sysctl(
            "net.ipv4.tcp_moderate_rcvbuf",
            "1",
            "TCP autotuning reset",
            &mut reset,
        )?;
        linux_apply_sysctl(
            "net.ipv4.tcp_window_scaling",
            "1",
            "Window scaling reset",
            &mut reset,
        )?;
        linux_apply_sysctl(
            "net.ipv4.tcp_ecn",
            "2",
            "ECN reset",
            &mut reset,
        )?;
        linux_apply_sysctl(
            "net.ipv4.tcp_timestamps",
            "1",
            "TCP timestamps reset",
            &mut reset,
        )?;
        linux_apply_sysctl(
            "net.core.default_qdisc",
            "fq_codel",
            "Default qdisc reset to fq_codel",
            &mut reset,
        )?;
        linux_apply_sysctl(
            "net.ipv4.tcp_fastopen",
            "1",
            "TCP Fast Open reset",
            &mut reset,
        )?;
        linux_apply_sysctl(
            "net.ipv4.tcp_slow_start_after_idle",
            "1",
            "Slow-start after idle reset",
            &mut reset,
        )?;
        linux_apply_sysctl(
            "net.ipv4.tcp_mtu_probing",
            "0",
            "TCP MTU probing reset",
            &mut reset,
        )?;

        if let Some(congestion) = linux_preferred_congestion("Conservative") {
            linux_apply_sysctl(
                "net.ipv4.tcp_congestion_control",
                &congestion,
                &format!("Congestion control reset to {}", congestion),
                &mut reset,
            )?;
        }

        if reset.is_empty() {
            return Err(
                "No se restauraron valores Linux; faltan permisos o soporte del kernel"
                    .to_string(),
            );
        }

        log::info!("Configuración Linux restaurada: {} cambios", reset.len());
        return Ok(reset);
    }

    #[cfg(windows)]
    {
        if run_netsh("int tcp set global autotuninglevel=normal").is_ok() {
            reset.push("Autotuning reset".to_string());
        }

        if run_netsh("int tcp set global ecncapability=default").is_ok() {
            reset.push("ECN reset".to_string());
        }

        if run_netsh("int tcp set global timestamps=disabled").is_ok() {
            reset.push("Timestamps reset".to_string());
        }

        // Delete registry keys (ignorar errores si no existen)
        let _ = delete_registry_value("EnableHyStart");
        reset.push("HyStart++ removed".to_string());

        let _ = delete_registry_value("EnablePrr");
        reset.push("PRR removed".to_string());

        let _ = delete_registry_value("EnableTFO");
        reset.push("TCP Fast Open removed".to_string());

        let _ = delete_registry_value("EnableWsd");
        reset.push("TCP Pacing removed".to_string());

        let _ = delete_registry_value("TcpInitialRto");
        reset.push("Initial RTO reset".to_string());

        // Reset nuevas optimizaciones
        let _ = delete_registry_value("NetworkThrottlingIndex");
        reset.push("Network Throttling reset".to_string());

        let _ = delete_registry_value("TcpNoDelay");
        reset.push("Nagle Algorithm restored".to_string());

        let _ = delete_registry_value("Tcp1323Opts");
        reset.push("Tcp1323Opts reset".to_string());

        log::info!("Configuración restaurada: {} cambios", reset.len());
        Ok(reset)
    }
}

/// Obtener lista de todas las optimizaciones disponibles
#[tauri::command]
pub async fn get_available_optimizations() -> Result<Vec<OptimizationInfo>, String> {
    let current = get_current_settings_internal()?;

    #[cfg(windows)]
    let hystart_applied = check_registry_value("EnableHyStart");
    #[cfg(not(windows))]
    let hystart_applied = current.hystart.to_lowercase() != "default";

    #[cfg(windows)]
    let prr_applied = check_registry_value("EnablePrr");
    #[cfg(not(windows))]
    let prr_applied = current.prr.to_lowercase().contains("enabled");

    #[cfg(windows)]
    let tfo_applied = check_registry_value("EnableTFO");
    #[cfg(not(windows))]
    let tfo_applied = current.fast_open.to_lowercase().contains("enabled");

    #[cfg(windows)]
    let pacing_applied = get_registry_dword("EnableWsd") == Some(0);
    #[cfg(not(windows))]
    let pacing_applied = matches!(current.pacing.as_str(), "fq" | "fq_codel");

    #[cfg(windows)]
    let initial_rto_applied = get_registry_dword("TcpInitialRto").map_or(false, |v| v <= 1000);
    #[cfg(not(windows))]
    let initial_rto_applied = current.initial_rto != "kernel default";

    Ok(vec![
        OptimizationInfo {
            id: "rss".to_string(),
            name: "Receive-Side Scaling (RSS)".to_string(),
            description: "Distribuye el procesamiento de paquetes entre múltiples CPUs".to_string(),
            category: "Network".to_string(),
            risk_level: "low".to_string(),
            applied: current.rss.to_lowercase().contains("enabled"),
        },
        OptimizationInfo {
            id: "rsc".to_string(),
            name: "Receive Segment Coalescing (RSC)".to_string(),
            description: "Combina segmentos TCP para reducir overhead de CPU".to_string(),
            category: "Network".to_string(),
            risk_level: "low".to_string(),
            applied: current.rsc.to_lowercase().contains("enabled"),
        },
        OptimizationInfo {
            id: "ecn".to_string(),
            name: "Explicit Congestion Notification (ECN)".to_string(),
            description: "Notificación de congestión sin pérdida de paquetes".to_string(),
            category: "TCP".to_string(),
            risk_level: "low".to_string(),
            applied: current.ecn.to_lowercase().contains("enabled"),
        },
        OptimizationInfo {
            id: "timestamps".to_string(),
            name: "TCP Timestamps".to_string(),
            description: "Mejora la medición de RTT y protección PAWS".to_string(),
            category: "TCP".to_string(),
            risk_level: "low".to_string(),
            applied: current.timestamps.to_lowercase().contains("enabled"),
        },
        OptimizationInfo {
            id: "hystart".to_string(),
            name: "HyStart++".to_string(),
            description: "Slow-start mejorado que evita bufferbloat".to_string(),
            category: "Congestion".to_string(),
            risk_level: "medium".to_string(),
            applied: hystart_applied,
        },
        OptimizationInfo {
            id: "prr".to_string(),
            name: "Proportional Rate Reduction (PRR)".to_string(),
            description: "Recuperación suave de pérdidas de paquetes".to_string(),
            category: "Congestion".to_string(),
            risk_level: "medium".to_string(),
            applied: prr_applied,
        },
        OptimizationInfo {
            id: "tfo".to_string(),
            name: "TCP Fast Open (TFO)".to_string(),
            description: "Envía datos en SYN, ahorra 1 RTT".to_string(),
            category: "TCP".to_string(),
            risk_level: "medium".to_string(),
            applied: tfo_applied,
        },
        OptimizationInfo {
            id: "pacing".to_string(),
            name: "TCP Pacing".to_string(),
            description: "Envío suave de paquetes, reduce bufferbloat".to_string(),
            category: "Congestion".to_string(),
            risk_level: "medium".to_string(),
            applied: pacing_applied,
        },
        OptimizationInfo {
            id: "initial_rto".to_string(),
            name: "Initial RTO Reducido".to_string(),
            description: "Timeout inicial reducido de 3s a 1s".to_string(),
            category: "TCP".to_string(),
            risk_level: "medium".to_string(),
            applied: initial_rto_applied,
        },
        // === NUEVAS OPTIMIZACIONES DEL CLI PYTHON ===
        OptimizationInfo {
            id: "network_throttling".to_string(),
            name: "Network Throttling Index".to_string(),
            description: "Elimina el delay artificial de 10ms que Windows aplica a aplicaciones multimedia. Mejora latencia en gaming y VoIP.".to_string(),
            category: "Gaming".to_string(),
            risk_level: "medium".to_string(),
            applied: current.network_throttling_disabled,
        },
        OptimizationInfo {
            id: "nagle".to_string(),
            name: "Nagle Algorithm (TcpNoDelay)".to_string(),
            description: "Desactiva el algoritmo de Nagle que agrupa paquetes pequeños. Reduce latencia en tiempo real y gaming.".to_string(),
            category: "Gaming".to_string(),
            risk_level: "medium".to_string(),
            applied: current.nagle_disabled,
        },
        OptimizationInfo {
            id: "tcp_1323_opts".to_string(),
            name: "TCP Window Scaling (RFC 1323)".to_string(),
            description: "Habilita window scaling y timestamps via registro. Permite ventanas TCP mayores a 64KB para conexiones de alta velocidad.".to_string(),
            category: "TCP".to_string(),
            risk_level: "low".to_string(),
            applied: current.tcp_1323_opts >= 3,
        },
    ])
}

#[cfg(windows)]
fn run_netsh(args: &str) -> Result<(), String> {
    let full_args: Vec<&str> = args.split_whitespace().collect();

    #[cfg(windows)]
    let output = Command::new("netsh")
        .args(&full_args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    #[cfg(not(windows))]
    let output = Command::new("netsh")
        .args(&full_args)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("netsh {} failed", args))
    }
}

#[cfg(windows)]
fn set_registry_dword(name: &str, value: u32) -> Result<(), String> {
    let command = format!(
        r#"reg add "HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters" /v {} /t REG_DWORD /d {} /f"#,
        name, value
    );

    #[cfg(windows)]
    let output = Command::new("cmd")
        .args(["/c", &command])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    #[cfg(not(windows))]
    let output = Command::new("cmd")
        .args(["/c", &command])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Failed to set registry {}", name))
    }
}

#[cfg(windows)]
fn delete_registry_value(name: &str) -> Result<(), String> {
    let command = format!(
        r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters" /v {} /f"#,
        name
    );

    // Ignore error if value doesn't exist
    #[cfg(windows)]
    let _ = Command::new("cmd")
        .args(["/c", &command])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    #[cfg(not(windows))]
    let _ = Command::new("cmd").args(["/c", &command]).output();

    Ok(())
}

#[cfg(windows)]
fn check_registry_value(name: &str) -> bool {
    let command = format!(
        r#"reg query "HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters" /v {}"#,
        name
    );

    #[cfg(windows)]
    let output = Command::new("cmd")
        .args(["/c", &command])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    #[cfg(not(windows))]
    let output = Command::new("cmd").args(["/c", &command]).output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

#[cfg(windows)]
fn get_registry_dword(name: &str) -> Option<u32> {
    let command = format!(
        r#"reg query "HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters" /v {}"#,
        name
    );

    #[cfg(windows)]
    let output = Command::new("cmd")
        .args(["/c", &command])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    #[cfg(not(windows))]
    let output = Command::new("cmd").args(["/c", &command]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse output like "    TcpInitialRto    REG_DWORD    0x3e8"
    for line in stdout.lines() {
        if line.contains(name) {
            // Find the hex value (0x...)
            if let Some(hex_pos) = line.find("0x") {
                let hex_str = &line[hex_pos..];
                let end = hex_str
                    .find(|c: char| !c.is_ascii_hexdigit() && c != 'x')
                    .unwrap_or(hex_str.len());
                if let Ok(val) = u32::from_str_radix(&hex_str[2..end], 16) {
                    return Some(val);
                }
            }
        }
    }
    None
}
