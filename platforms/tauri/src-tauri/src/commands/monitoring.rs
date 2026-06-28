//! Monitoring Commands
//!
//! Comandos para monitoreo en tiempo real de la red.
//!
//! By LOUST (www.loust.pro)

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// CREATE_NO_WINDOW flag for Windows
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Estado del monitoreo
static MONITORING_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Métricas de red en tiempo real
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub latency_ms: f64,
    pub packets_sent_per_sec: u64,
    pub packets_recv_per_sec: u64,
    pub errors_in: u64,
    pub errors_out: u64,
    pub drops_in: u64,
    pub drops_out: u64,
    pub timestamp: String,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            download_mbps: 0.0,
            upload_mbps: 0.0,
            latency_ms: 0.0,
            packets_sent_per_sec: 0,
            packets_recv_per_sec: 0,
            errors_in: 0,
            errors_out: 0,
            drops_in: 0,
            drops_out: 0,
            timestamp: chrono::Local::now().to_rfc3339(),
        }
    }
}

/// Iniciar monitoreo de red en tiempo real
#[tauri::command]
pub async fn start_monitoring(
    app: AppHandle,
    adapter: String,
    interval_ms: u64,
) -> Result<(), String> {
    if MONITORING_ACTIVE.load(Ordering::SeqCst) {
        return Err("El monitoreo ya está activo".to_string());
    }

    MONITORING_ACTIVE.store(true, Ordering::SeqCst);
    let interval = Duration::from_millis(interval_ms.max(500));

    // Spawn background task
    std::thread::spawn(move || {
        let mut prev_bytes_recv: u64 = 0;
        let mut prev_bytes_sent: u64 = 0;
        let mut prev_packets_recv: u64 = 0;
        let mut prev_packets_sent: u64 = 0;

        while MONITORING_ACTIVE.load(Ordering::SeqCst) {
            let metrics = match get_adapter_metrics(&adapter) {
                Ok((bytes_recv, bytes_sent, packets_recv, packets_sent, errors, drops)) => {
                    let interval_secs = interval.as_secs_f64();

                    // Calcular tasas
                    let download_bytes = bytes_recv.saturating_sub(prev_bytes_recv);
                    let upload_bytes = bytes_sent.saturating_sub(prev_bytes_sent);
                    let pkts_recv_delta = packets_recv.saturating_sub(prev_packets_recv);
                    let pkts_sent_delta = packets_sent.saturating_sub(prev_packets_sent);

                    let download_mbps =
                        (download_bytes as f64 * 8.0) / (interval_secs * 1_000_000.0);
                    let upload_mbps = (upload_bytes as f64 * 8.0) / (interval_secs * 1_000_000.0);

                    prev_bytes_recv = bytes_recv;
                    prev_bytes_sent = bytes_sent;
                    prev_packets_recv = packets_recv;
                    prev_packets_sent = packets_sent;

                    // Medir latencia (ping al gateway)
                    let latency = measure_latency().unwrap_or(0.0);

                    NetworkMetrics {
                        download_mbps: download_mbps.max(0.0),
                        upload_mbps: upload_mbps.max(0.0),
                        latency_ms: latency,
                        packets_sent_per_sec: (pkts_sent_delta as f64 / interval_secs) as u64,
                        packets_recv_per_sec: (pkts_recv_delta as f64 / interval_secs) as u64,
                        errors_in: errors.0,
                        errors_out: errors.1,
                        drops_in: drops.0,
                        drops_out: drops.1,
                        timestamp: chrono::Local::now().to_rfc3339(),
                    }
                }
                Err(_) => NetworkMetrics::default(),
            };

            // Emitir evento al frontend
            let _ = app.emit("metrics_update", &metrics);

            std::thread::sleep(interval);
        }
    });

    Ok(())
}

/// Detener monitoreo de red
#[tauri::command]
pub async fn stop_monitoring() -> Result<(), String> {
    MONITORING_ACTIVE.store(false, Ordering::SeqCst);
    Ok(())
}

/// Obtener métricas actuales
#[tauri::command]
pub async fn get_current_metrics(adapter: String) -> Result<NetworkMetrics, String> {
    let (_bytes_recv, _bytes_sent, packets_recv, packets_sent, errors, drops) =
        get_adapter_metrics(&adapter).map_err(|e| e.to_string())?;

    let latency = measure_latency().unwrap_or(0.0);

    Ok(NetworkMetrics {
        download_mbps: 0.0, // Requiere medición delta
        upload_mbps: 0.0,
        latency_ms: latency,
        packets_sent_per_sec: packets_sent,
        packets_recv_per_sec: packets_recv,
        errors_in: errors.0,
        errors_out: errors.1,
        drops_in: drops.0,
        drops_out: drops.1,
        timestamp: chrono::Local::now().to_rfc3339(),
    })
}

/// Obtener métricas del adaptador usando PowerShell
fn get_adapter_metrics(
    adapter: &str,
) -> Result<(u64, u64, u64, u64, (u64, u64), (u64, u64)), Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        let read_stat = |name: &str| -> Result<u64, Box<dyn std::error::Error>> {
            let value =
                fs::read_to_string(format!("/sys/class/net/{}/statistics/{}", adapter, name))?;
            Ok(value.trim().parse().unwrap_or(0))
        };

        return Ok((
            read_stat("rx_bytes")?,
            read_stat("tx_bytes")?,
            read_stat("rx_packets")?,
            read_stat("tx_packets")?,
            (read_stat("rx_errors")?, read_stat("tx_errors")?),
            (read_stat("rx_dropped")?, read_stat("tx_dropped")?),
        ));
    }

    #[cfg(windows)]
    {
        let ps_script = format!(
            r#"
        $stats = Get-NetAdapterStatistics -Name "{}" -ErrorAction SilentlyContinue
        if ($stats) {{
            "$($stats.ReceivedBytes)|$($stats.SentBytes)|$($stats.ReceivedUnicastPackets)|$($stats.SentUnicastPackets)|$($stats.InErrors)|$($stats.OutErrors)|$($stats.InDiscards)|$($stats.OutDiscards)"
        }} else {{
            "0|0|0|0|0|0|0|0"
        }}
        "#,
            adapter
        );

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.trim().split('|').collect();

        if parts.len() >= 8 {
            Ok((
                parts[0].parse().unwrap_or(0),
                parts[1].parse().unwrap_or(0),
                parts[2].parse().unwrap_or(0),
                parts[3].parse().unwrap_or(0),
                (parts[4].parse().unwrap_or(0), parts[5].parse().unwrap_or(0)),
                (parts[6].parse().unwrap_or(0), parts[7].parse().unwrap_or(0)),
            ))
        } else {
            Ok((0, 0, 0, 0, (0, 0), (0, 0)))
        }
    }
}

/// Medir latencia al gateway
fn measure_latency() -> Result<f64, Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        let route_output = Command::new("ip")
            .args(["route", "show", "default"])
            .output()?;
        let routes = String::from_utf8_lossy(&route_output.stdout);
        let gateway = routes
            .lines()
            .find_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts
                    .windows(2)
                    .find(|window| window[0] == "via")
                    .map(|window| window[1].to_string())
            })
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "No default gateway found")
            })?;

        let ping_output = Command::new("ping")
            .args(["-n", "-c", "1", "-W", "1", &gateway])
            .output()?;

        let stdout = String::from_utf8_lossy(&ping_output.stdout);
        let latency = stdout
            .lines()
            .find_map(|line| line.split("time=").nth(1))
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);

        return Ok(latency);
    }

    #[cfg(windows)]
    {
        let ps_script = r#"
        $gateway = (Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Select-Object -First 1).NextHop
        if ($gateway) {
            $ping = Test-Connection -ComputerName $gateway -Count 1 -ErrorAction SilentlyContinue
            if ($ping) { $ping.ResponseTime } else { 0 }
        } else { 0 }
    "#;

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", ps_script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().parse().unwrap_or(0.0))
    }
}
