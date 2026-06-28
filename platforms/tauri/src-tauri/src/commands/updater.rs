//! Updater commands
//!
//! Comandos para verificar, descargar e instalar actualizaciones.

#[cfg(not(windows))]
use std::fs;
#[cfg(not(windows))]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager};

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
        let candidate = PathBuf::from(prefix).join(program);
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

/// Descargar actualización a la carpeta de descargas
#[tauri::command]
pub async fn download_update(
    app: AppHandle,
    url: String,
    filename: String,
) -> Result<String, String> {
    log::info!("Descargando actualización desde: {}", url);

    // Obtener carpeta de descargas
    let download_path = app
        .path()
        .download_dir()
        .map_err(|e| format!("No se pudo obtener la carpeta de descargas: {}", e))?
        .join(&filename);

    // Descargar archivo usando reqwest
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Error descargando: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Error leyendo respuesta: {}", e))?;

    // Guardar archivo
    std::fs::write(&download_path, bytes).map_err(|e| format!("Error guardando archivo: {}", e))?;

    log::info!("Actualización descargada en: {:?}", download_path);

    Ok(download_path.to_string_lossy().to_string())
}

/// Instalar actualización (ejecutar instalador)
#[tauri::command]
pub async fn install_update(path: String) -> Result<(), String> {
    log::info!("Instalando actualización: {}", path);

    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err("El archivo de instalación no existe".to_string());
    }

    #[cfg(windows)]
    {
        use std::process::Command;

        // Ejecutar instalador con elevación
        let result = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Start-Process '{}' -Verb RunAs", path.display()),
            ])
            .spawn();

        match result {
            Ok(_) => {
                log::info!("Instalador ejecutado con éxito");
                // Opcional: cerrar la app actual después de un delay
                Ok(())
            }
            Err(e) => {
                log::error!("Error ejecutando instalador: {}", e);
                Err(format!("Error ejecutando instalador: {}", e))
            }
        }
    }

    #[cfg(not(windows))]
    {
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_lowercase();

        if filename.ends_with(".appimage") {
            let metadata =
                fs::metadata(&path).map_err(|e| format!("No se pudo leer el AppImage: {}", e))?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(permissions.mode() | 0o755);
            fs::set_permissions(&path, permissions)
                .map_err(|e| format!("No se pudo marcar el AppImage como ejecutable: {}", e))?;

            Command::new(&path)
                .spawn()
                .map_err(|e| format!("Error ejecutando AppImage: {}", e))?;

            log::info!("AppImage lanzado correctamente");
            return Ok(());
        }

        let path_string = path.to_string_lossy().to_string();

        if linux_find_program("xdg-open").is_some() {
            linux_command("xdg-open")
                .arg(path_string.as_str())
                .spawn()
                .map_err(|e| format!("Error abriendo instalador: {}", e))?;
            return Ok(());
        }

        if linux_find_program("gio").is_some() {
            linux_command("gio")
                .args(["open", path_string.as_str()])
                .spawn()
                .map_err(|e| format!("Error abriendo instalador con gio: {}", e))?;
            return Ok(());
        }

        Err(
            "No se encontró un abridor gráfico compatible para instalar la actualización en Linux"
                .to_string(),
        )
    }
}

/// Obtener ruta de descargas
#[tauri::command]
pub fn get_downloads_path(app: AppHandle) -> Result<String, String> {
    app.path()
        .download_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("No se pudo obtener la carpeta de descargas: {}", e))
}

/// Verificar si existe un archivo
#[tauri::command]
pub fn file_exists(path: String) -> bool {
    PathBuf::from(path).exists()
}
