//! Notifications Service Module
//!
//! Servicio de notificaciones del sistema.
//!
//! By LOUST (www.loust.pro)

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Mostrar notificación del sistema
pub fn show_notification(app: &AppHandle, title: &str, body: &str) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| format!("Error mostrando notificación: {}", e))
}

/// Notificación de cambio de DNS
pub fn notify_dns_changed(app: &AppHandle, provider: &str) -> Result<(), String> {
    show_notification(
        app,
        "NetBoozt - DNS Cambiado",
        &format!("DNS configurado a {} correctamente", provider),
    )
}

/// Notificación de failover
#[allow(dead_code)]
pub fn notify_failover(app: &AppHandle, from_tier: u8, to_tier: u8) -> Result<(), String> {
    show_notification(
        app,
        "NetBoozt - Auto-Failover",
        &format!(
            "DNS cambiado de Tier {} a Tier {} automáticamente",
            from_tier, to_tier
        ),
    )
}

/// Notificación de error de conexión
#[allow(dead_code)]
pub fn notify_connection_error(app: &AppHandle, message: &str) -> Result<(), String> {
    show_notification(app, "NetBoozt - Error de Conexión", message)
}

/// Notificación de diagnóstico completado
pub fn notify_diagnostic_complete(app: &AppHandle, health: &str) -> Result<(), String> {
    show_notification(
        app,
        "NetBoozt - Diagnóstico Completo",
        &format!("Estado de la conexión: {}", health),
    )
}

/// Notificación de optimización aplicada
pub fn notify_optimization_applied(
    app: &AppHandle,
    profile: &str,
    count: usize,
) -> Result<(), String> {
    show_notification(
        app,
        "NetBoozt - Optimización Aplicada",
        &format!("Perfil {} aplicado ({} optimizaciones)", profile, count),
    )
}
