//! NetBoozt Tauri Backend
//!
//! Backend Rust para NetBoozt v3.0
//! Maneja todas las operaciones de red y sistema.
//!
//! By LOUST (www.loust.pro)

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// Módulos
mod commands;
mod services;
mod tray;

#[cfg(not(windows))]
use std::path::{Path, PathBuf};
use tauri::{Manager, WindowEvent};

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
fn detect_repo_root() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.clone());
        if let Some(parent) = current_dir.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        let mut cursor = exe_path.parent();
        while let Some(dir) = cursor {
            candidates.push(dir.to_path_buf());
            cursor = dir.parent();
        }
    }

    candidates.into_iter().find(|candidate| {
        candidate.join("platforms").join("tauri").exists()
            || candidate.join("windows").join("netboozt_cli.py").exists()
    })
}

#[cfg(not(windows))]
fn linux_open_terminal_in(directory: &Path) -> Result<(), String> {
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
        if let Some(path) = linux_find_program(terminal) {
            std::process::Command::new(path)
                .current_dir(directory)
                .spawn()
                .map_err(|e| format!("No se pudo abrir {}: {}", terminal, e))?;
            return Ok(());
        }
    }

    Err("No se encontró un emulador de terminal compatible".to_string())
}

fn main() {
    // Inicializar logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("🚀 Iniciando NetBoozt v3.0...");

    // Iniciar DNS Intelligence en segundo plano
    services::start_dns_intelligence();
    log::info!("🧠 DNS Intelligence service started");

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .on_menu_event(tray::handle_menu_event)
        .on_tray_icon_event(tray::handle_tray_icon_event)
        // Eventos de ventana
        .on_window_event(|window, event| {
            // Minimizar a tray en vez de cerrar
            if let WindowEvent::CloseRequested { api, .. } = event {
                log::info!("Ventana cerrada, minimizando a tray...");
                let _ = window.hide();
                api.prevent_close();
            }
        })
        // Setup inicial
        .setup(|app| {
            log::info!("Configurando aplicación...");

            let app_handle = app.handle().clone();
            tray::create_system_tray(&app_handle)?;

            // Obtener ventana principal
            let window = app
                .get_webview_window("main")
                .expect("No se encontró ventana principal");

            // En desarrollo, mostrar devtools
            #[cfg(debug_assertions)]
            {
                window.open_devtools();
            }

            // Centrar ventana
            let _ = window.center();

            log::info!("✅ NetBoozt inicializado correctamente");
            Ok(())
        })
        // Comandos Tauri
        .invoke_handler(tauri::generate_handler![
            // Network commands
            commands::network::get_network_adapters,
            commands::network::get_current_dns,
            commands::network::set_dns_servers,
            commands::network::reset_dns_to_dhcp,
            commands::network::flush_dns_cache,
            // DNS Intelligence commands
            commands::network::get_dns_ranking,
            commands::network::get_best_dns,
            commands::network::get_dns_intel_summary,
            commands::network::force_dns_check,
            commands::network::set_dns_auto_failover,
            commands::network::apply_best_dns,
            commands::network::get_dns_failover_history,
            commands::network::start_dns_intel_service,
            commands::network::stop_dns_intel_service,
            commands::network::get_current_best_dns,
            commands::network::open_device_manager,
            // Diagnostics commands
            commands::diagnostics::run_full_diagnostic,
            commands::diagnostics::quick_check,
            commands::diagnostics::ping_host,
            commands::diagnostics::check_dns_health,
            commands::diagnostics::check_single_dns_health,
            commands::diagnostics::run_windows_network_troubleshooter,
            commands::diagnostics::open_system_tool,
            commands::diagnostics::reset_network_stack,
            commands::diagnostics::measure_dns_resolution,
            // Optimizer commands
            commands::optimizer::get_current_settings,
            commands::optimizer::apply_profile,
            commands::optimizer::reset_to_defaults,
            commands::optimizer::get_available_optimizations,
            // Monitoring commands
            commands::monitoring::start_monitoring,
            commands::monitoring::stop_monitoring,
            commands::monitoring::get_current_metrics,
            // Speed test commands
            commands::speedtest::run_speed_test,
            commands::speedtest::get_last_speedtest,
            // DevTools commands
            commands::devtools::http_request,
            commands::devtools::ping_multi,
            commands::devtools::traceroute,
            commands::devtools::scan_ports,
            commands::devtools::get_headers,
            // Utility commands
            is_admin,
            get_app_version,
            open_cli_manager,
            open_url,
            // Updater commands
            commands::updater::download_update,
            commands::updater::install_update,
            commands::updater::get_downloads_path,
            commands::updater::file_exists,
        ])
        .run(tauri::generate_context!())
        .expect("Error al iniciar NetBoozt");
}

// ============================================
// Utility Commands
// ============================================

/// Verificar si se ejecuta como administrador
#[tauri::command]
fn is_admin() -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)"
            ])
            .output();

        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim() == "True",
            Err(_) => false,
        }
    }

    #[cfg(not(windows))]
    {
        use std::process::Command;

        Command::new("id")
            .arg("-u")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "0")
            .unwrap_or(false)
    }
}

/// Obtener versión de la aplicación
#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Abrir CLI Manager en ventana externa con elevación
#[tauri::command]
async fn open_cli_manager() -> Result<String, String> {
    #[cfg(windows)]
    {
        use std::env;
        use std::process::Command;

        // Obtener ruta del proyecto (buscar desde el ejecutable hacia arriba)
        let exe_path = env::current_exe().map_err(|e| e.to_string())?;
        let project_root = exe_path
            .parent() // src-tauri/target/debug o release
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from(r"L:\NetworkFailover\NetBoozt"));

        let cli_path = project_root.join("windows").join("netboozt_cli.py");
        let cli_dir = project_root.join("windows");

        // Verificar que el CLI existe
        if !cli_path.exists() {
            // Intentar ruta absoluta como fallback
            let fallback =
                std::path::PathBuf::from(r"L:\NetworkFailover\NetBoozt\windows\netboozt_cli.py");
            if !fallback.exists() {
                return Err("CLI Manager no encontrado. Verifica la instalación.".to_string());
            }
        }

        // Abrir PowerShell con elevación ejecutando el CLI
        let ps_command = format!(
            "Start-Process powershell -ArgumentList '-NoExit', '-Command', 'cd \"{}\"; python netboozt_cli.py' -Verb RunAs",
            cli_dir.display()
        );

        log::info!("Opening CLI Manager: {}", ps_command);

        let result = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_command])
            .spawn();

        match result {
            Ok(_) => {
                log::info!("CLI Manager abierto en nueva ventana");
                Ok("CLI Manager abierto correctamente. Se abrirá una ventana de PowerShell con permisos de administrador.".to_string())
            }
            Err(e) => {
                log::error!("Error abriendo CLI Manager: {}", e);
                Err(format!(
                    "Error: {}. Asegúrate de tener Python instalado.",
                    e
                ))
            }
        }
    }

    #[cfg(not(windows))]
    {
        let terminal_dir = detect_repo_root()
            .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
            .unwrap_or_else(std::env::temp_dir);

        linux_open_terminal_in(&terminal_dir)?;

        Ok(format!(
            "Se abrió una terminal en {}. El CLI legado sigue siendo Windows-only; usa esta terminal para comandos Linux de NetBoozt.",
            terminal_dir.display()
        ))
    }
}

/// Abrir URL en navegador
#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(["/c", "start", "", &url])
            .spawn()
            .map_err(|e| format!("Error abriendo URL: {}", e))?;
    }

    #[cfg(not(windows))]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Error abriendo URL: {}", e))?;
    }

    Ok(())
}
