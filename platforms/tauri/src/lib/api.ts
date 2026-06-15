/**
 * NetBoozt - Tauri API Bridge
 * Funciones para comunicarse con el backend Rust
 * 
 * By LOUST (www.loust.pro)
 */

import { invoke, listen, type UnlistenFn } from './tauri-bridge';
import type {
    NetworkAdapter,
    DiagnosticResult,
    TcpSettings,
    NetworkMetrics,
    DNSHealth,
    DNSConfig,
    Alert,
    FailoverEvent,
    SpeedTestResult,
    OptimizationProfile,
    AppConfig
} from './types';

// ============================================
// Network Commands
// ============================================

/**
 * Obtener lista de adaptadores de red activos
 */
export async function getNetworkAdapters(): Promise<NetworkAdapter[]> {
    return invoke<NetworkAdapter[]>('get_network_adapters');
}

/**
 * Obtener configuración DNS actual de un adaptador
 */
export async function getCurrentDNS(adapter: string): Promise<DNSConfig> {
    return invoke<DNSConfig>('get_current_dns', { adapter });
}

/**
 * Establecer servidores DNS
 */
export async function setDNSServers(
    adapter: string, 
    primary: string, 
    secondary?: string
): Promise<boolean> {
    return invoke<boolean>('set_dns_servers', { adapter, primary, secondary });
}

/**
 * Resetear DNS a DHCP
 */
export async function resetDNSToDHCP(adapter: string): Promise<boolean> {
    return invoke<boolean>('reset_dns_to_dhcp', { adapter });
}

/**
 * Limpiar caché DNS
 */
export async function flushDNSCache(): Promise<boolean> {
    return invoke<boolean>('flush_dns_cache');
}

// ============================================
// Diagnostic Commands
// ============================================

/**
 * Ejecutar diagnóstico completo de red (4 fases)
 */
export async function runFullDiagnostic(): Promise<DiagnosticResult> {
    return invoke<DiagnosticResult>('run_full_diagnostic');
}

/**
 * Quick check de conectividad
 */
export async function quickCheck(): Promise<[boolean, string]> {
    return invoke<[boolean, string]>('quick_check');
}

/**
 * Ping a un host específico
 */
export async function pingHost(host: string): Promise<number | null> {
    return invoke<number | null>('ping_host', { host });
}

/**
 * Verificar salud de un servidor DNS
 */
export async function checkDNSHealth(server: string): Promise<DNSHealth> {
    return invoke<DNSHealth>('check_dns_health', { server });
}

/**
 * Ejecutar el asistente de red del sistema
 */
export async function runWindowsNetworkTroubleshooter(): Promise<string> {
    return invoke<string>('run_windows_network_troubleshooter');
}

/**
 * Resetear la red del sistema con la mejor estrategia soportada por la plataforma
 */
export async function resetNetworkStack(): Promise<string> {
    return invoke<string>('reset_network_stack');
}

/**
 * Medir tiempo de resolución DNS para un dominio
 */
export async function measureDnsResolution(domain: string): Promise<{
    domain: string;
    success: boolean;
    latency_ms: number | null;
    error: string | null;
}> {
    return invoke('measure_dns_resolution', { domain });
}

// ============================================
// Optimizer Commands
// ============================================

/**
 * Obtener configuración TCP actual
 */
export async function getCurrentSettings(): Promise<TcpSettings> {
    return invoke<TcpSettings>('get_current_settings');
}

/**
 * Aplicar perfil de optimización
 */
export async function applyProfile(profile: OptimizationProfile): Promise<string[]> {
    return invoke<string[]>('apply_profile', { profile });
}

/**
 * Restaurar configuración por defecto
 */
export async function resetToDefaults(): Promise<string[]> {
    return invoke<string[]>('reset_to_defaults');
}

/**
 * Aplicar optimización individual
 */
export async function applyOptimization(optimizationId: string): Promise<boolean> {
    return invoke<boolean>('apply_optimization', { optimizationId });
}

// ============================================
// Monitoring Commands
// ============================================

/**
 * Iniciar monitoreo de red
 */
export async function startMonitoring(adapter: string, intervalMs: number = 1000): Promise<void> {
    return invoke('start_monitoring', { adapter, intervalMs });
}

/**
 * Detener monitoreo de red
 */
export async function stopMonitoring(): Promise<void> {
    return invoke('stop_monitoring');
}

/**
 * Obtener métricas actuales
 */
export async function getCurrentMetrics(): Promise<NetworkMetrics> {
    return invoke<NetworkMetrics>('get_current_metrics');
}

/**
 * Obtener historial de métricas
 */
export async function getMetricsHistory(seconds: number = 60): Promise<NetworkMetrics[]> {
    return invoke<NetworkMetrics[]>('get_metrics_history', { seconds });
}

// ============================================
// Failover Commands
// ============================================

/**
 * Habilitar auto-failover
 */
export async function enableAutoFailover(): Promise<void> {
    return invoke('enable_auto_failover');
}

/**
 * Deshabilitar auto-failover
 */
export async function disableAutoFailover(): Promise<void> {
    return invoke('disable_auto_failover');
}

/**
 * Obtener estado del failover
 */
export async function getFailoverStatus(): Promise<{
    enabled: boolean;
    current_tier: number;
    last_failover?: string;
}> {
    return invoke('get_failover_status');
}

/**
 * Forzar failover a un tier específico
 */
export async function forceFailover(tier: number): Promise<boolean> {
    return invoke<boolean>('force_failover', { tier });
}

/**
 * Obtener historial de failovers
 */
export async function getFailoverHistory(limit: number = 20): Promise<FailoverEvent[]> {
    return invoke<FailoverEvent[]>('get_failover_history', { limit });
}

/**
 * Obtener historial de failovers DNS del servicio de inteligencia
 */
export async function getDnsFailoverHistory(): Promise<FailoverEvent[]> {
    return invoke<FailoverEvent[]>('get_dns_failover_history');
}

/**
 * Iniciar el servicio de DNS Intelligence
 */
export async function startDnsIntelService(): Promise<string> {
    return invoke<string>('start_dns_intel_service');
}

/**
 * Detener el servicio de DNS Intelligence
 */
export async function stopDnsIntelService(): Promise<string> {
    return invoke<string>('stop_dns_intel_service');
}

/**
 * Obtener el mejor DNS actual según el servicio de inteligencia
 */
export async function getCurrentBestDns(): Promise<string | null> {
    return invoke<string | null>('get_current_best_dns');
}

// ============================================
// Speed Test Commands
// ============================================

/**
 * Ejecutar speed test
 */
export async function runSpeedTest(): Promise<SpeedTestResult> {
    return invoke<SpeedTestResult>('run_speed_test');
}

/**
 * Obtener historial de speed tests
 */
export async function getSpeedTestHistory(limit: number = 10): Promise<SpeedTestResult[]> {
    return invoke<SpeedTestResult[]>('get_speed_test_history', { limit });
}

// ============================================
// Settings Commands
// ============================================

/**
 * Obtener configuración de la app
 */
export async function getAppConfig(): Promise<AppConfig> {
    return invoke<AppConfig>('get_app_config');
}

/**
 * Guardar configuración de la app
 */
export async function saveAppConfig(config: Partial<AppConfig>): Promise<void> {
    return invoke('save_app_config', { config });
}

/**
 * Exportar configuración
 */
export async function exportConfig(path: string): Promise<void> {
    return invoke('export_config', { path });
}

/**
 * Importar configuración
 */
export async function importConfig(path: string): Promise<void> {
    return invoke('import_config', { path });
}

// ============================================
// Event Listeners
// ============================================

/**
 * Escuchar actualizaciones de métricas en tiempo real
 */
export function onMetricsUpdate(callback: (metrics: NetworkMetrics) => void): Promise<UnlistenFn> {
    return listen<NetworkMetrics>('metrics_update', (event) => {
        callback(event.payload);
    });
}

/**
 * Escuchar alertas
 */
export function onAlert(callback: (alert: Alert) => void): Promise<UnlistenFn> {
    return listen<Alert>('alert_triggered', (event) => {
        callback(event.payload);
    });
}

/**
 * Escuchar eventos de failover
 */
export function onFailover(callback: (event: FailoverEvent) => void): Promise<UnlistenFn> {
    return listen<FailoverEvent>('failover_triggered', (event) => {
        callback(event.payload);
    });
}

/**
 * Escuchar actualizaciones de salud DNS
 */
export function onDNSHealthUpdate(callback: (health: Record<string, DNSHealth>) => void): Promise<UnlistenFn> {
    return listen<Record<string, DNSHealth>>('dns_health_update', (event) => {
        callback(event.payload);
    });
}

/**
 * Escuchar progreso del speed test
 */
export function onSpeedTestProgress(callback: (progress: { stage: string; progress: number; speed?: number }) => void): Promise<UnlistenFn> {
    return listen<{ stage: string; progress: number; speed?: number }>('speed_test_progress', (event) => {
        callback(event.payload);
    });
}

// ============================================
// Utility Functions
// ============================================

/**
 * Verificar si se ejecuta como administrador
 */
export async function isAdmin(): Promise<boolean> {
    return invoke<boolean>('is_admin');
}

/**
 * Reiniciar como administrador
 */
export async function restartAsAdmin(): Promise<void> {
    return invoke('restart_as_admin');
}

/**
 * Abrir URL en navegador
 */
export async function openUrl(url: string): Promise<void> {
    return invoke('open_url', { url });
}

/**
 * Obtener versión de la app
 */
export async function getAppVersion(): Promise<string> {
    return invoke<string>('get_app_version');
}

/**
 * Verificar actualizaciones
 */
export async function checkForUpdates(): Promise<{ available: boolean; version?: string; url?: string }> {
    return invoke('check_for_updates');
}
