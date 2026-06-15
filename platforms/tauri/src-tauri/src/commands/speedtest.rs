//! Speed Test Module
//!
//! Prueba de velocidad máxima de conexión.
//! Descarga/sube archivos de prueba para medir throughput real.
//!
//! By LOUST (www.loust.pro)

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// Servidores de prueba (CDNs con archivos de prueba conocidos)
const TEST_SERVERS: &[(&str, &str, usize)] = &[
    // (nombre, url, tamaño esperado en bytes)
    (
        "Cloudflare",
        "https://speed.cloudflare.com/__down?bytes=10000000",
        10_000_000,
    ),
    (
        "Fast.com (Netflix)",
        "https://api.fast.com/netflix/speedtest",
        0,
    ), // Dinámico
    (
        "Google",
        "https://www.google.com/images/branding/googlelogo/2x/googlelogo_color_272x92dp.png",
        13_504,
    ),
];

/// Resultado del speed test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub ping_ms: f64,
    pub jitter_ms: f64,
    pub server_name: String,
    pub server_location: String,
    pub timestamp: String,
    pub test_duration_secs: f64,
    pub bytes_downloaded: u64,
    pub bytes_uploaded: u64,
}

/// Progreso del speed test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestProgress {
    pub stage: String, // "ping", "download", "upload", "complete"
    pub progress: f64, // 0-100
    pub current_speed: f64,
    pub message: String,
}

/// Ejecutar speed test completo
#[tauri::command]
pub async fn run_speed_test(app: AppHandle) -> Result<SpeedTestResult, String> {
    log::info!("🚀 Iniciando Speed Test...");

    // Emitir progreso: Ping
    emit_progress(&app, "ping", 0.0, 0.0, "Midiendo latencia...");

    let ping_result = measure_ping().await;
    let (ping_ms, jitter_ms) = ping_result.unwrap_or((0.0, 0.0));

    emit_progress(&app, "ping", 100.0, 0.0, &format!("Ping: {:.0}ms", ping_ms));

    // Emitir progreso: Download
    emit_progress(
        &app,
        "download",
        0.0,
        0.0,
        "Iniciando prueba de descarga...",
    );

    let download_result = measure_download(&app).await;
    let (download_mbps, bytes_down, download_duration) = download_result.unwrap_or((0.0, 0, 0.0));

    emit_progress(
        &app,
        "download",
        100.0,
        download_mbps,
        &format!("Descarga: {:.2} Mbps", download_mbps),
    );

    // Emitir progreso: Upload
    emit_progress(&app, "upload", 0.0, 0.0, "Iniciando prueba de subida...");

    let upload_result = measure_upload(&app).await;
    let (upload_mbps, bytes_up, _upload_duration) = upload_result.unwrap_or((0.0, 0, 0.0));

    emit_progress(
        &app,
        "upload",
        100.0,
        upload_mbps,
        &format!("Subida: {:.2} Mbps", upload_mbps),
    );

    // Completado
    emit_progress(&app, "complete", 100.0, 0.0, "Speed Test completado");

    let result = SpeedTestResult {
        download_mbps,
        upload_mbps,
        ping_ms,
        jitter_ms,
        server_name: "Cloudflare".to_string(),
        server_location: "Global CDN".to_string(),
        timestamp: chrono::Local::now().to_rfc3339(),
        test_duration_secs: download_duration,
        bytes_downloaded: bytes_down,
        bytes_uploaded: bytes_up,
    };

    log::info!(
        "✅ Speed Test completado: ↓{:.2} Mbps ↑{:.2} Mbps",
        download_mbps,
        upload_mbps
    );

    Ok(result)
}

/// Medir latencia (ping) y jitter usando TCP connect time (más preciso)
async fn measure_ping() -> Result<(f64, f64), Box<dyn std::error::Error + Send + Sync>> {
    use std::net::ToSocketAddrs;

    let mut pings: Vec<f64> = Vec::new();

    // Servidores de baja latencia para medir
    let ping_targets = [
        ("1.1.1.1", 443), // Cloudflare
        ("8.8.8.8", 443), // Google
    ];

    // Hacer múltiples mediciones
    for _ in 0..3 {
        for (host, port) in ping_targets.iter() {
            let addr = format!("{}:{}", host, port);
            if let Ok(mut addrs) = addr.to_socket_addrs() {
                if let Some(socket_addr) = addrs.next() {
                    let start = Instant::now();

                    // TCP connect time es una buena aproximación al RTT
                    match tokio::time::timeout(
                        Duration::from_secs(2),
                        tokio::net::TcpStream::connect(socket_addr),
                    )
                    .await
                    {
                        Ok(Ok(_stream)) => {
                            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                            // El connect time es aproximadamente 1 RTT
                            pings.push(elapsed);
                        }
                        _ => continue,
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    if pings.is_empty() {
        // Fallback: usar HTTP HEAD con timeout corto
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()?;

        for _ in 0..3 {
            let start = Instant::now();
            if client.head("https://1.1.1.1").send().await.is_ok() {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                // Restar overhead de TLS handshake (~50-100ms típico)
                pings.push((elapsed - 50.0).max(1.0));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    if pings.is_empty() {
        return Ok((50.0, 5.0)); // Valores por defecto razonables
    }

    // Ordenar y quitar outliers (usar la mediana)
    pings.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_idx = pings.len() / 2;
    let avg_ping = pings[median_idx];

    // Calcular jitter como desviación entre pings consecutivos
    let jitter = if pings.len() > 1 {
        let mut diffs: Vec<f64> = Vec::new();
        for i in 1..pings.len() {
            diffs.push((pings[i] - pings[i - 1]).abs());
        }
        diffs.iter().sum::<f64>() / diffs.len() as f64
    } else {
        0.0
    };

    Ok((avg_ping, jitter))
}

/// Medir velocidad de descarga
async fn measure_download(
    app: &AppHandle,
) -> Result<(f64, u64, f64), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // Intentar con los servidores de TEST_SERVERS en orden
    let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

    for (server_name, url, expected_bytes) in TEST_SERVERS.iter() {
        // Saltar Fast.com (requiere API key)
        if *server_name == "Fast.com (Netflix)" {
            continue;
        }

        log::info!("📥 Probando descarga desde {}", server_name);

        let start = Instant::now();
        let mut total_bytes: u64 = 0;
        let target_bytes: u64 = if *expected_bytes > 0 {
            *expected_bytes as u64
        } else {
            10_000_000
        };

        match client.get(*url).send().await {
            Ok(response) => {
                let mut stream = response.bytes_stream();

                use futures_util::StreamExt;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(bytes) => {
                            total_bytes += bytes.len() as u64;

                            // Calcular progreso y velocidad actual
                            let elapsed = start.elapsed().as_secs_f64();
                            let progress =
                                (total_bytes as f64 / target_bytes as f64 * 100.0).min(100.0);
                            let current_speed = if elapsed > 0.0 {
                                (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0)
                            } else {
                                0.0
                            };

                            // Emitir progreso cada ~500KB
                            if total_bytes % 500_000 < 65536 {
                                emit_progress(
                                    app,
                                    "download",
                                    progress,
                                    current_speed,
                                    &format!(
                                        "Descargando desde {}... {:.2} Mbps",
                                        server_name, current_speed
                                    ),
                                );
                            }
                        }
                        Err(_) => break,
                    }
                }

                let elapsed = start.elapsed().as_secs_f64();
                let speed_mbps = if elapsed > 0.0 && total_bytes > 0 {
                    (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0)
                } else {
                    0.0
                };

                // Si obtuvimos datos, retornar
                if total_bytes > 1000 {
                    log::info!(
                        "✅ Descarga desde {} completada: {:.2} Mbps",
                        server_name,
                        speed_mbps
                    );
                    return Ok((speed_mbps, total_bytes, elapsed));
                }
            }
            Err(e) => {
                log::warn!("⚠️ Error descargando desde {}: {}", server_name, e);
                last_error = Some(Box::new(e));
            }
        }
    }

    // Si todos fallaron, retornar error o valores por defecto
    match last_error {
        Some(e) => Err(e),
        None => Ok((0.0, 0, 0.0)),
    }
}

/// Medir velocidad de subida (usando múltiples chunks para mejor precisión)
async fn measure_upload(
    app: &AppHandle,
) -> Result<(f64, u64, f64), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // Usar un tamaño más pequeño para upload (2MB) - más rápido y confiable
    let test_data = vec![b'X'; 2_000_000];
    let data_size = test_data.len() as u64;

    // Probar varios servidores para upload
    let upload_servers = [
        "https://speed.cloudflare.com/__up", // Cloudflare upload endpoint
        "https://httpbin.org/post",          // Fallback
    ];

    let mut best_speed = 0.0;
    let mut total_time = 0.0;

    for (idx, url) in upload_servers.iter().enumerate() {
        emit_progress(
            app,
            "upload",
            (idx as f64 / upload_servers.len() as f64) * 50.0,
            0.0,
            &format!("Probando servidor {}...", idx + 1),
        );

        let start = Instant::now();

        let result = tokio::time::timeout(
            Duration::from_secs(15),
            client
                .post(*url)
                .header("Content-Type", "application/octet-stream")
                .body(test_data.clone())
                .send(),
        )
        .await;

        let elapsed = start.elapsed().as_secs_f64();

        if let Ok(Ok(_response)) = result {
            let speed_mbps = if elapsed > 0.1 {
                (data_size as f64 * 8.0) / (elapsed * 1_000_000.0)
            } else {
                0.0
            };

            if speed_mbps > best_speed {
                best_speed = speed_mbps;
                total_time = elapsed;
            }

            log::debug!(
                "Upload test {}: {:.2} Mbps en {:.2}s",
                url,
                speed_mbps,
                elapsed
            );

            // Si obtuvimos un resultado decente, no probar más
            if speed_mbps > 5.0 {
                break;
            }
        }
    }

    emit_progress(
        app,
        "upload",
        100.0,
        best_speed,
        &format!("Subida: {:.2} Mbps", best_speed),
    );

    Ok((best_speed, data_size, total_time))
}

/// Emitir evento de progreso al frontend
fn emit_progress(app: &AppHandle, stage: &str, progress: f64, current_speed: f64, message: &str) {
    let progress_event = SpeedTestProgress {
        stage: stage.to_string(),
        progress,
        current_speed,
        message: message.to_string(),
    };
    let _ = app.emit("speedtest_progress", &progress_event);
}

/// Obtener último resultado guardado
#[tauri::command]
pub async fn get_last_speedtest() -> Result<Option<SpeedTestResult>, String> {
    // TODO: Implementar persistencia
    Ok(None)
}
