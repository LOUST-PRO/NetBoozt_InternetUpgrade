<!--
    DashboardTab.svelte - Tab modular de Dashboard
    By LOUST
-->
<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { invoke, isTauriAvailable } from '$lib/tauri-bridge';
    import StatCard from '../StatCard.svelte';
    import DiagnosticChain from '../DiagnosticChain.svelte';
    import DiagnosticHistory from '../DiagnosticHistory.svelte';
    import AlgorithmChart from '../AlgorithmChart.svelte';
    import MiniRealtimeChart from '../MiniRealtimeChart.svelte';
    import SkeletonCard from '../SkeletonCard.svelte';
    import SpeedTestCard from '../SpeedTestCard.svelte';
    import OptimizationProfiles from '../OptimizationProfiles.svelte';
    import DNSManager from '../DNSManager.svelte';
    import Icon from '../Icon.svelte';
    import { diagnosticHistory, calculateHealthScore, type DiagnosticReport } from '$lib/stores/diagnosticStore';
    import type { TcpSettings, DNSConfig } from '$lib/types';
    
    export let loading = false;
    export let adapters: any[] = [];
    export let selectedAdapter = '';
    export let diagnostic: any = null;
    export let diagnosing = false;
    export let downloadRate = 0;
    export let uploadRate = 0;
    export let latency = 0;
    export let uptime = 0;
    
    // Métricas adicionales de red (antes en MonitorTab)
    export let packetsSent = 0;
    export let packetsRecv = 0;
    export let totalErrors = 0;
    export let totalDrops = 0;
    export let monitoringActive = false;
    
    const dispatch = createEventDispatcher();
    
    // Detectar si estamos en Tauri o web
    let isTauri = false;
    let platform: 'windows' | 'linux' | 'other' = 'other';
    
    // Datos históricos para gráficas en tiempo real (últimos 30 segundos)
    let downloadHistory: number[] = [];
    let uploadHistory: number[] = [];
    let latencyHistory: number[] = [];
    const MAX_HISTORY_POINTS = 30;
    
    // Actualizar historial cuando cambian los valores
    $: {
        if (downloadRate !== undefined) {
            downloadHistory = [...downloadHistory, downloadRate].slice(-MAX_HISTORY_POINTS);
        }
    }
    $: {
        if (uploadRate !== undefined) {
            uploadHistory = [...uploadHistory, uploadRate].slice(-MAX_HISTORY_POINTS);
        }
    }
    $: {
        if (latency !== undefined && latency > 0) {
            latencyHistory = [...latencyHistory, latency].slice(-MAX_HISTORY_POINTS);
        }
    }
    
    // Estado del protocolo y DNS
    let tcpSettings: TcpSettings | null = null;
    let dnsConfig: DNSConfig | null = null;
    let detectedProfile: 'Conservative' | 'Balanced' | 'Aggressive' | 'Unoptimized' | 'Custom' = 'Unoptimized';
    let loadingProtocol = true;
    let protocolError: string | null = null;
    let dnsError: string | null = null;
    
    // Speed Test result
    let speedTestResult: { download_mbps: number; upload_mbps: number; ping_ms: number } | null = null;
    
    // Optimization state
    let optimizing = false;
    let dryRunMode = false;
    
    // DNS state
    let autoFailoverEnabled = true; // Habilitado por defecto para mejor experiencia
    
    // Tooltip activo
    let activeTooltip: string | null = null;
    
    // Mapeo de DNS conocidos
    const DNS_PROVIDERS: Record<string, { name: string; icon: string; tier: number }> = {
        '1.1.1.1': { name: 'Cloudflare', icon: '☁️', tier: 1 },
        '1.0.0.1': { name: 'Cloudflare', icon: '☁️', tier: 1 },
        '8.8.8.8': { name: 'Google', icon: '🔍', tier: 2 },
        '8.8.4.4': { name: 'Google', icon: '🔍', tier: 2 },
        '9.9.9.9': { name: 'Quad9', icon: '🛡️', tier: 3 },
        '149.112.112.112': { name: 'Quad9', icon: '🛡️', tier: 3 },
        '208.67.222.222': { name: 'OpenDNS', icon: '🔐', tier: 4 },
        '208.67.220.220': { name: 'OpenDNS', icon: '🔐', tier: 4 },
        '94.140.14.14': { name: 'AdGuard', icon: '🚫', tier: 5 },
        '94.140.15.15': { name: 'AdGuard', icon: '🚫', tier: 5 },
        '185.228.168.9': { name: 'CleanBrowsing', icon: '🧹', tier: 6 },
    };
    
    // Información COMPLETA de optimizaciones con tooltips detallados
    const TCP_INFO: Record<string, { 
        label: string; 
        docPath: string; 
        shortDesc: string;
        fullDesc: string;
        impact: 'high' | 'medium' | 'low';
        profiles: string[]; // En qué perfiles está incluido
    }> = {
        // Configuración Base
        rss: { 
            label: 'RSS', 
            docPath: 'tcp-core', 
            shortDesc: 'Multi-CPU para paquetes',
            fullDesc: 'Receive Side Scaling distribuye el procesamiento de paquetes entre múltiples núcleos de CPU. Mejora el rendimiento en conexiones de alta velocidad.',
            impact: 'high',
            profiles: ['Conservative', 'Balanced', 'Aggressive']
        },
        rsc: { 
            label: 'RSC', 
            docPath: 'tcp-core', 
            shortDesc: 'Combina segmentos TCP',
            fullDesc: 'Receive Segment Coalescing agrupa múltiples segmentos TCP pequeños en uno más grande, reduciendo interrupciones de CPU.',
            impact: 'medium',
            profiles: ['Conservative', 'Balanced', 'Aggressive']
        },
        autotuning: { 
            label: 'Autotuning', 
            docPath: 'tcp-core', 
            shortDesc: 'Buffers dinámicos TCP',
            fullDesc: 'Ajusta automáticamente el tamaño del buffer de recepción. "normal" es seguro, "experimental" permite buffers más grandes para conexiones rápidas.',
            impact: 'high',
            profiles: ['Conservative', 'Balanced', 'Aggressive']
        },
        ecn: { 
            label: 'ECN', 
            docPath: 'tcp-congestion-control', 
            shortDesc: 'Notificación de congestión',
            fullDesc: 'Explicit Congestion Notification permite a los routers señalar congestión SIN descartar paquetes. Reduce retransmisiones.',
            impact: 'medium',
            profiles: ['Balanced', 'Aggressive']
        },
        timestamps: { 
            label: 'Timestamps', 
            docPath: 'tcp-core', 
            shortDesc: 'Mejor cálculo de RTT',
            fullDesc: 'RFC 1323 Timestamps permiten calcular el Round-Trip Time con mayor precisión, mejorando el control de congestión.',
            impact: 'low',
            profiles: ['Aggressive']
        },
        congestion_provider: { 
            label: 'Algoritmo', 
            docPath: 'bbr-vs-cubic', 
            shortDesc: 'Control de congestión',
            fullDesc: 'Windows usa CUBIC (estándar). Linux puede usar BBR. CUBIC es loss-based, BBR es model-based. Ambos funcionan bien.',
            impact: 'high',
            profiles: ['Conservative', 'Balanced', 'Aggressive']
        },
        // Optimizaciones Avanzadas
        fast_open: { 
            label: 'Fast Open', 
            docPath: 'tcp-core', 
            shortDesc: 'Datos en handshake',
            fullDesc: 'TCP Fast Open (TFO) envía datos en el paquete SYN inicial, ahorrando 1 RTT en conexiones repetidas. Requiere soporte del servidor.',
            impact: 'medium',
            profiles: ['Aggressive']
        },
        hystart: { 
            label: 'HyStart++', 
            docPath: 'tcp-congestion-control', 
            shortDesc: 'Slow-start inteligente',
            fullDesc: 'Hybrid Slow Start detecta cuando salir del slow-start antes de causar pérdidas. Evita queue buildup similar a BBR.',
            impact: 'medium',
            profiles: ['Balanced', 'Aggressive']
        },
        prr: { 
            label: 'PRR', 
            docPath: 'tcp-congestion-control', 
            shortDesc: 'Recuperación suave',
            fullDesc: 'Proportional Rate Reduction recupera la velocidad gradualmente tras pérdidas, en lugar de reducir bruscamente a la mitad.',
            impact: 'medium',
            profiles: ['Balanced', 'Aggressive']
        },
        pacing: { 
            label: 'Pacing', 
            docPath: 'tcp-congestion-control', 
            shortDesc: 'Envío uniforme',
            fullDesc: 'Envía paquetes de forma uniforme en el tiempo en lugar de ráfagas. Reduce buffer bloat y mejora latencia. Técnica clave de BBR.',
            impact: 'high',
            profiles: ['Aggressive']
        },
        initial_rto: { 
            label: 'Initial RTO', 
            docPath: 'tcp-core', 
            shortDesc: 'Timeout inicial',
            fullDesc: 'Retransmission Timeout inicial. El default es 3 segundos. Reducirlo a 1s acelera la detección de pérdidas pero puede causar retransmisiones innecesarias.',
            impact: 'low',
            profiles: ['Aggressive']
        },
        rack: { 
            label: 'RACK', 
            docPath: 'tcp-congestion-control', 
            shortDesc: 'Detección de pérdidas',
            fullDesc: 'Recent ACK usa timestamps para detectar pérdidas más rápido que el tradicional triple-duplicate ACK.',
            impact: 'medium',
            profiles: ['Aggressive']
        },
    };
    
    // Categorizar para mostrar
    const basicKeys = ['rss', 'rsc', 'autotuning', 'ecn', 'timestamps', 'congestion_provider'];
    const advancedKeys = ['fast_open', 'hystart', 'prr', 'pacing', 'initial_rto', 'rack'];

    $: deviceManagerTitle = platform === 'windows'
        ? 'Abrir Administrador de Dispositivos de Windows'
        : 'Abrir ajustes del sistema';
    $: deviceManagerLabel = platform === 'windows' ? 'Abrir en Windows' : 'Abrir ajustes';
    
    onMount(async () => {
        isTauri = isTauriAvailable();
        const ua = navigator.userAgent.toLowerCase();
        if (ua.includes('windows')) {
            platform = 'windows';
        } else if (ua.includes('linux')) {
            platform = 'linux';
        }
        await loadProtocolInfo();
    });
    
    async function loadProtocolInfo() {
        loadingProtocol = true;
        protocolError = null;
        dnsError = null;
        
        try {
            tcpSettings = await invoke<TcpSettings>('get_current_settings');
            detectedProfile = detectProfile(tcpSettings);
        } catch (e) {
            console.error('Error cargando protocolo TCP:', e);
            protocolError = String(e);
        }
        
        const adapterToUse = selectedAdapter || (adapters.length > 0 ? adapters[0].name : '');
        if (adapterToUse) {
            try {
                dnsConfig = await invoke<DNSConfig>('get_current_dns', { adapter: adapterToUse });
            } catch (e) {
                console.warn('No se pudo cargar DNS config:', e);
                dnsError = String(e);
            }
        }
        
        loadingProtocol = false;
    }
    
    function detectProfile(settings: any): 'Conservative' | 'Balanced' | 'Aggressive' | 'Unoptimized' | 'Custom' {
        if (!settings) return 'Unoptimized';
        
        const checks = {
            rss: settings.rss?.toLowerCase().includes('enabled'),
            rsc: settings.rsc?.toLowerCase().includes('enabled'),
            autotuning: settings.autotuning?.toLowerCase().includes('normal') || settings.autotuning?.toLowerCase().includes('experimental'),
            ecn: settings.ecn?.toLowerCase().includes('enabled'),
            timestamps: settings.timestamps?.toLowerCase().includes('enabled'),
            fast_open: settings.fast_open?.toLowerCase().includes('enabled'),
            hystart: settings.hystart?.toLowerCase().includes('enabled'),
            prr: settings.prr?.toLowerCase().includes('enabled'),
            pacing: settings.pacing?.toLowerCase().includes('always'),
        };
        
        // Contar por categoría
        const basicCount = [checks.rss, checks.rsc, checks.autotuning].filter(Boolean).length;
        const balancedCount = [checks.ecn, checks.hystart, checks.prr].filter(Boolean).length;
        const aggressiveCount = [checks.fast_open, checks.pacing, checks.timestamps].filter(Boolean).length;
        
        // Aggressive: todo habilitado
        if (basicCount === 3 && balancedCount >= 2 && aggressiveCount >= 2) {
            return 'Aggressive';
        }
        
        // Balanced: básicos + ECN/HyStart/PRR
        if (basicCount === 3 && balancedCount >= 2) {
            return 'Balanced';
        }
        
        // Conservative: solo básicos
        if (basicCount === 3) {
            return 'Conservative';
        }
        
        // Algo configurado pero no coincide con perfil
        if (basicCount > 0 || balancedCount > 0 || aggressiveCount > 0) {
            return 'Custom';
        }
        
        return 'Unoptimized';
    }
    
    function getDnsProvider(server: string): { name: string; icon: string; tier: number } {
        const known = DNS_PROVIDERS[server];
        if (known) return known;
        if (server.startsWith('192.168.') || server.startsWith('10.') || server.startsWith('172.')) {
            return { name: 'Router/Local', icon: '🏠', tier: 7 };
        }
        return { name: 'ISP/Custom', icon: '🌐', tier: 8 };
    }
    
    function getProfileInfo(profile: string): { color: string; icon: string; action: string } {
        const info: Record<string, { color: string; icon: string; action: string }> = {
            'Conservative': { color: '#4CAF50', icon: '🟢', action: 'Ya tienes optimizaciones básicas' },
            'Balanced': { color: '#FFC107', icon: '🟡', action: 'Puedes mejorar a Agresivo' },
            'Aggressive': { color: '#F44336', icon: '🔴', action: '¡Máximo rendimiento!' },
            'Unoptimized': { color: '#9E9E9E', icon: '⚪', action: 'Ir a optimizar →' },
            'Custom': { color: '#9C27B0', icon: '🟣', action: 'Configuración personalizada' },
        };
        return info[profile] || info['Unoptimized'];
    }
    
    function formatUptime(s: number): string {
        const h = Math.floor(s / 3600);
        const m = Math.floor((s % 3600) / 60);
        const sec = s % 60;
        return `${h.toString().padStart(2,'0')}:${m.toString().padStart(2,'0')}:${sec.toString().padStart(2,'0')}`;
    }
    
    function goToOptimize() {
        dispatch('navigate', { tab: 'optimize' });
    }
    
    function goToDocsSection(section: string) {
        dispatch('navigate', { tab: 'docs', section });
    }
    
    async function handleApplyProfile(event: CustomEvent<string>) {
        const profile = event.detail;
        optimizing = true;
        
        try {
            if (dryRunMode) {
                // En modo dry-run, mostrar info
                dispatch('showNotification', { 
                    type: 'info', 
                    message: `Modo simulación: el perfil "${profile}" no se aplicará` 
                });
            } else {
                // El comando espera el enum: Conservative, Balanced, Aggressive
                const result = await invoke<string[]>('apply_profile', { profile });
                await loadProtocolInfo(); // Recargar estado
                const count = result?.length || 0;
                dispatch('showNotification', { 
                    type: 'success', 
                    message: `Perfil "${profile}" aplicado (${count} optimizaciones)` 
                });
            }
        } catch (e) {
            console.error('Error aplicando perfil:', e);
            dispatch('showNotification', { 
                type: 'error', 
                message: `Error: ${e}` 
            });
        } finally {
            optimizing = false;
        }
    }
    
    function handleToggleDryRun() {
        dryRunMode = !dryRunMode;
    }
    
    async function handleResetSettings() {
        optimizing = true;
        try {
            await invoke('reset_to_defaults');
            await loadProtocolInfo();
            dispatch('showNotification', { 
                type: 'success', 
                message: 'Configuración TCP restaurada a valores por defecto' 
            });
        } catch (e) {
            console.error('Error restaurando configuración:', e);
            dispatch('showNotification', { 
                type: 'error', 
                message: `Error: ${e}` 
            });
        } finally {
            optimizing = false;
        }
    }
    
    function handleDNSChange(event: CustomEvent<{ dns: string[]; dhcp?: boolean }>) {
        const { dns, dhcp } = event.detail;
        if (dhcp) {
            dispatch('showNotification', { type: 'success', message: 'DNS restaurado a DHCP' });
        } else if (dns.length > 0) {
            dispatch('showNotification', { type: 'success', message: `DNS configurado: ${dns[0]}` });
        }
        // Recargar la info del protocolo para actualizar el estado
        loadProtocolInfo();
    }
    
    function getTcpValue(key: string): string {
        if (!tcpSettings) return 'N/A';
        const val = (tcpSettings as any)[key];
        if (val === null || val === undefined) return 'N/A';
        if (key === 'initial_rto' && typeof val === 'number') return `${val}ms`;
        
        // Para congestion_provider, mostrar algo más útil
        if (key === 'congestion_provider') {
            const v = String(val).toLowerCase();
            if (v === 'default' || v === 'unknown' || v === '' || v === 'none') {
                return 'CUBIC (default)';
            }
            // Si es NewReno u otro algoritmo, mostrarlo
            return String(val);
        }
        
        return String(val);
    }
    
    function isEnabled(key: string): boolean {
        const val = getTcpValue(key).toLowerCase();
        return val.includes('enabled') || val === 'normal' || val === 'experimental' || 
               val === 'always' || val.includes('cubic') || val.includes('newreno') ||
               val.includes('bbr');
    }
    
    function showTooltip(key: string) { activeTooltip = key; }
    function hideTooltip() { activeTooltip = null; }
    
    function safeValue(val: any, fallback: string = 'N/A'): string {
        if (val === null || val === undefined || val === '') return fallback;
        return String(val);
    }
    
    // Toggle para mostrar/ocultar gráficas de algoritmos
    let showAlgorithmCharts = true;
    
    // Manejar evento de diagnóstico completado para guardar en historial
    function handleDiagnosticComplete(result: any) {
        if (!result) return;
        
        // Calcular health score basado en resultados
        const partialReport = {
            adapterOk: result.adapter_check?.success ?? true,
            routerOk: result.router_check?.success ?? true,
            routerLatencyMs: result.router_check?.latency_ms || 0,
            ispOk: result.isp_check?.success ?? true,
            ispLatencyMs: result.isp_check?.latency_ms || 0,
            dnsOk: result.dns_check?.success ?? true,
            dnsLatencyMs: result.dns_check?.latency_ms || 0,
        };
        
        const { score, status } = calculateHealthScore(partialReport);
        
        const report: Omit<DiagnosticReport, 'id' | 'timestamp' | 'appVersion'> = {
            type: 'full',
            health: status,
            healthScore: score,
            failurePoint: result.failure_point || 'none',
            // Fases
            adapterOk: partialReport.adapterOk,
            adapterName: selectedAdapter || 'Unknown',
            routerOk: partialReport.routerOk,
            routerLatencyMs: partialReport.routerLatencyMs,
            ispOk: partialReport.ispOk,
            ispLatencyMs: partialReport.ispLatencyMs,
            dnsOk: partialReport.dnsOk,
            dnsLatencyMs: partialReport.dnsLatencyMs,
            // Metadata
            recommendation: result.recommendation || '',
            selectedAdapter: selectedAdapter,
            dnsServers: dnsConfig?.servers || [],
        };
        
        diagnosticHistory.add(report);
    }
    
    function handleViewDiagnosticDetails(event: CustomEvent<DiagnosticReport>) {
        // Por ahora mostramos en consola, luego modal
        console.log('Ver detalles:', event.detail);
    }
    
    function handleViewAllDiagnostics() {
        dispatch('navigate', { tab: 'reports' });
    }
    
    // Exponer función para que el padre pueda guardar diagnósticos
    export function saveDiagnostic(result: any) {
        handleDiagnosticComplete(result);
    }
    
    // Abrir ajustes del sistema o el administrador equivalente
    async function openDeviceManager() {
        try {
            if (isTauriAvailable()) {
                await invoke('open_device_manager');
                dispatch('showNotification', {
                    type: 'info',
                    message: platform === 'windows'
                        ? 'Administrador de Dispositivos abierto'
                        : 'Ajustes del sistema abiertos'
                });
            } else {
                dispatch('showNotification', { 
                    type: 'info', 
                    message: platform === 'windows'
                        ? '💻 En modo escritorio, esto abrirá el Administrador de Dispositivos de Windows (devmgmt.msc)'
                        : '💻 En modo escritorio, esto abrirá los ajustes del sistema de Linux'
                });
            }
        } catch (e) {
            console.error('Error abriendo Device Manager:', e);
            dispatch('showNotification', { 
                type: 'warning', 
                message: platform === 'windows'
                    ? 'No se pudo abrir el Administrador de Dispositivos. Ejecuta "devmgmt.msc" manualmente.'
                    : 'No se pudieron abrir los ajustes del sistema. Abre tu centro de control manualmente.'
            });
        }
    }
    
    $: if (selectedAdapter) { loadProtocolInfo(); }
</script>

<div class="dashboard">
    <!-- Fila 1: Métricas en tiempo real con gráficas -->
    <section class="realtime-metrics-section">
        <div class="metrics-header">
            <div class="metrics-title">
                <Icon name="activity" size={16} />
                <h3>Monitoreo en Tiempo Real</h3>
                <span class="monitoring-status" class:active={monitoringActive}>
                    {monitoringActive ? '● Activo' : '○ Detenido'}
                </span>
            </div>
            <button 
                class="monitoring-toggle-btn" 
                class:active={monitoringActive}
                on:click={() => dispatch('toggleMonitoring')}
                title={monitoringActive ? 'Detener monitoreo' : 'Iniciar monitoreo'}
            >
                <Icon name={monitoringActive ? 'pause' : 'play'} size={14} />
                {monitoringActive ? 'Detener' : 'Iniciar'}
            </button>
        </div>
        <div class="realtime-metrics-grid">
            {#if loading}
                {#each [1,2,3] as _}
                    <SkeletonCard variant="stat" />
                {/each}
            {:else}
                <MiniRealtimeChart 
                    title="Descarga" 
                    unit="Mbps" 
                    icon="arrow-down"
                    color="#00d4aa"
                    data={downloadHistory}
                    currentValue={downloadRate}
                    maxPoints={MAX_HISTORY_POINTS}
                />
                <MiniRealtimeChart 
                    title="Subida" 
                    unit="Mbps" 
                    icon="arrow-up"
                    color="#9C27B0"
                    data={uploadHistory}
                    currentValue={uploadRate}
                    maxPoints={MAX_HISTORY_POINTS}
                />
                <MiniRealtimeChart 
                    title="Latencia" 
                    unit="ms" 
                    icon="activity"
                    color="#FFC107"
                    data={latencyHistory}
                    currentValue={latency}
                    maxPoints={MAX_HISTORY_POINTS}
                />
            {/if}
        </div>
        
        <!-- Métricas adicionales: Paquetes, Errores, Drops -->
        <div class="packet-metrics-bar">
            <div class="packet-metric" title="Paquetes enviados por segundo">
                <Icon name="arrow-up-circle" size={14} />
                <span class="metric-label">TX</span>
                <span class="metric-value">{packetsSent.toLocaleString()}</span>
                <span class="metric-unit">pkt/s</span>
            </div>
            <div class="packet-metric" title="Paquetes recibidos por segundo">
                <Icon name="arrow-down-circle" size={14} />
                <span class="metric-label">RX</span>
                <span class="metric-value">{packetsRecv.toLocaleString()}</span>
                <span class="metric-unit">pkt/s</span>
            </div>
            <div class="packet-metric" class:warning={totalErrors > 0} title="Errores totales de red">
                <Icon name="alert-triangle" size={14} />
                <span class="metric-label">Errores</span>
                <span class="metric-value" class:error={totalErrors > 0}>{totalErrors}</span>
            </div>
            <div class="packet-metric" class:warning={totalDrops > 0} title="Paquetes descartados">
                <Icon name="x-circle" size={14} />
                <span class="metric-label">Drops</span>
                <span class="metric-value" class:error={totalDrops > 0}>{totalDrops}</span>
            </div>
        </div>
        
        <div class="uptime-bar">
            <Icon name="clock" size={12} />
            <span>Uptime: <strong>{formatUptime(uptime)}</strong></span>
        </div>
    </section>
    
    <!-- Fila 2: Speed Test -->
    <section class="speedtest-row">
        <SpeedTestCard 
            on:complete={(e) => {
                speedTestResult = e.detail;
                dispatch('speedTestComplete', e.detail);
            }}
        />
    </section>
    
    <!-- Fila 3: Quick Optimization Profiles -->
    <section class="quick-optimization">
        <OptimizationProfiles 
            loading={loadingProtocol}
            {optimizing}
            {tcpSettings}
            {dryRunMode}
            compact={true}
            showSettings={false}
            showReset={false}
            on:applyProfile={handleApplyProfile}
            on:toggleDryRun={handleToggleDryRun}
            on:reset={handleResetSettings}
        />
    </section>
    
    <!-- Fila 4: Protocol Status & DNS Summary (2 columnas) -->
    <div class="info-grid">
        <!-- TCP Protocol Status -->
        <section class="card protocol-card">
            <div class="card-header">
                <h2>⚙️ Configuración de Red</h2>
                <div class="header-actions">
                    <button class="doc-link-btn" on:click={() => goToDocsSection('tcp-congestion-control')} title="Ver documentación">
                        <Icon name="book-open" size={14} />
                    </button>
                    {#if detectedProfile !== 'Aggressive'}
                        <button class="btn btn-sm btn-accent" on:click={goToOptimize}>
                            ⚡ Optimizar
                        </button>
                    {/if}
                </div>
            </div>
            
            <!-- Aviso de modo web -->
            {#if !isTauri}
                <div class="web-mode-notice">
                    <Icon name="info" size={16} />
                    <span>Modo desarrollo - Los datos mostrados son de demostración. Ejecuta la app Tauri para ver tu configuración real.</span>
                </div>
            {/if}
            
            {#if loadingProtocol}
                <div class="loading-mini">
                    <span class="spinner-mini"></span>
                    Leyendo configuración del sistema...
                </div>
            {:else if protocolError}
                <div class="error-mini">
                    <Icon name="alert-triangle" size={20} />
                    <span>Error: {protocolError}</span>
                    <button class="btn btn-sm" on:click={loadProtocolInfo}>Reintentar</button>
                </div>
            {:else if tcpSettings}
                <!-- Perfil detectado con acción -->
                <div class="protocol-header">
                    <div class="detected-profile" style="--profile-color: {getProfileInfo(detectedProfile).color}">
                        <span class="profile-icon">{getProfileInfo(detectedProfile).icon}</span>
                        <span class="profile-name">{detectedProfile}</span>
                    </div>
                    <span class="profile-action">{getProfileInfo(detectedProfile).action}</span>
                </div>
                
                <!-- Configuración Base -->
                <div class="settings-section">
                    <h4 class="section-title">📊 Configuración Base</h4>
                    <div class="tcp-settings-grid">
                        {#each basicKeys as key}
                            {@const info = TCP_INFO[key]}
                            <div 
                                class="tcp-setting" 
                                class:enabled={isEnabled(key)}
                                on:mouseenter={() => showTooltip(key)}
                                on:mouseleave={hideTooltip}
                                role="button"
                                tabindex="0"
                            >
                                <div class="setting-header">
                                    <span class="setting-label">{info.label}</span>
                                    <button 
                                        class="info-btn" 
                                        on:click|stopPropagation={() => goToDocsSection(info.docPath)}
                                        title="Ver documentación"
                                    >
                                        <Icon name="help-circle" size={12} />
                                    </button>
                                </div>
                                <span class="setting-value" class:enabled={isEnabled(key)}>
                                    {getTcpValue(key)}
                                </span>
                                <span class="setting-short-desc">{info.shortDesc}</span>
                                
                                <!-- Tooltip -->
                                {#if activeTooltip === key}
                                    <div class="tooltip">
                                        <strong>{info.label}</strong>
                                        <p>{info.fullDesc}</p>
                                        <div class="tooltip-meta">
                                            <span class="impact impact-{info.impact}">Impacto: {info.impact}</span>
                                            <span class="profiles">En: {info.profiles.join(', ')}</span>
                                        </div>
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    </div>
                </div>
                
                <!-- Optimizaciones Avanzadas -->
                <div class="settings-section advanced">
                    <h4 class="section-title">
                        ⚡ Optimizaciones Avanzadas
                        <span class="section-hint" title="Técnicas similares a BBR de Linux, adaptadas para Windows">
                            <Icon name="info" size={12} />
                        </span>
                    </h4>
                    <p class="section-note">
                        Técnicas inspiradas en BBR de Linux. Windows usa CUBIC como algoritmo base, pero estas optimizaciones aplican principios similares.
                    </p>
                    <div class="tcp-settings-grid compact">
                        {#each advancedKeys as key}
                            {@const info = TCP_INFO[key]}
                            <div 
                                class="tcp-setting mini" 
                                class:enabled={isEnabled(key)}
                                on:mouseenter={() => showTooltip(key)}
                                on:mouseleave={hideTooltip}
                                role="button"
                                tabindex="0"
                            >
                                <div class="setting-header">
                                    <span class="setting-label">{info.label}</span>
                                </div>
                                <span class="setting-value mini" class:enabled={isEnabled(key)}>
                                    {getTcpValue(key)}
                                </span>
                                
                                {#if activeTooltip === key}
                                    <div class="tooltip compact">
                                        <strong>{info.label}</strong>
                                        <p>{info.fullDesc}</p>
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    </div>
                </div>
            {:else}
                <div class="empty-mini">No se pudo cargar configuración TCP</div>
            {/if}
        </section>
        
        <!-- DNS Manager -->
        <section class="dns-section">
            <DNSManager 
                adapter={selectedAdapter}
                compact={false}
                autoFailoverEnabled={autoFailoverEnabled}
                on:change={handleDNSChange}
                on:error={e => dispatch('showNotification', { type: 'error', message: e.detail })}
                on:flush={() => dispatch('showNotification', { type: 'success', message: 'Caché DNS limpiada' })}
                on:toggleFailover={() => { autoFailoverEnabled = !autoFailoverEnabled; }}
                on:detectFailover={e => { 
                    autoFailoverEnabled = e.detail.enabled;
                    dispatch('showNotification', { 
                        type: 'info', 
                        message: `Detectado: ${e.detail.count} DNS configurados - Auto-failover activado` 
                    });
                }}
                on:openFullDNS={() => dispatch('navigate', { tab: 'dns' })}
            />
        </section>
    </div>
    
    <!-- Adapters -->
    <section class="card adapters-section">
        <div class="card-header">
            <h2>📡 Adaptadores de Red</h2>
            <button 
                class="icon-btn-device" 
                on:click={openDeviceManager}
                title={deviceManagerTitle}
            >
                <Icon name="external-link" size={12} />
                <span>{deviceManagerLabel}</span>
            </button>
        </div>
        {#if loading}
            <div class="adapter-grid">
                {#each [1,2] as _}
                    <SkeletonCard variant="adapter" />
                {/each}
            </div>
        {:else if adapters.length === 0}
            <p class="empty-state">No se encontraron adaptadores activos</p>
        {:else}
            <div class="adapter-grid" style="--adapter-count: {Math.min(adapters.length, 4)}">
                {#each adapters as adapter, index}
                    {@const isSelected = adapter.name === selectedAdapter}
                    {@const isEthernet = adapter.name.toLowerCase().includes('ethernet') || adapter.description.toLowerCase().includes('realtek') || adapter.description.toLowerCase().includes('intel')}
                    {@const isPrimary = index === 0}
                    {@const isFallback = index > 0}
                    <div class="adapter-card" class:selected={isSelected} class:ethernet={isEthernet} class:primary={isPrimary} class:fallback={isFallback}>
                        <div class="adapter-header">
                            <div class="adapter-title">
                                <span class="adapter-icon">{isEthernet ? '🔌' : '📶'}</span>
                                <h3>{safeValue(adapter.name, 'Adaptador')}</h3>
                            </div>
                            <div class="adapter-badges">
                                {#if isPrimary}
                                    <span class="priority-badge primary" title="Adaptador principal - Usado para optimizaciones DNS y TCP">
                                        <Icon name="zap" size={10} />
                                        Primario
                                    </span>
                                {:else}
                                    <span class="priority-badge fallback" title="Respaldo automático si el primario falla">
                                        <Icon name="shield" size={10} />
                                        Fallback
                                    </span>
                                {/if}
                                {#if isSelected}
                                    <span class="status-badge selected" title="Adaptador activo para DNS Intelligence">🧠 DNS</span>
                                {/if}
                                <span class="status-badge up">Up</span>
                            </div>
                        </div>
                        <p class="adapter-desc">{safeValue(adapter.description, 'Sin descripción')}</p>
                        <div class="adapter-stats">
                            <span title="Velocidad de enlace">🔗 {safeValue(adapter.link_speed, 'N/A')}</span>
                            <span title="Dirección MAC (parcial)">🆔 {safeValue(adapter.mac_address?.substring(0, 8) + '...', 'N/A')}</span>
                        </div>
                        
                        {#if isPrimary}
                            <div class="adapter-priority-info">
                                <Icon name="info" size={11} />
                                <span>
                                    {#if isEthernet}
                                        Ethernet priorizado por menor latencia y mayor estabilidad
                                    {:else}
                                        Seleccionado automáticamente como primario (sin Ethernet detectado)
                                    {/if}
                                </span>
                            </div>
                        {/if}
                    </div>
                {/each}
            </div>
            
            <!-- Info boxes explicativas -->
            <div class="adapters-info">
                <div class="info-box priority">
                    <div class="info-icon"><Icon name="layers" size={14} /></div>
                    <div class="info-content">
                        <strong>¿Por qué se prioriza uno sobre otro?</strong>
                        <p>NetBoozt prioriza Ethernet sobre Wi-Fi por: menor latencia (~1ms vs ~5-20ms), menos interferencia, y conexión más estable. El sistema detecta automáticamente el tipo de adaptador.</p>
                    </div>
                </div>
                
                <div class="info-box fallback">
                    <div class="info-icon"><Icon name="shield" size={14} /></div>
                    <div class="info-content">
                        <strong>Sistema de Fallback</strong>
                        <p>Si el adaptador primario falla, el sistema cambia automáticamente al siguiente disponible sin intervención manual. Esto asegura conectividad continua.</p>
                    </div>
                </div>
                
                <div class="info-box limitation">
                    <div class="info-icon"><Icon name="alert-triangle" size={14} /></div>
                    <div class="info-content">
                        <strong>¿Por qué no hay "Boost Dual"?</strong>
                        <p>En Windows, combinar múltiples adaptadores para boost simultáneo requiere acceso a nivel de kernel (NDIS drivers) y licencias de Microsoft. Esta funcionalidad está reservada para NIC Teaming empresarial.</p>
                    </div>
                </div>
            </div>
        {/if}
    </section>
    
    <!-- Algorithm Visualization & Optimization Comparison -->
    <section class="card algorithm-section">
        <div class="card-header">
            <h2>📈 Optimizaciones TCP/IP</h2>
            <button 
                class="toggle-btn" 
                on:click={() => showAlgorithmCharts = !showAlgorithmCharts}
                title={showAlgorithmCharts ? 'Ocultar comparativas' : 'Ver comparativas'}
            >
                {showAlgorithmCharts ? 'Ocultar' : 'Ver comparativas'}
                <Icon name={showAlgorithmCharts ? 'chevron-up' : 'chevron-down'} size={14} />
            </button>
        </div>
        
        <div class="algorithm-intro-box">
            <div class="intro-icon"><Icon name="cpu" size={16} /></div>
            <div class="intro-content">
                <p>
                    Windows usa <strong>CUBIC</strong> como algoritmo de control de congestión. NetBoozt aplica 
                    optimizaciones inspiradas en <strong>Google BBR</strong> para mejorar el rendimiento:
                </p>
            </div>
        </div>
        
        {#if showAlgorithmCharts}
            <!-- Comparativas de Algoritmos -->
            <div class="algorithm-comparison-grid">
                <!-- CUBIC vs BBR-like -->
                <div class="comparison-section">
                    <h4 class="comparison-title">
                        <Icon name="git-branch" size={14} />
                        Control de Congestión
                    </h4>
                    <div class="comparison-charts">
                        <div class="algorithm-chart-wrapper default-algo">
                            <div class="algo-badge windows">Windows Default</div>
                            <h5>CUBIC</h5>
                            <p class="chart-desc">Reduce ventana 50% ante pérdida</p>
                            <AlgorithmChart algorithm="cubic" animated compact />
                        </div>
                        <div class="comparison-arrow">
                            <Icon name="arrow-right" size={20} />
                        </div>
                        <div class="algorithm-chart-wrapper optimized-algo">
                            <div class="algo-badge netboozt">Con NetBoozt</div>
                            <h5>BBR-like</h5>
                            <p class="chart-desc">Mantiene throughput estable</p>
                            <AlgorithmChart algorithm="bbr-like" animated compact />
                        </div>
                    </div>
                </div>
            </div>
            
            <!-- Optimizaciones Detalladas -->
            <div class="optimizations-comparison">
                <h4 class="section-subtitle">
                    <Icon name="zap" size={14} />
                    Comparativa de Optimizaciones
                </h4>
                
                <div class="opt-grid">
                    <!-- HyStart++ -->
                    <div class="opt-card">
                        <div class="opt-header">
                            <span class="opt-name">HyStart++</span>
                            <span class="opt-status enabled">Habilitado</span>
                        </div>
                        <div class="opt-comparison">
                            <div class="opt-default">
                                <span class="opt-label">❌ Sin optimizar</span>
                                <p>Slow-start agresivo, puede causar pérdidas y buffer overflow</p>
                            </div>
                            <div class="opt-improved">
                                <span class="opt-label">✅ Con NetBoozt</span>
                                <p>Sale temprano del slow-start, evita saturar buffers</p>
                            </div>
                        </div>
                        <div class="opt-benefit">
                            <Icon name="trending-up" size={12} />
                            <span>Reduce pérdidas en conexiones nuevas hasta 30%</span>
                        </div>
                    </div>
                    
                    <!-- PRR -->
                    <div class="opt-card">
                        <div class="opt-header">
                            <span class="opt-name">PRR (Proportional Rate Reduction)</span>
                            <span class="opt-status enabled">Habilitado</span>
                        </div>
                        <div class="opt-comparison">
                            <div class="opt-default">
                                <span class="opt-label">❌ Sin optimizar</span>
                                <p>Reducción brusca de velocidad (50%) ante pérdidas</p>
                            </div>
                            <div class="opt-improved">
                                <span class="opt-label">✅ Con NetBoozt</span>
                                <p>Reducción proporcional y gradual, recuperación suave</p>
                            </div>
                        </div>
                        <div class="opt-benefit">
                            <Icon name="trending-up" size={12} />
                            <span>Recuperación 2x más rápida tras congestión</span>
                        </div>
                    </div>
                    
                    <!-- ECN -->
                    <div class="opt-card">
                        <div class="opt-header">
                            <span class="opt-name">ECN (Explicit Congestion Notification)</span>
                            <span class="opt-status enabled">Habilitado</span>
                        </div>
                        <div class="opt-comparison">
                            <div class="opt-default">
                                <span class="opt-label">❌ Sin optimizar</span>
                                <p>Solo detecta congestión cuando hay pérdidas reales</p>
                            </div>
                            <div class="opt-improved">
                                <span class="opt-label">✅ Con NetBoozt</span>
                                <p>Routers marcan paquetes antes de descartarlos</p>
                            </div>
                        </div>
                        <div class="opt-benefit">
                            <Icon name="trending-up" size={12} />
                            <span>Previene pérdidas, mejor para gaming y VoIP</span>
                        </div>
                    </div>
                    
                    <!-- TCP Pacing -->
                    <div class="opt-card">
                        <div class="opt-header">
                            <span class="opt-name">TCP Pacing</span>
                            <span class="opt-status enabled">Habilitado</span>
                        </div>
                        <div class="opt-comparison">
                            <div class="opt-default">
                                <span class="opt-label">❌ Sin optimizar</span>
                                <p>Envío en ráfagas, puede saturar buffers intermedios</p>
                            </div>
                            <div class="opt-improved">
                                <span class="opt-label">✅ Con NetBoozt</span>
                                <p>Envío espaciado uniformemente en el tiempo</p>
                            </div>
                        </div>
                        <div class="opt-benefit">
                            <Icon name="trending-up" size={12} />
                            <span>Latencia más estable, menos jitter</span>
                        </div>
                    </div>
                    
                    <!-- TFO -->
                    <div class="opt-card">
                        <div class="opt-header">
                            <span class="opt-name">TCP Fast Open</span>
                            <span class="opt-status enabled">Habilitado</span>
                        </div>
                        <div class="opt-comparison">
                            <div class="opt-default">
                                <span class="opt-label">❌ Sin optimizar</span>
                                <p>3-way handshake tradicional (3 RTT para datos)</p>
                            </div>
                            <div class="opt-improved">
                                <span class="opt-label">✅ Con NetBoozt</span>
                                <p>Datos en el SYN inicial (ahorra 1 RTT)</p>
                            </div>
                        </div>
                        <div class="opt-benefit">
                            <Icon name="trending-up" size={12} />
                            <span>Conexiones 33% más rápidas en reconexiones</span>
                        </div>
                    </div>
                    
                    <!-- Initial RTO -->
                    <div class="opt-card">
                        <div class="opt-header">
                            <span class="opt-name">Initial RTO Reducido</span>
                            <span class="opt-status enabled">Habilitado</span>
                        </div>
                        <div class="opt-comparison">
                            <div class="opt-default">
                                <span class="opt-label">❌ Sin optimizar</span>
                                <p>Timeout inicial de 3 segundos (muy conservador)</p>
                            </div>
                            <div class="opt-improved">
                                <span class="opt-label">✅ Con NetBoozt</span>
                                <p>Timeout inicial de 1 segundo</p>
                            </div>
                        </div>
                        <div class="opt-benefit">
                            <Icon name="trending-up" size={12} />
                            <span>Detección de problemas 3x más rápida</span>
                        </div>
                    </div>
                </div>
            </div>
            
            <div class="algorithm-legend">
                <span class="legend-item"><span class="dot loss"></span> Pérdida de paquete</span>
                <span class="legend-item"><span class="dot speed"></span> Velocidad de transmisión</span>
            </div>
            
            <p class="simulation-note">
                <Icon name="info" size={11} />
                Las gráficas muestran comportamiento simulado. El impacto real depende de tu conexión, ISP y la congestión de red.
            </p>
        {/if}
    </section>
    
    <!-- Diagnostic -->
    <section class="card">
        <h2>🔍 Diagnóstico de Red</h2>
        <button class="btn btn-primary btn-lg" on:click={() => dispatch('diagnose')} disabled={diagnosing}>
            {#if diagnosing}
                <span class="spinner-small"></span> Diagnosticando...
            {:else}
                Ejecutar Diagnóstico de 5 Fases
            {/if}
        </button>
        <DiagnosticChain 
            {diagnostic} 
            loading={diagnosing} 
            on:toast={(e) => dispatch('toast', e.detail)}
        />
        
        <!-- Historial de diagnósticos -->
        <div class="diagnostic-history-section">
            <DiagnosticHistory 
                compact 
                limit={5}
                on:viewDetails={handleViewDiagnosticDetails}
                on:viewAll={handleViewAllDiagnostics}
            />
        </div>
    </section>
</div>

<style>
    .dashboard { display: flex; flex-direction: column; gap: 1.25rem; }
    
    /* ========== REALTIME METRICS SECTION ========== */
    .realtime-metrics-section {
        background: var(--bg-card, #1a1a1a);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 12px;
        padding: 1rem 1.25rem;
    }
    
    .metrics-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 0.75rem;
    }
    
    .metrics-title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
    
    .metrics-header h3 {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--text-primary, #fff);
        margin: 0;
    }
    
    .monitoring-status {
        font-size: 0.7rem;
        padding: 0.2rem 0.5rem;
        border-radius: 10px;
        background: var(--bg-tertiary, #3d3d3d);
        color: var(--text-muted, #666);
    }
    
    .monitoring-status.active {
        background: rgba(0, 212, 170, 0.15);
        color: #00d4aa;
    }
    
    .monitoring-toggle-btn {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.4rem 0.75rem;
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 6px;
        background: var(--bg-tertiary, #2b2b2b);
        color: var(--text-secondary, #aaa);
        font-size: 0.75rem;
        cursor: pointer;
        transition: all 0.2s ease;
    }
    
    .monitoring-toggle-btn:hover {
        background: var(--bg-hover, #3d3d3d);
        color: var(--text-primary, #fff);
    }
    
    .monitoring-toggle-btn.active {
        border-color: #ff6b6b;
        color: #ff6b6b;
    }
    
    .monitoring-toggle-btn.active:hover {
        background: rgba(255, 107, 107, 0.1);
    }
    
    .realtime-metrics-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 1rem;
    }
    
    @media (max-width: 800px) {
        .realtime-metrics-grid {
            grid-template-columns: 1fr;
        }
    }
    
    /* Barra de métricas de paquetes */
    .packet-metrics-bar {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 1.5rem;
        margin-top: 0.75rem;
        padding: 0.65rem 1rem;
        background: var(--bg-tertiary, #252525);
        border-radius: 8px;
        flex-wrap: wrap;
    }
    
    .packet-metric {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.75rem;
        color: var(--text-secondary, #aaa);
    }
    
    .packet-metric.warning {
        color: #FFC107;
    }
    
    .metric-label {
        font-weight: 500;
        color: var(--text-muted, #666);
    }
    
    .metric-value {
        font-weight: 600;
        font-variant-numeric: tabular-nums;
        color: var(--text-primary, #fff);
    }
    
    .metric-value.error {
        color: #ff6b6b;
    }
    
    .metric-unit {
        font-size: 0.65rem;
        color: var(--text-muted, #666);
    }
    
    @media (max-width: 600px) {
        .packet-metrics-bar {
            gap: 1rem;
        }
        
        .packet-metric {
            font-size: 0.7rem;
        }
    }
    
    .uptime-bar {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        margin-top: 0.75rem;
        padding-top: 0.75rem;
        border-top: 1px solid var(--border, #3d3d3d);
        font-size: 0.75rem;
        color: var(--text-muted, #666);
    }
    
    .uptime-bar strong {
        color: var(--text-primary, #fff);
        font-variant-numeric: tabular-nums;
    }
    
    /* Fila 1: Quick Stats - ya no se usa, ahora usamos realtime-metrics-section */
    .quick-stats-row {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 0.75rem;
    }
    
    @media (max-width: 900px) {
        .quick-stats-row {
            grid-template-columns: repeat(2, 1fr);
        }
    }
    
    @media (max-width: 500px) {
        .quick-stats-row {
            grid-template-columns: 1fr;
        }
    }
    
    /* Fila 2: Speed Test - ancho completo */
    .speedtest-row {
        background: transparent;
        border-radius: 12px;
        padding: 0;
    }
    
    .speedtest-row :global(.speedtest-card) {
        min-height: 480px;
    }
    
    .speedtest-row :global(.speedtest-card.compact) {
        min-height: 400px;
    }

    /* Quick Optimization Section */
    .quick-optimization {
        margin-bottom: 0;
    }
    
    .quick-optimization :global(.optimization-profiles) {
        border: 1px solid var(--border, #3d3d3d);
    }
    
    /* Fila 4: Info Grid - Protocol & DNS lado a lado */
    .info-grid { 
        display: grid; 
        grid-template-columns: repeat(2, 1fr); 
        gap: 1rem; 
    }
    
    @media (max-width: 1100px) {
        .info-grid { 
            grid-template-columns: 1fr; 
        }
    }
    
    /* DNS Section */
    .dns-section {
        background: var(--bg-card, #1a1a1a);
        border-radius: 12px;
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        min-height: 350px;
    }
    
    .card { background: var(--bg-card, #1a1a1a); border-radius: 12px; padding: 1.25rem; }
    .card h2 { font-size: 1rem; font-weight: 600; margin: 0 0 1rem 0; color: var(--text-primary, #fff); }
    .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
    .card-header h2 { margin: 0; }
    .header-actions { display: flex; gap: 0.5rem; align-items: center; }
    
    /* Protocol Card */
    .protocol-card { min-height: 350px; }

    /* Web mode notice */
    .web-mode-notice {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.65rem 0.85rem;
        background: rgba(255, 193, 7, 0.1);
        border: 1px solid rgba(255, 193, 7, 0.3);
        border-radius: 8px;
        font-size: 0.75rem;
        color: var(--warning, #fdcb6e);
        margin-bottom: 1rem;
    }
    
    .doc-link-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0.4rem;
        background: var(--bg-elevated, #2b2b2b);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 6px;
        color: var(--text-secondary, #a0a0a0);
        cursor: pointer;
        transition: all 0.15s;
    }
    
    .doc-link-btn:hover {
        background: var(--primary, #00d4aa);
        color: #000;
        border-color: var(--primary, #00d4aa);
    }
    
    .btn-accent {
        background: var(--primary, #00d4aa) !important;
        color: #000 !important;
        font-weight: 600;
    }
    
    .btn-accent:hover {
        background: #00e6b8 !important;
    }
    
    .loading-mini, .empty-mini {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 1.5rem;
        color: var(--text-muted, #666);
        font-size: 0.8125rem;
    }
    
    .spinner-mini {
        width: 20px;
        height: 20px;
        border: 2px solid var(--border, #3d3d3d);
        border-top-color: var(--primary, #00d4aa);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }
    
    .error-mini {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 1.5rem;
        color: var(--error, #ff6b6b);
        font-size: 0.8125rem;
        text-align: center;
        background: rgba(255, 107, 107, 0.05);
        border-radius: 8px;
    }
    
    .error-mini.compact {
        flex-direction: row;
        padding: 1rem;
    }
    
    .protocol-header {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 1rem;
    }
    
    .detected-profile {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.4rem 0.85rem;
        background: rgba(255,255,255,0.05);
        border: 2px solid var(--profile-color);
        border-radius: 20px;
    }
    
    .profile-icon { font-size: 1rem; }
    .profile-name { font-size: 0.875rem; font-weight: 600; color: var(--profile-color); }
    .profile-action { font-size: 0.75rem; color: var(--text-secondary, #a0a0a0); }
    
    /* Settings Sections */
    .settings-section { margin-bottom: 1rem; }
    .settings-section.advanced {
        padding-top: 0.75rem;
        border-top: 1px solid var(--border, #3d3d3d);
    }
    
    .section-title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.75rem;
        font-weight: 600;
        color: var(--text-secondary, #a0a0a0);
        margin: 0 0 0.5rem 0;
    }
    
    .section-hint {
        cursor: help;
        opacity: 0.6;
    }
    
    .section-note {
        font-size: 0.7rem;
        color: var(--text-muted, #666);
        margin: 0 0 0.65rem 0;
        line-height: 1.4;
    }
    
    .tcp-settings-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 0.75rem;
    }
    
    @media (min-width: 1200px) {
        .tcp-settings-grid {
            grid-template-columns: repeat(6, 1fr);
        }
    }
    
    @media (max-width: 700px) {
        .tcp-settings-grid {
            grid-template-columns: repeat(2, 1fr);
        }
    }
    
    .tcp-settings-grid.compact {
        grid-template-columns: repeat(6, 1fr);
        gap: 0.5rem;
    }
    
    @media (max-width: 700px) {
        .tcp-settings-grid.compact {
            grid-template-columns: repeat(3, 1fr);
        }
    }
    
    .tcp-setting {
        position: relative;
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 8px;
        padding: 0.6rem;
        border-left: 3px solid var(--border, #3d3d3d);
        transition: all 0.15s;
        cursor: default;
    }
    
    .tcp-setting:hover {
        background: var(--bg-hover, #333);
    }
    
    .tcp-setting.enabled {
        border-left-color: var(--primary, #00d4aa);
    }
    
    .tcp-setting.mini {
        padding: 0.45rem;
    }
    
    .setting-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.25rem;
    }
    
    .setting-label {
        font-size: 0.7rem;
        font-weight: 600;
        color: var(--text-secondary, #a0a0a0);
        text-transform: uppercase;
        letter-spacing: 0.3px;
    }
    
    .info-btn {
        background: transparent;
        border: none;
        color: var(--text-muted, #666);
        padding: 0.15rem;
        cursor: pointer;
        opacity: 0;
        transition: all 0.15s;
        border-radius: 50%;
    }
    
    .tcp-setting:hover .info-btn { opacity: 1; }
    .info-btn:hover { color: var(--primary, #00d4aa); background: rgba(0,212,170,0.1); }
    
    .setting-value {
        display: block;
        font-size: 0.8125rem;
        font-weight: 500;
        color: var(--text-muted, #666);
        margin-bottom: 0.2rem;
    }
    
    .setting-value.mini {
        font-size: 0.75rem;
        margin-bottom: 0;
    }
    
    .setting-value.enabled {
        color: var(--primary, #00d4aa);
    }
    
    .setting-short-desc {
        display: block;
        font-size: 0.625rem;
        color: var(--text-muted, #666);
        line-height: 1.3;
    }
    
    /* Tooltips */
    .tooltip {
        position: absolute;
        bottom: calc(100% + 8px);
        left: 50%;
        transform: translateX(-50%);
        z-index: 100;
        min-width: 220px;
        max-width: 280px;
        padding: 0.75rem;
        background: var(--bg-card, #1a1a1a);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
        font-size: 0.75rem;
        color: var(--text-primary, #fff);
        pointer-events: none;
    }
    
    .tooltip.compact {
        min-width: 180px;
        padding: 0.6rem;
    }
    
    .tooltip::after {
        content: '';
        position: absolute;
        top: 100%;
        left: 50%;
        transform: translateX(-50%);
        border: 6px solid transparent;
        border-top-color: var(--border, #3d3d3d);
    }
    
    .tooltip strong {
        display: block;
        color: var(--primary, #00d4aa);
        margin-bottom: 0.35rem;
    }
    
    .tooltip p {
        margin: 0 0 0.5rem 0;
        line-height: 1.4;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .tooltip-meta {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        font-size: 0.65rem;
        border-top: 1px solid var(--border, #3d3d3d);
        padding-top: 0.5rem;
        margin-top: 0.35rem;
    }
    
    .impact {
        padding: 0.15rem 0.4rem;
        border-radius: 4px;
        font-weight: 600;
    }
    
    .impact-high { background: rgba(244,67,54,0.15); color: #F44336; }
    .impact-medium { background: rgba(255,193,7,0.15); color: #FFC107; }
    .impact-low { background: rgba(76,175,80,0.15); color: #4CAF50; }
    
    .profiles {
        color: var(--text-muted, #666);
    }
    
    /* DNS Card */
    .dns-mode {
        margin-bottom: 1rem;
    }
    
    .dhcp-badge, .static-badge {
        display: inline-block;
        padding: 0.35rem 0.75rem;
        font-size: 0.75rem;
        font-weight: 600;
        border-radius: 6px;
    }
    
    .dhcp-badge {
        background: rgba(255, 193, 7, 0.15);
        color: var(--warning, #fdcb6e);
    }
    
    .static-badge {
        background: rgba(0, 212, 170, 0.15);
        color: var(--primary, #00d4aa);
    }
    
    .dns-hint {
        font-size: 0.75rem;
        color: var(--text-muted, #666);
        margin: 0.5rem 0 0 0;
    }
    
    .dns-hint.good {
        color: var(--primary, #00d4aa);
    }
    
    .failover-explanation {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 0.75rem;
        background: rgba(0, 212, 170, 0.08);
        border-radius: 6px;
        font-size: 0.7rem;
        color: var(--text-secondary, #a0a0a0);
        margin-bottom: 0.75rem;
    }
    
    .dns-servers-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        max-height: 200px;
        overflow-y: auto;
    }
    
    .dns-server-item {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        padding: 0.65rem;
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 8px;
        border-left: 3px solid var(--border, #3d3d3d);
    }
    
    .dns-server-item.primary {
        border-left-color: var(--primary, #00d4aa);
    }
    
    .dns-server-item.fallback {
        opacity: 0.75;
        padding: 0.5rem 0.65rem;
    }
    
    .dns-rank {
        font-size: 0.65rem;
        font-weight: 600;
        text-transform: uppercase;
        color: var(--text-muted, #666);
        letter-spacing: 0.5px;
    }
    
    .dns-server-info {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-wrap: wrap;
    }
    
    .dns-icon { font-size: 1rem; }
    
    .dns-ip {
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.875rem;
        color: var(--text-primary, #fff);
    }
    
    .dns-provider {
        font-size: 0.75rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .tier-badge {
        font-size: 0.625rem;
        font-weight: 600;
        padding: 0.15rem 0.4rem;
        border-radius: 4px;
    }
    
    .tier-badge.tier-1 { background: rgba(0, 212, 170, 0.2); color: var(--primary, #00d4aa); }
    .tier-badge.tier-2 { background: rgba(66, 133, 244, 0.2); color: #4285F4; }
    .tier-badge.tier-3 { background: rgba(156, 39, 176, 0.2); color: #9C27B0; }
    
    .dns-warning {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.65rem;
        background: rgba(255, 193, 7, 0.1);
        border-radius: 8px;
        font-size: 0.75rem;
        color: var(--warning, #fdcb6e);
    }
    
    /* Adapters */
    .adapters-section .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
    .adapters-section .card-header h2 { margin: 0; }
    .icon-btn-device { 
        display: inline-flex; 
        align-items: center; 
        gap: 0.35rem; 
        padding: 0.4rem 0.75rem; 
        background: var(--bg-elevated, #2b2b2b); 
        border: 1px solid var(--border, #3d3d3d); 
        border-radius: 6px; 
        color: var(--text-secondary, #a0a0a0); 
        font-size: 0.6875rem; 
        cursor: pointer; 
        transition: all 0.15s; 
    }
    .icon-btn-device:hover { background: var(--primary, #00d4aa); color: #000; border-color: var(--primary, #00d4aa); }
    
    /* Adapter grid: divide el ancho total entre N adaptadores */
    .adapter-grid { 
        display: grid; 
        grid-template-columns: repeat(var(--adapter-count, 2), 1fr);
        gap: 1rem; 
    }
    
    /* Para 1 adaptador, limitar ancho */
    .adapter-grid:has(> :only-child) {
        grid-template-columns: minmax(300px, 500px);
        justify-content: start;
    }
    
    /* Responsive: 1 columna en móvil */
    @media (max-width: 700px) {
        .adapter-grid { grid-template-columns: 1fr; }
    }
    
    .adapter-card { background: var(--bg-elevated, #2b2b2b); border-radius: 10px; padding: 1rem; border: 2px solid transparent; transition: all 0.2s; position: relative; }
    .adapter-card.selected { border-color: var(--primary, #00d4aa); background: rgba(0, 212, 170, 0.05); }
    .adapter-card.primary { border-color: var(--primary, #00d4aa); }
    .adapter-card.fallback { border-color: var(--border, #3d3d3d); opacity: 0.85; }
    .adapter-card.fallback:hover { opacity: 1; }
    .adapter-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.5rem; gap: 0.5rem; }
    .adapter-title { display: flex; align-items: center; gap: 0.5rem; }
    .adapter-icon { font-size: 1rem; }
    .adapter-badges { display: flex; gap: 0.35rem; flex-wrap: wrap; }
    .adapter-card h3 { font-size: 0.9375rem; font-weight: 600; color: var(--text-primary, #fff); margin: 0; }
    .adapter-card.selected h3 { color: var(--primary, #00d4aa); }
    
    .priority-badge {
        display: inline-flex;
        align-items: center;
        gap: 0.25rem;
        font-size: 0.5625rem;
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
        font-weight: 600;
        text-transform: uppercase;
    }
    .priority-badge.primary { background: rgba(0, 212, 170, 0.2); color: var(--primary, #00d4aa); }
    .priority-badge.fallback { background: rgba(255, 193, 7, 0.15); color: var(--warning, #fdcb6e); }
    
    .status-badge { font-size: 0.625rem; padding: 0.2rem 0.5rem; border-radius: 4px; font-weight: 600; text-transform: uppercase; }
    .status-badge.up { background: rgba(0, 212, 170, 0.2); color: var(--primary, #00d4aa); }
    .status-badge.selected { background: rgba(100, 124, 255, 0.2); color: #6b7cff; font-size: 0.5625rem; }
    .adapter-desc { font-size: 0.75rem; color: var(--text-muted, #666); margin: 0 0 0.75rem 0; }
    .adapter-stats { display: flex; gap: 1rem; font-size: 0.7rem; color: var(--text-secondary, #a0a0a0); }
    
    .adapter-priority-info { 
        display: flex; 
        align-items: center; 
        gap: 0.35rem; 
        margin-top: 0.75rem; 
        padding: 0.5rem 0.65rem; 
        background: rgba(0, 212, 170, 0.08); 
        border-radius: 6px; 
        font-size: 0.6875rem; 
        color: var(--primary, #00d4aa); 
    }
    
    /* Info boxes explicativas */
    .adapters-info {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 0.75rem;
        margin-top: 1rem;
    }
    
    .info-box {
        display: flex;
        gap: 0.75rem;
        padding: 0.75rem;
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 8px;
        border-left: 3px solid var(--border, #3d3d3d);
    }
    
    .info-box.priority { border-left-color: var(--primary, #00d4aa); }
    .info-box.fallback { border-left-color: var(--warning, #fdcb6e); }
    .info-box.limitation { border-left-color: var(--error, #ff6b6b); }
    
    .info-box .info-icon {
        display: flex;
        align-items: flex-start;
        padding-top: 0.1rem;
        color: var(--text-muted, #666);
    }
    
    .info-box.priority .info-icon { color: var(--primary, #00d4aa); }
    .info-box.fallback .info-icon { color: var(--warning, #fdcb6e); }
    .info-box.limitation .info-icon { color: var(--error, #ff6b6b); }
    
    .info-box .info-content {
        flex: 1;
    }
    
    .info-box .info-content strong {
        display: block;
        font-size: 0.75rem;
        color: var(--text-primary, #fff);
        margin-bottom: 0.25rem;
    }
    
    .info-box .info-content p {
        margin: 0;
        font-size: 0.6875rem;
        color: var(--text-muted, #666);
        line-height: 1.4;
    }

    /* Buttons & misc */
    .empty-state { text-align: center; padding: 2rem; color: var(--text-muted, #666); }
    .btn { display: inline-flex; align-items: center; gap: 0.5rem; padding: 0.625rem 1rem; border-radius: 8px; font-size: 0.875rem; font-weight: 500; cursor: pointer; border: none; transition: all 0.15s; }
    .btn:disabled { opacity: 0.5; cursor: not-allowed; }
    .btn-primary { background: var(--primary, #00d4aa); color: #000; }
    .btn-primary:hover:not(:disabled) { background: #00e6b8; }
    .btn-lg { padding: 0.875rem 1.5rem; font-size: 0.9375rem; }
    .btn-sm { padding: 0.4rem 0.75rem; font-size: 0.75rem; margin-top: 0.5rem; background: var(--bg-elevated, #2b2b2b); color: var(--text-secondary, #a0a0a0); }
    .btn-sm:hover { background: var(--primary, #00d4aa); color: #000; }
    .spinner-small { width: 16px; height: 16px; border: 2px solid rgba(0,0,0,0.2); border-top-color: #000; border-radius: 50%; animation: spin 1s linear infinite; }
    @keyframes spin { to { transform: rotate(360deg); } }
    
    /* Algorithm Section */
    .algorithm-section {
        background: var(--bg-card, #1a1a1a);
    }
    
    .toggle-btn {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.4rem 0.75rem;
        background: var(--bg-elevated, #2b2b2b);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 6px;
        color: var(--text-secondary, #a0a0a0);
        font-size: 0.75rem;
        cursor: pointer;
        transition: all 0.15s;
    }
    
    .toggle-btn:hover {
        background: var(--primary, #00d4aa);
        color: #000;
        border-color: var(--primary, #00d4aa);
    }
    
    .algorithm-intro-box {
        display: flex;
        gap: 1rem;
        padding: 1rem;
        background: rgba(0, 212, 170, 0.05);
        border: 1px solid rgba(0, 212, 170, 0.15);
        border-radius: 10px;
        margin-bottom: 1rem;
    }
    
    .algorithm-intro-box .intro-icon {
        display: flex;
        align-items: flex-start;
        padding-top: 0.15rem;
        color: var(--primary, #00d4aa);
    }
    
    .algorithm-intro-box .intro-content p {
        margin: 0 0 0.75rem 0;
        font-size: 0.8125rem;
        color: var(--text-secondary, #a0a0a0);
        line-height: 1.5;
    }
    
    .algorithm-intro-box .intro-content strong {
        color: var(--primary, #00d4aa);
    }
    
    .simulation-note {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        margin-top: 0.75rem;
        padding: 0.5rem 0.75rem;
        background: rgba(255, 193, 7, 0.08);
        border-radius: 6px;
        font-size: 0.6875rem;
        color: var(--warning, #fdcb6e);
    }
    
    .algorithm-chart-wrapper.optimized {
        border: 1px solid rgba(0, 212, 170, 0.2);
        background: rgba(0, 212, 170, 0.03);
    }
    
    .algorithm-intro {
        font-size: 0.8125rem;
        color: var(--text-secondary, #a0a0a0);
        margin: 0;
        line-height: 1.5;
    }
    
    .algorithm-intro-box strong {
        color: var(--primary, #00d4aa);
    }
    
    .algorithm-charts {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
        gap: 1rem;
    }
    
    .algorithm-chart-wrapper {
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 10px;
        padding: 1rem;
        border: 1px solid var(--border, #3d3d3d);
    }
    
    .chart-desc {
        font-size: 0.7rem;
        color: var(--text-muted, #666);
        margin: 0 0 0.75rem 0;
    }
    
    .algorithm-legend {
        display: flex;
        justify-content: center;
        gap: 1.5rem;
        margin-top: 1rem;
        padding-top: 0.75rem;
        border-top: 1px solid var(--border, #3d3d3d);
    }
    
    .legend-item {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.7rem;
        color: var(--text-muted, #666);
    }
    
    .dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
    }
    
    .dot.loss {
        background: var(--error, #ff6b6b);
    }
    
    .dot.speed {
        background: var(--primary, #00d4aa);
    }
    
    /* ========== OPTIMIZATION COMPARISON STYLES ========== */
    .algorithm-comparison-grid {
        margin-bottom: 1.5rem;
    }
    
    .comparison-section {
        background: rgba(0, 0, 0, 0.2);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 10px;
        padding: 1rem;
    }
    
    .comparison-title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.85rem;
        font-weight: 600;
        color: var(--text-primary, #fff);
        margin: 0 0 1rem 0;
    }
    
    .comparison-charts {
        display: grid;
        grid-template-columns: 1fr auto 1fr;
        gap: 1rem;
        align-items: center;
    }
    
    @media (max-width: 800px) {
        .comparison-charts {
            grid-template-columns: 1fr;
        }
        .comparison-arrow {
            transform: rotate(90deg);
        }
    }
    
    .comparison-arrow {
        color: var(--primary, #00d4aa);
        opacity: 0.7;
    }
    
    .algorithm-chart-wrapper.default-algo {
        border-color: rgba(255, 152, 0, 0.3);
    }
    
    .algorithm-chart-wrapper.optimized-algo {
        border-color: rgba(0, 212, 170, 0.3);
        background: rgba(0, 212, 170, 0.05);
    }
    
    .algo-badge {
        display: inline-block;
        font-size: 0.6rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }
    
    .algo-badge.windows {
        background: rgba(255, 152, 0, 0.15);
        color: #ff9800;
    }
    
    .algo-badge.netboozt {
        background: rgba(0, 212, 170, 0.15);
        color: var(--primary, #00d4aa);
    }
    
    .algorithm-chart-wrapper h5 {
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--text-primary, #fff);
        margin: 0 0 0.25rem 0;
    }
    
    /* Optimizations Comparison Grid */
    .optimizations-comparison {
        margin-top: 1.5rem;
    }
    
    .section-subtitle {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.85rem;
        font-weight: 600;
        color: var(--text-primary, #fff);
        margin: 0 0 1rem 0;
    }
    
    .opt-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 1rem;
    }
    
    @media (max-width: 900px) {
        .opt-grid {
            grid-template-columns: 1fr;
        }
    }
    
    .opt-card {
        background: rgba(0, 0, 0, 0.25);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 10px;
        padding: 1rem;
        transition: border-color 0.2s;
    }
    
    .opt-card:hover {
        border-color: rgba(0, 212, 170, 0.3);
    }
    
    .opt-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.75rem;
    }
    
    .opt-name {
        font-size: 0.85rem;
        font-weight: 600;
        color: var(--text-primary, #fff);
    }
    
    .opt-status {
        font-size: 0.65rem;
        font-weight: 600;
        text-transform: uppercase;
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
    }
    
    .opt-status.enabled {
        background: rgba(0, 212, 170, 0.15);
        color: var(--primary, #00d4aa);
    }
    
    .opt-status.disabled {
        background: rgba(255, 255, 255, 0.1);
        color: var(--text-muted, #666);
    }
    
    .opt-comparison {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 0.75rem;
        margin-bottom: 0.75rem;
    }
    
    .opt-default, .opt-improved {
        padding: 0.75rem;
        border-radius: 8px;
        font-size: 0.75rem;
    }
    
    .opt-default {
        background: rgba(255, 82, 82, 0.08);
        border: 1px solid rgba(255, 82, 82, 0.2);
    }
    
    .opt-improved {
        background: rgba(0, 212, 170, 0.08);
        border: 1px solid rgba(0, 212, 170, 0.2);
    }
    
    .opt-label {
        display: block;
        font-size: 0.7rem;
        font-weight: 600;
        margin-bottom: 0.35rem;
    }
    
    .opt-default .opt-label {
        color: var(--error, #ff5252);
    }
    
    .opt-improved .opt-label {
        color: var(--primary, #00d4aa);
    }
    
    .opt-default p, .opt-improved p {
        margin: 0;
        color: var(--text-secondary, #a0a0a0);
        line-height: 1.4;
    }
    
    .opt-benefit {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 0.75rem;
        background: rgba(0, 212, 170, 0.1);
        border-radius: 6px;
        font-size: 0.7rem;
        color: var(--primary, #00d4aa);
    }

    /* Diagnostic History Section */
    .diagnostic-history-section {
        margin-top: 1.25rem;
        padding-top: 1.25rem;
        border-top: 1px solid var(--border, #3d3d3d);
    }
</style>
