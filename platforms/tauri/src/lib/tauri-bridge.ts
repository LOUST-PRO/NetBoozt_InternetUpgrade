/**
 * Tauri Bridge - Wrapper para manejar llamadas Tauri en modo web
 * Proporciona datos simulados cuando no está disponible Tauri IPC
 */

// Tipo para la función de unlisten (compatible con Tauri)
export type UnlistenFn = () => void;

// Detectar si estamos en Tauri
export function isTauriAvailable(): boolean {
    return typeof window !== 'undefined' && 
           '__TAURI_INTERNALS__' in window;
}

// Mock data para modo web/desarrollo
const MOCK_ADAPTERS = [
    {
        name: 'Wi-Fi',
        description: 'Intel(R) Wi-Fi 6 AX201 160MHz',
        status: 'Up',
        link_speed: '866 Mbps',
        mac_address: 'AA:BB:CC:DD:EE:FF'
    },
    {
        name: 'Ethernet',
        description: 'Realtek PCIe GbE Family Controller',
        status: 'Up',
        link_speed: '1 Gbps',
        mac_address: '11:22:33:44:55:66'
    }
];

// Mock basado en configuración REAL del sistema (netsh int tcp show global)
const MOCK_TCP_SETTINGS = {
    // Básicos
    autotuning: 'experimental',  // Nivel de ajuste automático de ventana
    rss: 'enabled',              // Estado de escalado de lado de recepción
    rsc: 'enabled',              // Estado de fusión de segmento de recepción
    ecn: 'enabled',              // Funcionalidad de ECN
    timestamps: 'enabled',       // Marcas de hora RFC 1323
    chimney: 'disabled',         // TCP Chimney Offload (deprecado)
    
    // Avanzados (BBR-like)
    congestion_provider: 'CUBIC',  // Proveedor de control de congestión
    initial_rto: 1000,             // RTO inicial (ms) - reducido de 3000
    fast_open: 'enabled',          // TCP Fast Open
    fast_open_fallback: 'enabled', // Reserva Fast Open
    hystart: 'enabled',            // HyStart++ (slow-start mejorado)
    prr: 'enabled',                // Reducción de tasa proporcional (PRR)
    pacing: 'always',              // Perfil de velocidad (pacing)
    max_syn_retransmissions: 4,    // Número máximo de retransmisiones SYN
    rack: 'enabled',               // RACK (Recent ACK)
    tail_loss_probe: 'enabled',    // Sondeo de pérdida de cola
};

// Mock DNS basado en configuración real (Ethernet tiene fallback chain completo)
const MOCK_DNS_CONFIG = {
    servers: ['1.1.1.1', '8.8.8.8', '1.0.0.1', '8.8.4.4', '9.9.9.9', '208.67.222.222', '192.168.1.254'],
    is_dhcp: false
};

// DNS para Wi-Fi (más simple)
const MOCK_DNS_WIFI = {
    servers: ['8.8.8.8', '8.8.4.4'],
    is_dhcp: false
};

const MOCK_DIAGNOSTIC = {
    health: 'Good',
    score: 85,
    failure_point: 'none',
    adapter_ok: true,
    adapter_name: 'Wi-Fi',
    router_ok: true,
    router_latency_ms: 2.5,
    isp_ok: true,
    isp_latency_ms: 15.3,
    dns_ok: true,
    dns_latency_ms: 12.1,
    internet_ok: true,
    internet_latency_ms: 42.5,
    recommendation: 'Red funcionando correctamente'
};

type MockCommands = {
    get_network_adapters: typeof MOCK_ADAPTERS;
    get_current_settings: typeof MOCK_TCP_SETTINGS;
    run_full_diagnostic: typeof MOCK_DIAGNOSTIC;
    start_monitoring: null;
    stop_monitoring: null;
    set_dns_servers: null;
    reset_dns_to_dhcp: null;
    flush_dns_cache: null;
    apply_profile: string[];
    reset_to_defaults: string[];
};

// Invoke wrapper con mock data
export async function invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    if (isTauriAvailable()) {
        // Usar Tauri real
        const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
        return tauriInvoke<T>(cmd, args);
    }
    
    // Modo desarrollo/web - retornar mock data
    console.log(`[Mock] invoke('${cmd}')`, args);
    
    // Simular delay de red
    await new Promise(resolve => setTimeout(resolve, 300 + Math.random() * 200));
    
    // Para get_current_dns, usar el adaptador solicitado
    if (cmd === 'get_current_dns') {
        const adapter = (args?.adapter as string)?.toLowerCase() || '';
        if (adapter.includes('wifi') || adapter.includes('wi-fi')) {
            return MOCK_DNS_WIFI as T;
        }
        return MOCK_DNS_CONFIG as T; // Ethernet con fallback chain completo
    }
    
    const mockResponses: Record<string, unknown> = {
        get_network_adapters: MOCK_ADAPTERS,
        get_current_settings: MOCK_TCP_SETTINGS,
        run_full_diagnostic: MOCK_DIAGNOSTIC,
        start_monitoring: null,
        stop_monitoring: null,
        set_dns_servers: null,
        reset_dns_to_dhcp: null,
        flush_dns_cache: null,
        apply_profile: ['RSS', 'RSC', 'ECN', 'Autotuning', 'HyStart', 'PRR', 'FastOpen', 'Pacing'],
        reset_to_defaults: ['RSS', 'RSC', 'ECN', 'Autotuning', 'Timestamps', 'HyStart', 'PRR', 'FastOpen'],
        // Nuevos comandos de diagnóstico
        run_windows_network_troubleshooter: 'Asistente de red del sistema iniciado',
        reset_network_stack: 'Reset de red solicitado correctamente',
        measure_dns_resolution: { domain: 'google.com', success: true, latency_ms: 25, error: null },
        // Comandos DNS Intelligence
        get_dns_failover_history: [],
        start_dns_intel_service: 'DNS Intelligence service started',
        stop_dns_intel_service: 'DNS Intelligence service stopped',
        get_current_best_dns: '1.1.1.1'
    };
    
    if (cmd in mockResponses) {
        return mockResponses[cmd] as T;
    }
    
    console.warn(`[Mock] Unknown command: ${cmd}`);
    return null as T;
}

// Listen wrapper con mock events
export async function listen<T = unknown>(
    event: string, 
    handler: (event: { payload: T }) => void
): Promise<() => void> {
    if (isTauriAvailable()) {
        const { listen: tauriListen } = await import('@tauri-apps/api/event');
        return tauriListen<T>(event, handler);
    }
    
    console.log(`[Mock] listen('${event}')`);
    
    // Simular eventos periódicos en modo desarrollo
    if (event === 'metrics_update') {
        const intervalId = setInterval(() => {
            handler({
                payload: {
                    download_mbps: 50 + Math.random() * 100,
                    upload_mbps: 10 + Math.random() * 30,
                    latency_ms: 15 + Math.random() * 25,
                    packets_sent_per_sec: Math.floor(800 + Math.random() * 400),
                    packets_recv_per_sec: Math.floor(1200 + Math.random() * 600),
                    errors_in: Math.floor(Math.random() * 2),
                    errors_out: Math.floor(Math.random() * 2),
                    drops_in: Math.floor(Math.random() * 3),
                    drops_out: Math.floor(Math.random() * 3)
                } as unknown as T
            });
        }, 1000);
        
        return () => clearInterval(intervalId);
    }
    
    // Para otros eventos, no simular nada
    return () => {};
}
