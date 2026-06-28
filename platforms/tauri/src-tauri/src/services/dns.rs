//! DNS Service Module
//!
//! Servicio de DNS centralizado para operaciones de red.
//! Usado por comandos y system tray.
//!
//! By LOUST (www.loust.pro)

use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(not(windows))]
fn run_command_capture(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Error ejecutando {}: {}", program, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(format!(
            "{} falló: {}",
            program,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(not(windows))]
fn is_virtual_linux_interface(device: &str, device_type: &str) -> bool {
    matches!(
        device_type,
        "loopback" | "bridge" | "wireguard" | "wifi-p2p"
    ) || matches!(
        device,
        name if name == "lo"
            || name.starts_with("docker")
            || name.starts_with("br-")
            || name.starts_with("veth")
            || name.starts_with("virbr")
            || name.starts_with("wg")
            || name.starts_with("tun")
            || name.starts_with("tap")
    )
}

#[cfg(not(windows))]
fn get_active_connection(adapter: &str) -> Result<String, String> {
    let connection = run_command_capture(
        "nmcli",
        &["-t", "-g", "GENERAL.CONNECTION", "device", "show", adapter],
    )?;

    let connection = connection.lines().next().unwrap_or("").trim();
    if connection.is_empty() || connection == "--" {
        Err(format!(
            "No hay una conexión activa de NetworkManager para {}",
            adapter
        ))
    } else {
        Ok(connection.to_string())
    }
}

#[cfg(not(windows))]
fn reapply_connection(adapter: &str, connection: &str) -> Result<(), String> {
    let reapply = Command::new("nmcli")
        .args(["device", "reapply", adapter])
        .output()
        .map_err(|e| format!("Error reaplicando {}: {}", adapter, e))?;

    if reapply.status.success() {
        return Ok(());
    }

    let reconnect = Command::new("nmcli")
        .args(["connection", "up", connection, "ifname", adapter])
        .output()
        .map_err(|e| format!("Error reactivando {}: {}", connection, e))?;

    if reconnect.status.success() {
        Ok(())
    } else {
        Err(format!(
            "No se pudo reaplicar la conexión {}: {}",
            connection,
            String::from_utf8_lossy(&reconnect.stderr).trim()
        ))
    }
}

#[cfg(not(windows))]
fn normalize_nm_bool(value: &str) -> bool {
    matches!(
        value.trim().to_lowercase().as_str(),
        "1" | "yes" | "true" | "on" | "sí" | "si"
    )
}

#[cfg(not(windows))]
fn parse_dns_server_list(raw: &str) -> Vec<String> {
    let values = raw.replace('\n', " ");
    let values = values
        .split_once(':')
        .map(|(_, rest)| rest)
        .unwrap_or(&values);

    values
        .split(|ch: char| ch == ',' || ch.is_whitespace())
        .map(|token| token.trim().replace("\\:", ":"))
        .filter(|token| !token.is_empty())
        .collect()
}

#[cfg(not(windows))]
pub fn get_current_dns_servers(adapter: &str) -> Result<Vec<String>, String> {
    if let Ok(raw) = run_command_capture("resolvectl", &["dns", adapter]) {
        let servers = parse_dns_server_list(&raw);
        if !servers.is_empty() {
            return Ok(servers);
        }
    }

    let raw = run_command_capture(
        "nmcli",
        &["-t", "-g", "IP4.DNS,IP6.DNS", "device", "show", adapter],
    )?;
    Ok(parse_dns_server_list(&raw))
}

#[cfg(not(windows))]
pub fn is_dns_dhcp(adapter: &str) -> Result<bool, String> {
    let connection = get_active_connection(adapter)?;
    let raw = run_command_capture(
        "nmcli",
        &[
            "-t",
            "-g",
            "ipv4.ignore-auto-dns,ipv4.dns,ipv6.ignore-auto-dns,ipv6.dns",
            "connection",
            "show",
            &connection,
        ],
    )?;

    let mut lines = raw.lines();
    let ipv4_ignore = normalize_nm_bool(lines.next().unwrap_or(""));
    let ipv4_dns = lines.next().unwrap_or("").trim();
    let ipv6_ignore = normalize_nm_bool(lines.next().unwrap_or(""));
    let ipv6_dns = lines.next().unwrap_or("").trim();

    Ok(!(ipv4_ignore || ipv6_ignore || !ipv4_dns.is_empty() || !ipv6_dns.is_empty()))
}

/// Proveedores DNS predefinidos
#[derive(Debug, Clone)]
pub struct DnsProvider {
    pub id: &'static str,
    pub name: &'static str,
    pub primary: &'static str,
    pub secondary: &'static str,
    #[allow(dead_code)]
    pub tier: u8,
}

/// Lista de proveedores DNS
pub const DNS_PROVIDERS: &[DnsProvider] = &[
    DnsProvider {
        id: "cloudflare",
        name: "Cloudflare",
        primary: "1.1.1.1",
        secondary: "1.0.0.1",
        tier: 1,
    },
    DnsProvider {
        id: "google",
        name: "Google",
        primary: "8.8.8.8",
        secondary: "8.8.4.4",
        tier: 2,
    },
    DnsProvider {
        id: "quad9",
        name: "Quad9",
        primary: "9.9.9.9",
        secondary: "149.112.112.112",
        tier: 3,
    },
    DnsProvider {
        id: "opendns",
        name: "OpenDNS",
        primary: "208.67.222.222",
        secondary: "208.67.220.220",
        tier: 4,
    },
    DnsProvider {
        id: "adguard",
        name: "AdGuard",
        primary: "94.140.14.14",
        secondary: "94.140.15.15",
        tier: 5,
    },
    DnsProvider {
        id: "cleanbrowsing",
        name: "CleanBrowsing",
        primary: "185.228.168.9",
        secondary: "185.228.169.9",
        tier: 6,
    },
];

/// Obtener proveedor DNS por ID
pub fn get_provider(id: &str) -> Option<&DnsProvider> {
    DNS_PROVIDERS.iter().find(|p| p.id == id)
}

/// Obtener adaptador de red principal (activo)
pub fn get_primary_adapter() -> Result<String, String> {
    #[cfg(not(windows))]
    {
        let raw = run_command_capture(
            "nmcli",
            &[
                "-t",
                "-f",
                "DEVICE,TYPE,STATE,CONNECTION",
                "device",
                "status",
            ],
        )?;

        for line in raw.lines() {
            let parts: Vec<&str> = line.splitn(4, ':').collect();
            if parts.len() < 3 {
                continue;
            }

            let device = parts[0].trim();
            let device_type = parts[1].trim();
            let state = parts[2].trim();

            if device.is_empty()
                || !state.starts_with("connected")
                || is_virtual_linux_interface(device, device_type)
            {
                continue;
            }

            return Ok(device.to_string());
        }

        return Err(
            "No se encontró un adaptador Linux activo administrado por NetworkManager".to_string(),
        );
    }

    #[cfg(windows)]
    {
        let ps_script = r#"
        Get-NetAdapter | Where-Object Status -eq 'Up' | 
        Select-Object -First 1 -ExpandProperty Name
    "#;

        run_powershell(ps_script)
    }
}

/// Cambiar DNS de un adaptador
pub fn set_dns(adapter: &str, primary: &str, secondary: Option<&str>) -> Result<bool, String> {
    #[cfg(not(windows))]
    {
        let connection = get_active_connection(adapter)?;
        let dns_list = match secondary {
            Some(sec) => format!("{} {}", primary, sec),
            None => primary.to_string(),
        };

        run_command_capture(
            "nmcli",
            &[
                "connection",
                "modify",
                &connection,
                "ipv4.ignore-auto-dns",
                "yes",
                "ipv4.dns",
                &dns_list,
                "ipv6.ignore-auto-dns",
                "yes",
                "ipv6.dns",
                "",
            ],
        )?;
        reapply_connection(adapter, &connection)?;
        return Ok(true);
    }

    #[cfg(windows)]
    {
        let dns_list = match secondary {
            Some(sec) => format!("{},{}", primary, sec),
            None => primary.to_string(),
        };

        let command = format!(
            "Set-DnsClientServerAddress -InterfaceAlias '{}' -ServerAddresses {}",
            adapter, dns_list
        );

        let result = run_powershell(&command)?;
        Ok(result.is_empty() || !result.contains("error"))
    }
}

/// Cambiar DNS por ID de proveedor
pub fn set_dns_by_provider(provider_id: &str) -> Result<bool, String> {
    let provider = get_provider(provider_id)
        .ok_or_else(|| format!("Proveedor DNS '{}' no encontrado", provider_id))?;

    let adapter = get_primary_adapter()?;

    set_dns(&adapter, provider.primary, Some(provider.secondary))
}

/// Resetear DNS a DHCP (automático)
pub fn reset_dns_to_dhcp(adapter: &str) -> Result<bool, String> {
    #[cfg(not(windows))]
    {
        let connection = get_active_connection(adapter)?;
        run_command_capture(
            "nmcli",
            &[
                "connection",
                "modify",
                &connection,
                "ipv4.ignore-auto-dns",
                "no",
                "ipv4.dns",
                "",
                "ipv6.ignore-auto-dns",
                "no",
                "ipv6.dns",
                "",
            ],
        )?;
        reapply_connection(adapter, &connection)?;
        return Ok(true);
    }

    #[cfg(windows)]
    {
        let command = format!(
            "Set-DnsClientServerAddress -InterfaceAlias '{}' -ResetServerAddresses",
            adapter
        );

        let result = run_powershell(&command)?;
        Ok(result.is_empty() || !result.contains("error"))
    }
}

/// Limpiar caché DNS
pub fn flush_dns_cache() -> Result<bool, String> {
    #[cfg(not(windows))]
    {
        if Command::new("resolvectl")
            .args(["flush-caches"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return Ok(true);
        }

        let output = Command::new("systemd-resolve")
            .args(["--flush-caches"])
            .output()
            .map_err(|e| format!("Error limpiando caché DNS: {}", e))?;

        return Ok(output.status.success());
    }

    #[cfg(windows)]
    {
        let result = run_powershell("Clear-DnsClientCache")?;
        Ok(result.is_empty() || !result.contains("error"))
    }
}

/// Seleccionar mejor DNS automáticamente basado en latencia
pub fn select_best_dns() -> Result<DnsProvider, String> {
    let mut best_provider: Option<&DnsProvider> = None;
    let mut best_latency = f64::MAX;

    for provider in DNS_PROVIDERS {
        if let Ok(latency) = ping_dns(provider.primary) {
            log::info!("DNS {} latency: {:.1}ms", provider.name, latency);
            if latency < best_latency && latency > 0.0 {
                best_latency = latency;
                best_provider = Some(provider);
            }
        }
    }

    best_provider
        .map(|p| p.clone())
        .ok_or_else(|| "No se pudo determinar el mejor DNS".to_string())
}

/// Verificar latencia de un servidor DNS
pub fn ping_dns(dns_server: &str) -> Result<f64, String> {
    #[cfg(not(windows))]
    {
        let output = Command::new("ping")
            .args(["-n", "-c", "1", "-W", "1", dns_server])
            .output()
            .map_err(|e| format!("Error ejecutando ping: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Ping falló: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let latency = stdout
            .lines()
            .find_map(|line| line.split("time=").nth(1))
            .and_then(|value| value.split_whitespace().next())
            .ok_or_else(|| "No se pudo extraer latencia del ping".to_string())?;

        return latency
            .parse::<f64>()
            .map_err(|e| format!("Error parsing latency: {}", e));
    }

    #[cfg(windows)]
    {
        let ps_script = format!(
            r#"
        $ping = Test-Connection -ComputerName {} -Count 1 -ErrorAction SilentlyContinue
        if ($ping) {{ $ping.ResponseTime }} else {{ -1 }}
        "#,
            dns_server
        );

        let result = run_powershell(&ps_script)?;
        result
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Error parsing latency: {}", e))
    }
}

/// Verificar si un DNS resuelve correctamente
#[allow(dead_code)]
pub fn check_dns_resolution(dns_server: &str, domain: &str) -> Result<bool, String> {
    #[cfg(not(windows))]
    {
        let output = Command::new("nslookup")
            .args([domain, dns_server])
            .output()
            .map_err(|e| format!("Error ejecutando nslookup: {}", e))?;

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        return Ok(stdout.contains("Address") && !stdout.contains("can't find"));
    }

    #[cfg(windows)]
    {
        let ps_script = format!(
            r#"
        try {{
            $result = Resolve-DnsName -Name {} -Server {} -DnsOnly -ErrorAction Stop
            if ($result) {{ "OK" }} else {{ "FAIL" }}
        }} catch {{
            "FAIL"
        }}
        "#,
            domain, dns_server
        );

        let result = run_powershell(&ps_script)?;
        Ok(result.trim() == "OK")
    }
}

/// Ejecutar comando PowerShell
#[cfg(windows)]
fn run_powershell(command: &str) -> Result<String, String> {
    #[cfg(windows)]
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", command])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Error ejecutando PowerShell: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("PowerShell error: {}", stderr))
    }
}
