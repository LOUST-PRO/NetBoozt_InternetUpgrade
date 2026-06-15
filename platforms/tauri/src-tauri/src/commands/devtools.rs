//! DevTools Commands
//!
//! Comandos Tauri para las utilidades de desarrollo:
//! - HTTP requests (curl-like)
//! - Ping con múltiples paquetes
//! - Traceroute
//! - Port scanning
//! - Security headers check

use serde::Serialize;
use std::collections::HashMap;
use std::net::{TcpStream, ToSocketAddrs};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(not(windows))]
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
fn powershell_command() -> Command {
    let mut command = Command::new("powershell");
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

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

// ============================================
// HTTP REQUEST (cURL-like)
// ============================================

#[derive(Debug, Serialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time_ms: f64,
}

#[tauri::command]
pub async fn http_request(
    url: String,
    method: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<HttpResponse, String> {
    let start = Instant::now();
    let parsed_method = method
        .parse::<reqwest::Method>()
        .map_err(|e| format!("Método HTTP inválido: {}", e))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let mut request = client.request(parsed_method.clone(), &url);

    if let Some(hdrs) = headers {
        let mut header_map = reqwest::header::HeaderMap::new();
        for (key, value) in hdrs {
            let name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| format!("Header inválido {}: {}", key, e))?;
            let value = reqwest::header::HeaderValue::from_str(&value)
                .map_err(|e| format!("Valor inválido para header {}: {}", key, e))?;
            header_map.insert(name, value);
        }
        request = request.headers(header_map);
    }

    if parsed_method != reqwest::Method::GET && parsed_method != reqwest::Method::HEAD {
        if let Some(body) = body {
            request = request.body(body);
        }
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Error realizando request HTTP: {}", e))?;

    let status = response.status().as_u16();
    let mut headers_map = HashMap::new();
    for (key, value) in response.headers().iter() {
        headers_map.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
    }

    let body_content = response
        .text()
        .await
        .map_err(|e| format!("Error leyendo cuerpo HTTP: {}", e))?;

    Ok(HttpResponse {
        status,
        headers: headers_map,
        body: body_content,
        time_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

// ============================================
// PING CON MÚLTIPLES PAQUETES
// ============================================

#[derive(Debug, Serialize)]
pub struct PingPacket {
    pub seq: u32,
    pub ttl: u32,
    pub time: f64,
}

#[derive(Debug, Serialize)]
pub struct PingStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub loss: f64,
}

#[derive(Debug, Serialize)]
pub struct PingMultiResult {
    pub results: Vec<PingPacket>,
    pub stats: PingStats,
}

#[tauri::command]
pub async fn ping_multi(host: String, count: u32) -> Result<PingMultiResult, String> {
    #[cfg(not(windows))]
    {
        let count = count.max(1);
        let count_arg = count.to_string();

        let output = linux_command("ping")
            .args(["-n", "-c", count_arg.as_str(), host.as_str()])
            .output()
            .map_err(|e| format!("Error ejecutando ping: {}", e))?;

        if !output.status.success() && output.stdout.is_empty() {
            return Err(format!(
                "Ping falló: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();
        let mut loss = 100.0;
        let mut parsed_stats: Option<(f64, f64, f64)> = None;

        for line in stdout.lines() {
            if line.contains("icmp_seq=") && line.contains("time=") {
                let seq = line
                    .split("icmp_seq=")
                    .nth(1)
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|value| value.parse::<u32>().ok());
                let ttl = line
                    .split("ttl=")
                    .nth(1)
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|value| value.parse::<u32>().ok());
                let time = line
                    .split("time=")
                    .nth(1)
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|value| value.parse::<f64>().ok());

                if let (Some(seq), Some(ttl), Some(time)) = (seq, ttl, time) {
                    results.push(PingPacket { seq, ttl, time });
                }
            }

            if line.contains("packet loss") {
                if let Some(raw_loss) = line.split("% packet loss").next() {
                    if let Some(value) = raw_loss.split(',').last() {
                        loss = value.trim().parse::<f64>().unwrap_or(loss);
                    }
                }
            }

            if line.contains("min/avg/max") || line.contains("round-trip min/avg/max") {
                if let Some(metrics) = line.split('=').nth(1) {
                    let metrics = metrics.split_whitespace().next().unwrap_or("");
                    let mut values = metrics.split('/');
                    let min = values.next().and_then(|value| value.parse::<f64>().ok());
                    let avg = values.next().and_then(|value| value.parse::<f64>().ok());
                    let max = values.next().and_then(|value| value.parse::<f64>().ok());

                    if let (Some(min), Some(avg), Some(max)) = (min, avg, max) {
                        parsed_stats = Some((min, max, avg));
                    }
                }
            }
        }

        let computed_stats = if results.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let min = results.iter().map(|packet| packet.time).fold(f64::INFINITY, f64::min);
            let max = results.iter().map(|packet| packet.time).fold(f64::NEG_INFINITY, f64::max);
            let avg = results.iter().map(|packet| packet.time).sum::<f64>() / results.len() as f64;
            (min, max, avg)
        };

        let (min, max, avg) = parsed_stats.unwrap_or(computed_stats);

        return Ok(PingMultiResult {
            results,
            stats: PingStats {
                min,
                max,
                avg,
                loss,
            },
        });
    }

    #[cfg(windows)]
    {
    let ps_script = format!(
        r#"
        $results = @()
        $host_target = '{}'
        $count = {}
        $success = 0
        $times = @()
        
        for ($i = 1; $i -le $count; $i++) {{
            try {{
                $ping = Test-Connection -ComputerName $host_target -Count 1 -ErrorAction Stop
                $results += @{{
                    seq = $i
                    ttl = $ping.ResponseTimeToLive
                    time = $ping.ResponseTime
                }}
                $times += $ping.ResponseTime
                $success++
            }} catch {{
                $results += @{{
                    seq = $i
                    ttl = 0
                    time = -1
                }}
            }}
        }}
        
        $stats = @{{
            min = if ($times.Count -gt 0) {{ ($times | Measure-Object -Minimum).Minimum }} else {{ 0 }}
            max = if ($times.Count -gt 0) {{ ($times | Measure-Object -Maximum).Maximum }} else {{ 0 }}
            avg = if ($times.Count -gt 0) {{ ($times | Measure-Object -Average).Average }} else {{ 0 }}
            loss = [math]::Round((($count - $success) / $count) * 100, 1)
        }}
        
        @{{
            results = $results
            stats = $stats
        }} | ConvertTo-Json -Depth 3
        "#,
        host, count
    );

    let output = powershell_command()
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .output()
        .map_err(|e| format!("Error ejecutando ping: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let mut results = Vec::new();

        if let Some(res_array) = json.get("results").and_then(|v| v.as_array()) {
            for item in res_array {
                let seq = item.get("seq").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let ttl = item.get("ttl").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let time = item.get("time").and_then(|v| v.as_f64()).unwrap_or(-1.0);

                if time >= 0.0 {
                    results.push(PingPacket { seq, ttl, time });
                }
            }
        }

        let stats = if let Some(s) = json.get("stats") {
            PingStats {
                min: s.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0),
                max: s.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0),
                avg: s.get("avg").and_then(|v| v.as_f64()).unwrap_or(0.0),
                loss: s.get("loss").and_then(|v| v.as_f64()).unwrap_or(0.0),
            }
        } else {
            PingStats {
                min: 0.0,
                max: 0.0,
                avg: 0.0,
                loss: 100.0,
            }
        };

        Ok(PingMultiResult { results, stats })
    } else {
        Err("Error parseando resultados de ping".to_string())
    }
    }
}

// ============================================
// TRACEROUTE
// ============================================

#[derive(Debug, Serialize)]
pub struct TraceHop {
    pub hop: u32,
    pub ip: String,
    pub hostname: Option<String>,
    pub time: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct TracerouteResult {
    pub hops: Vec<TraceHop>,
}

#[tauri::command]
pub async fn traceroute(host: String, max_hops: Option<u32>) -> Result<TracerouteResult, String> {
    let max = max_hops.unwrap_or(30);

    #[cfg(not(windows))]
    {
        let max_arg = max.to_string();

        let output = if linux_find_program("traceroute").is_some() {
            linux_command("traceroute")
                .args(["-n", "-m", max_arg.as_str(), "-w", "1", host.as_str()])
                .output()
        } else {
            return Err("No se encontró traceroute en este Linux".to_string());
        }
        .map_err(|e| format!("Error ejecutando traceroute: {}", e))?;

        if !output.status.success() && output.stdout.is_empty() {
            return Err(format!(
                "Traceroute falló: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut hops = Vec::new();

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            let hop = match tokens.first().and_then(|value| value.parse::<u32>().ok()) {
                Some(hop) => hop,
                None => continue,
            };

            if tokens.get(1) == Some(&"*") {
                hops.push(TraceHop {
                    hop,
                    ip: "*".to_string(),
                    hostname: None,
                    time: vec![],
                });
                continue;
            }

            let ip = tokens.get(1).unwrap_or(&"*").to_string();
            let mut times = Vec::new();

            for window in tokens.windows(2) {
                if window[1] == "ms" {
                    if let Ok(time) = window[0].trim_start_matches('<').parse::<f64>() {
                        times.push(time);
                    }
                }
            }

            hops.push(TraceHop {
                hop,
                ip,
                hostname: None,
                time: times,
            });
        }

        return Ok(TracerouteResult { hops });
    }

    #[cfg(windows)]
    {

    let ps_script = format!(
        r#"
        $results = @()
        $output = tracert -h {} -w 1000 {} 2>&1
        
        $hopNum = 0
        foreach ($line in $output) {{
            if ($line -match '^\s*(\d+)\s+') {{
                $hopNum = [int]$Matches[1]
                $times = @()
                
                # Extraer tiempos (pueden ser <1 ms, X ms, o *)
                $timeMatches = [regex]::Matches($line, '(<?\d+)\s*ms|(\*)')
                foreach ($match in $timeMatches) {{
                    if ($match.Groups[1].Success) {{
                        $times += [double]$match.Groups[1].Value
                    }} elseif ($match.Groups[2].Success) {{
                        $times += -1
                    }}
                }}
                
                # Extraer IP y hostname
                $ip = ''
                $hostname = $null
                
                if ($line -match '\[([^\]]+)\]') {{
                    $ip = $Matches[1]
                    if ($line -match '(\S+)\s+\[') {{
                        $hostname = $Matches[1]
                    }}
                }} elseif ($line -match '(\d+\.\d+\.\d+\.\d+)') {{
                    $ip = $Matches[1]
                }}
                
                if ($ip -or $times.Count -gt 0) {{
                    $results += @{{
                        hop = $hopNum
                        ip = if ($ip) {{ $ip }} else {{ '*' }}
                        hostname = $hostname
                        time = $times | Where-Object {{ $_ -ge 0 }}
                    }}
                }}
            }}
        }}
        
        @{{ hops = $results }} | ConvertTo-Json -Depth 3
        "#,
        max, host
    );

    let output = powershell_command()
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .output()
        .map_err(|e| format!("Error ejecutando traceroute: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let mut hops = Vec::new();

        if let Some(hops_array) = json.get("hops").and_then(|v| v.as_array()) {
            for item in hops_array {
                let hop = item.get("hop").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let ip = item
                    .get("ip")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*")
                    .to_string();
                let hostname = item
                    .get("hostname")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let mut times = Vec::new();
                if let Some(time_array) = item.get("time").and_then(|v| v.as_array()) {
                    for t in time_array {
                        if let Some(time_val) = t.as_f64() {
                            times.push(time_val);
                        }
                    }
                }

                hops.push(TraceHop {
                    hop,
                    ip,
                    hostname,
                    time: times,
                });
            }
        }

        Ok(TracerouteResult { hops })
    } else {
        Err("Error parseando resultados de traceroute".to_string())
    }
    }
}

// ============================================
// PORT SCANNER
// ============================================

#[derive(Debug, Serialize)]
pub struct PortResult {
    pub port: u16,
    pub status: String, // "open", "closed", "filtered"
}

#[derive(Debug, Serialize)]
pub struct ScanPortsResult {
    pub results: Vec<PortResult>,
}

#[tauri::command]
pub async fn scan_ports(host: String, ports: Vec<u16>) -> Result<ScanPortsResult, String> {
    let mut results = Vec::new();

    for port in ports {
        let addr = format!("{}:{}", host, port);
        let mut status = "closed".to_string();

        if let Ok(addresses) = addr.to_socket_addrs() {
            for socket in addresses {
                match TcpStream::connect_timeout(&socket, Duration::from_millis(1000)) {
                    Ok(_) => {
                        status = "open".to_string();
                        break;
                    }
                    Err(error) => {
                        status = match error.kind() {
                            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
                                "filtered".to_string()
                            }
                            _ => "closed".to_string(),
                        };
                    }
                }
            }
        }

        results.push(PortResult { port, status });
    }

    Ok(ScanPortsResult { results })
}

// ============================================
// GET HEADERS (Security Headers Check)
// ============================================

#[derive(Debug, Serialize)]
pub struct GetHeadersResult {
    pub headers: HashMap<String, String>,
}

#[tauri::command]
pub async fn get_headers(url: String) -> Result<GetHeadersResult, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let head_response = client.head(&url).send().await;
    let response = match head_response {
        Ok(response) if response.status().as_u16() != 405 => response,
        _ => client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Error obteniendo headers: {}", e))?,
    };

    let mut headers = HashMap::new();
    for (key, value) in response.headers().iter() {
        headers.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
    }

    Ok(GetHeadersResult { headers })
}
