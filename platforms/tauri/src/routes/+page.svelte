<script lang="ts">
    /**
     * NetBoozt v3.0 - Main Application (Modularizado)
     * By LOUST (www.loust.pro)
     */
    
    import { onMount, onDestroy } from 'svelte';
    import { invoke, listen } from '$lib/tauri-bridge';
    
    // SvelteKit page data - ignorar para evitar warnings
    export let data: Record<string, unknown> = {};
    // @ts-ignore - SvelteKit pasa params automáticamente
    let { params } = $$restProps || {};
    $: params; // evitar unused warning
    $: data; // evitar unused warning
    
    // Components
    import Toast from '$lib/components/Toast.svelte';
    import ToastNotification from '$lib/components/ToastNotification.svelte';
    import CollapsibleSidebar from '$lib/components/CollapsibleSidebar.svelte';
    import FloatingTerminal from '$lib/components/FloatingTerminal.svelte';
    
    // Stores
    import { alerts as alertStore, activeAlerts, errorAlerts } from '$lib/stores/alertStore';
    import { terminalLogs } from '$lib/stores/terminalStore';
    import { diagnosticHistory, calculateHealthScore, type DiagnosticReport, type HealthStatus } from '$lib/stores/diagnosticStore';
    import type { Alert as StoreAlert } from '$lib/stores/alertStore';
    
    // Tabs
    import { DashboardTab, AlertsTab, SettingsTab, ReportsTab, DevUtilitiesTab } from '$lib/components/tabs';
    import DocsTab from '$lib/components/DocsTab.svelte';
    import GitHubIntegration from '$lib/components/GitHubIntegration.svelte';
    import UpdaterTab from '$lib/components/UpdaterTab.svelte';
    
    type UnlistenFn = () => void;
    type TabType = 'dashboard' | 'alerts' | 'settings' | 'docs' | 'github' | 'reports' | 'devtools' | 'updater';
    
    // Interfaces
    interface NetworkAdapter { name: string; description: string; status: string; link_speed: string; mac_address: string; }
    interface DiagnosticResult { health: string; score: number; failure_point: string; adapter_ok: boolean; adapter_name: string; router_ok: boolean; router_latency_ms: number; isp_ok: boolean; isp_latency_ms: number; dns_ok: boolean; dns_latency_ms: number; recommendation: string; }
    interface TcpSettings { autotuning: string; rss: string; rsc: string; ecn: string; timestamps: string; chimney: string; }
    interface Alert { id: string; type: string; severity: 'info' | 'warning' | 'critical'; message: string; timestamp: string; resolved: boolean; }
    
    // ============ STATE ============
    let adapters: NetworkAdapter[] = [];
    let selectedAdapter = '';
    let diagnostic: DiagnosticResult | null = null;
    let tcpSettings: TcpSettings | null = null;
    
    // UI State
    let loading = true;
    let diagnosing = false;
    let optimizing = false;
    let settingDNS = false;
    let activeTab: TabType = 'dashboard';
    let sidebarCollapsed = false;
    let showTerminal = false;
    let terminalTab: 'terminal' | 'problems' = 'terminal';
    let dryRunMode = false;
    
    // Toasts (legacy - se mantiene para compatibilidad)
    let toasts: { id: string; type: 'success' | 'error' | 'warning' | 'info'; message: string }[] = [];
    let error: string | null = null;
    
    // Metrics
    let downloadRate = 0, uploadRate = 0, latency = 0;
    let packetsSent = 0, packetsRecv = 0, totalErrors = 0, totalDrops = 0, uptime = 0;
    
    // DNS
    let currentDNS: string[] = [];
    let dnsHealthMap: Record<string, any> = {};
    let autoFailoverEnabled = false;
    let currentTier = 1;
    
    // Alerts
    let alerts: Alert[] = [];
    let alertThresholds = { latency_high: 100, packet_loss: 2, speed_low: 10, dns_failure: 3 };
    
    // Monitoring
    let monitoringActive = false;
    
    // Event listeners
    let unlistenMetrics: UnlistenFn | null = null;
    let unlistenAlerts: UnlistenFn | null = null;
    let uptimeInterval: ReturnType<typeof setInterval> | null = null;
    
    // Store alerts como toasts
    $: storeToasts = $activeAlerts.map(a => ({
        id: a.id,
        type: a.type,
        title: a.title,
        message: a.message,
        timestamp: a.timestamp,
        duration: a.duration
    }));
    
    // ============ HELPERS ============
    /**
     * Selecciona el mejor adaptador: prioriza adaptadores físicos reales
     * Excluye adaptadores virtuales (Hyper-V, VirtualBox, VMware)
     */
    function selectBestAdapter(adapters: NetworkAdapter[]): string {
        // Palabras clave de adaptadores virtuales a evitar
        const virtualKeywords = ['virtual', 'hyper-v', 'vmware', 'virtualbox', 'loopback', 'bluetooth', 'wan miniport'];
        
        // Filtrar adaptadores reales
        const realAdapters = adapters.filter(a => {
            const desc = (a.description || '').toLowerCase();
            const name = a.name.toLowerCase();
            return !virtualKeywords.some(v => desc.includes(v) || name.includes(v));
        });
        
        // Si no hay adaptadores reales, usar todos
        const candidates = realAdapters.length > 0 ? realAdapters : adapters;
        
        // Prioridad: 1) Wi-Fi activo con buena señal, 2) Ethernet físico, 3) primero disponible
        // Nota: En laptops, Wi-Fi suele ser el principal
        const wifiAdapter = candidates.find(a => 
            a.name.toLowerCase().includes('wi-fi') || 
            a.name.toLowerCase().includes('wifi') ||
            a.description?.toLowerCase().includes('wireless')
        );
        
        // Ethernet físico (no "Ethernet 2" que suele ser virtual)
        const ethernetAdapter = candidates.find(a => {
            const name = a.name.toLowerCase();
            const desc = (a.description || '').toLowerCase();
            return (name === 'ethernet' || desc.includes('realtek') || desc.includes('intel')) &&
                   !name.includes('ethernet 2') && !name.includes('ethernet 3');
        });
        
        // Si hay Ethernet físico con Realtek/Intel, usarlo
        if (ethernetAdapter) return ethernetAdapter.name;
        
        // Si hay Wi-Fi, usarlo
        if (wifiAdapter) return wifiAdapter.name;
        
        // Último recurso: primer adaptador
        return candidates[0]?.name || adapters[0]?.name || '';
    }
    
    // ============ LIFECYCLE ============
    onMount(async () => {
        await loadInitialData();
        setupEventListeners();
        startUptimeCounter();
    });
    
    onDestroy(() => {
        if (unlistenMetrics) unlistenMetrics();
        if (unlistenAlerts) unlistenAlerts();
        if (uptimeInterval) clearInterval(uptimeInterval);
    });
    
    // ============ DATA LOADING ============
    async function loadInitialData() {
        try {
            loading = true;
            error = null;
            const [adapterResult, settingsResult] = await Promise.all([
                invoke<NetworkAdapter[]>('get_network_adapters'),
                invoke<TcpSettings>('get_current_settings')
            ]);
            adapters = adapterResult;
            tcpSettings = settingsResult;
            if (adapters.length > 0) {
                // Priorizar Ethernet sobre Wi-Fi (más estable)
                selectedAdapter = selectBestAdapter(adapters);
                // Auto-iniciar monitoreo
                await startMonitoringAuto();
            }
        } catch (e) {
            error = `Error cargando datos: ${e}`;
        } finally {
            loading = false;
        }
    }
    
    // Auto-iniciar monitoreo al cargar la app
    async function startMonitoringAuto() {
        if (!selectedAdapter) return;
        
        // Si ya estamos en estado activo localmente, no hacer nada
        if (monitoringActive) return;
        
        try {
            // Primero intentamos detener cualquier monitoreo previo (limpieza)
            try {
                await invoke('stop_monitoring');
            } catch { /* ignorar si no había monitoreo */ }
            
            // Ahora iniciamos fresco
            await invoke('start_monitoring', { adapter: selectedAdapter, intervalMs: 1000 });
            monitoringActive = true;
            console.log('📊 Monitoreo automático iniciado para:', selectedAdapter);
        } catch (e) {
            console.warn('No se pudo iniciar monitoreo automático:', e);
            // Si falló por alguna razón, asegurar estado consistente
            monitoringActive = false;
        }
    }
    
    async function setupEventListeners() {
        try {
            unlistenMetrics = await listen<any>('metrics_update', (event) => {
                const m = event.payload;
                downloadRate = m.download_mbps || 0;
                uploadRate = m.upload_mbps || 0;
                latency = m.latency_ms || 0;
                packetsSent = m.packets_sent_per_sec || 0;
                packetsRecv = m.packets_recv_per_sec || 0;
                totalErrors = (m.errors_in || 0) + (m.errors_out || 0);
                totalDrops = (m.drops_in || 0) + (m.drops_out || 0);
            });
            unlistenAlerts = await listen<Alert>('alert_triggered', (event) => {
                alerts = [event.payload, ...alerts.slice(0, 49)];
                showToast('warning', event.payload.message);
            });
        } catch (e) { console.error('Event listeners error:', e); }
    }
    
    function startUptimeCounter() {
        uptimeInterval = setInterval(() => { if (monitoringActive) uptime++; }, 1000);
    }
    
    // ============ ACTIONS ============
    async function runDiagnostic() {
        try {
            diagnosing = true;
            error = null;
            diagnostic = await invoke<DiagnosticResult>('run_full_diagnostic');
            
            // Guardar en historial
            if (diagnostic) {
                // Usar el score que viene del backend (más preciso)
                const backendScore = diagnostic.score ?? 0;
                const healthFromBackend = diagnostic.health?.toLowerCase() || 'unknown';
                
                // Mapear health del backend a HealthStatus
                const statusMap: Record<string, HealthStatus> = {
                    'excellent': 'excellent',
                    'good': 'good', 
                    'fair': 'fair',
                    'poor': 'poor',
                    'bad': 'critical',
                    'down': 'critical',
                };
                const status: HealthStatus = statusMap[healthFromBackend] || 'unknown';
                
                const report: Omit<DiagnosticReport, 'id' | 'timestamp' | 'appVersion'> = {
                    type: 'full',
                    health: status,
                    healthScore: Math.round(backendScore),
                    failurePoint: (diagnostic.failure_point as 'none' | 'adapter' | 'router' | 'isp' | 'dns') || 'none',
                    adapterOk: diagnostic.adapter_ok ?? true,
                    adapterName: diagnostic.adapter_name || selectedAdapter || 'Unknown',
                    routerOk: diagnostic.router_ok ?? true,
                    routerLatencyMs: diagnostic.router_latency_ms || 0,
                    ispOk: diagnostic.isp_ok ?? true,
                    ispLatencyMs: diagnostic.isp_latency_ms || 0,
                    dnsOk: diagnostic.dns_ok ?? true,
                    dnsLatencyMs: diagnostic.dns_latency_ms || 0,
                    recommendation: diagnostic.recommendation || '',
                    selectedAdapter: selectedAdapter,
                    dnsServers: currentDNS,
                };
                diagnosticHistory.add(report);
                showToast('success', 'Diagnóstico guardado en historial');
            }
        }
        catch (e) { error = `Error en diagnóstico: ${e}`; }
        finally { diagnosing = false; }
    }
    
    async function setDNS(event: CustomEvent<any>) {
        const provider = event.detail;
        if (!selectedAdapter) { showToast('error', 'No hay adaptador seleccionado'); return; }
        try {
            settingDNS = true;
            if (dryRunMode) {
                showToast('info', `[DRY-RUN] DNS → ${provider.name}`);
                logDryRun('DNS', [
                    `Set-DnsClientServerAddress -InterfaceAlias "${selectedAdapter}" -ServerAddresses ${provider.primary},${provider.secondary}`,
                    `Clear-DnsClientCache`
                ]);
            } else {
                await invoke('set_dns_servers', { adapter: selectedAdapter, primary: provider.primary, secondary: provider.secondary });
                await invoke('flush_dns_cache');
                currentDNS = [provider.primary, provider.secondary];
                currentTier = provider.tier;
                showToast('success', `DNS cambiado a ${provider.name}`);
            }
        } catch (e) { showToast('error', `Error: ${e}`); }
        finally { settingDNS = false; }
    }
    
    async function resetDNS() {
        if (!selectedAdapter) return;
        try {
            settingDNS = true;
            if (dryRunMode) {
                showToast('info', '[DRY-RUN] Reset DNS → DHCP');
                logDryRun('DNS Reset', [
                    `Set-DnsClientServerAddress -InterfaceAlias "${selectedAdapter}" -ResetServerAddresses`
                ]);
            } else {
                await invoke('reset_dns_to_dhcp', { adapter: selectedAdapter });
                currentDNS = []; currentTier = 7;
                showToast('success', 'DNS reseteado');
            }
        } catch (e) { showToast('error', `Error: ${e}`); }
        finally { settingDNS = false; }
    }
    
    async function flushDNS() {
        try { await invoke('flush_dns_cache'); showToast('success', 'Caché DNS limpiado'); }
        catch (e) { showToast('error', `Error: ${e}`); }
    }
    
    async function applyProfile(event: CustomEvent<string>) {
        const profile = event.detail as 'Conservative' | 'Balanced' | 'Aggressive';
        try {
            optimizing = true;
            if (dryRunMode) {
                showToast('info', `[DRY-RUN] Perfil ${profile}`);
                logDryRunProfile(profile);
            } else {
                const applied = await invoke<string[]>('apply_profile', { profile });
                await loadInitialData();
                showToast('success', `Perfil ${profile}: ${applied.length} optimizaciones`);
            }
        } catch (e) { showToast('error', `Error: ${e}`); }
        finally { optimizing = false; }
    }
    
    async function resetOptimizations() {
        try {
            optimizing = true;
            if (dryRunMode) {
                showToast('info', '[DRY-RUN] Reset a defaults');
                logDryRun('Reset Optimizations', [
                    'netsh int tcp set global autotuninglevel=normal',
                    'netsh int tcp set global rss=enabled',
                    'netsh int tcp set global rsc=enabled',
                    'netsh int tcp set global ecncapability=disabled',
                    'netsh int tcp set global timestamps=disabled',
                ]);
            } else {
                const reset = await invoke<string[]>('reset_to_defaults');
                await loadInitialData();
                showToast('success', `Reseteado: ${reset.length} cambios`);
            }
        } catch (e) { showToast('error', `Error: ${e}`); }
        finally { optimizing = false; }
    }
    
    async function toggleMonitoring() {
        try {
            if (monitoringActive) {
                await invoke('stop_monitoring');
                monitoringActive = false; uptime = 0;
                showToast('info', 'Monitoreo detenido');
            } else {
                // Intentar detener cualquier monitoreo previo antes de iniciar
                try { await invoke('stop_monitoring'); } catch { /* ignorar */ }
                
                await invoke('start_monitoring', { adapter: selectedAdapter, intervalMs: 1000 });
                monitoringActive = true;
                showToast('success', 'Monitoreo iniciado');
            }
        } catch (e: any) {
            // Si dice que ya está activo, sincronizar estado
            const errorMsg = String(e);
            if (errorMsg.includes('ya está activo') || errorMsg.includes('already')) {
                monitoringActive = true;
                showToast('info', 'Monitoreo sincronizado');
            } else {
                showToast('error', `Error: ${e}`);
            }
        }
    }
    
    // ============ HELPERS ============
    function showToast(type: 'success' | 'error' | 'warning' | 'info', message: string, title?: string) {
        // Usar el nuevo store de alertas
        alertStore[type](title || getToastTitle(type), message, 'app');
    }
    
    function getToastTitle(type: string): string {
        const titles: Record<string, string> = {
            success: 'Éxito',
            error: 'Error',
            warning: 'Advertencia',
            info: 'Información'
        };
        return titles[type] || 'Notificación';
    }
    
    function logDryRun(action: string, commands: string[]) {
        terminalLogs.dryrun(`━━━ DRY-RUN: ${action} ━━━`);
        commands.forEach(cmd => terminalLogs.dryrun(`  > ${cmd}`));
        terminalLogs.dryrun(`━━━ Fin (no se ejecutó) ━━━`);
        // Abrir terminal automáticamente
        showTerminal = true;
        terminalTab = 'terminal';
    }
    
    function logDryRunProfile(profile: string) {
        const commands: Record<string, string[]> = {
            'Conservative': [
                'netsh int tcp set global rss=enabled',
                'netsh int tcp set global rsc=enabled',
                'netsh int tcp set global autotuninglevel=normal',
            ],
            'Balanced': [
                'netsh int tcp set global rss=enabled',
                'netsh int tcp set global rsc=enabled',
                'netsh int tcp set global autotuninglevel=normal',
                'netsh int tcp set global ecncapability=enabled',
                'reg add "HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters" /v EnableHyStart /t REG_DWORD /d 1 /f',
                'reg add "HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters" /v EnablePrr /t REG_DWORD /d 1 /f',
            ],
            'Aggressive': [
                'netsh int tcp set global rss=enabled',
                'netsh int tcp set global rsc=enabled',
                'netsh int tcp set global autotuninglevel=normal',
                'netsh int tcp set global ecncapability=enabled',
                'netsh int tcp set global timestamps=enabled',
                'reg add "HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters" /v EnableHyStart /t REG_DWORD /d 1 /f',
                'reg add "HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters" /v EnablePrr /t REG_DWORD /d 1 /f',
                'reg add "HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters" /v EnableTFO /t REG_DWORD /d 1 /f',
                'reg add "HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters" /v EnableWsd /t REG_DWORD /d 0 /f',
                'reg add "HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters" /v TcpInitialRto /t REG_DWORD /d 1000 /f',
            ],
        };
        logDryRun(`Perfil ${profile}`, commands[profile] || []);
    }
    
    function handleDismissToast(e: CustomEvent<string>) {
        alertStore.dismiss(e.detail);
    }
    
    function openProblemsPanel() {
        showTerminal = true;
        terminalTab = 'problems';
    }
    
    async function openCLIManager() {
        // Mostrar mensaje en terminal flotante
        terminalLogs.info('🖥️ Abriendo CLI Manager en ventana externa...');
        showTerminal = true;
        terminalTab = 'terminal';
        
        try {
            // Usar comando Rust para abrir CLI con elevación
            const result = await invoke<string>('open_cli_manager');
            
            terminalLogs.info('✅ ' + result);
            if (result.toLowerCase().includes('windows-only')) {
                terminalLogs.info('💡 El CLI legado es exclusivo de Windows; en Linux se abrió una terminal del sistema para seguir trabajando con NetBoozt');
            } else {
                terminalLogs.info('💡 El CLI Manager requiere permisos de administrador para algunas operaciones');
            }
            
            showToast('success', result);
        } catch (e) {
            terminalLogs.error(`❌ Error abriendo CLI: ${e}`);
            showToast('error', `Error abriendo CLI Manager: ${e}`);
        }
    }

    function handleTabChange(e: CustomEvent<TabType>) { activeTab = e.detail; }
    
    const tabTitles: Record<TabType, string> = {
        dashboard: 'Dashboard',
        alerts: 'Sistema de Alertas', settings: 'Configuración',
        docs: 'Documentación', github: 'GitHub', reports: 'Reportes de Diagnóstico', devtools: 'Dev Utilities',
        updater: 'Actualizaciones'
    };
    
    const tabSubtitles: Record<TabType, string> = {
        dashboard: 'Métricas de red en tiempo real',
        alerts: 'Notificaciones y advertencias del sistema',
        settings: 'Personaliza tu experiencia',
        docs: 'Guías y documentación técnica',
        github: 'Repositorio, versiones y comunidad',
        reports: 'Historial de diagnósticos y análisis',
        devtools: 'Herramientas de desarrollo y testing',
        updater: 'Verifica y descarga nuevas versiones'
    };
</script>

<svelte:head><title>NetBoozt - Network Optimizer</title></svelte:head>

<div class="app">
    <!-- Toast Notifications (nuevo sistema) -->
    <ToastNotification 
        toasts={storeToasts} 
        on:dismiss={handleDismissToast} 
    />
    
    <!-- Sidebar -->
    <CollapsibleSidebar 
        {activeTab} 
        collapsed={sidebarCollapsed} 
        {monitoringActive} 
        {uptime} 
        {alerts}
        on:tabChange={handleTabChange}
        on:toggleCollapse={() => sidebarCollapsed = !sidebarCollapsed}
        on:openProblems={openProblemsPanel}
        on:openCLI={openCLIManager}
    />
    
    <!-- Main -->
    <main class="main">
        <header class="header">
            <div class="header-title">
                <h1>{tabTitles[activeTab]}</h1>
                <span class="header-subtitle">{tabSubtitles[activeTab]}</span>
            </div>
            <div class="header-actions">
                {#if selectedAdapter}<span class="adapter-badge">📡 {selectedAdapter}</span>{/if}
                {#if dryRunMode}<span class="dryrun-badge">🧪 Dry-Run</span>{/if}
                <button class="btn btn-ghost" on:click={loadInitialData}>🔄 Actualizar</button>
            </div>
        </header>
        
        {#if error}
            <div class="alert alert-error"><span>⚠️ {error}</span><button on:click={() => error = null}>✕</button></div>
        {/if}
        
        <div class="content">
            {#if activeTab === 'dashboard'}
                <DashboardTab 
                    {loading} 
                    {adapters} 
                    {selectedAdapter} 
                    {diagnostic} 
                    {diagnosing} 
                    {downloadRate} 
                    {uploadRate} 
                    {latency} 
                    {uptime}
                    {packetsSent}
                    {packetsRecv}
                    {totalErrors}
                    {totalDrops}
                    {monitoringActive}
                    on:diagnose={runDiagnostic}
                    on:toggleMonitoring={toggleMonitoring}
                />
            {:else if activeTab === 'alerts'}
                <AlertsTab {loading} {alerts} bind:alertThresholds />
            {:else if activeTab === 'settings'}
                <SettingsTab />
            {:else if activeTab === 'docs'}
                <DocsTab />
            {:else if activeTab === 'github'}
                <GitHubIntegration on:navigate={(e) => {
                    activeTab = e.detail.tab;
                }} />
            {:else if activeTab === 'updater'}
                <UpdaterTab />
            {:else if activeTab === 'reports'}
                <ReportsTab />
            {:else if activeTab === 'devtools'}
                <DevUtilitiesTab />
            {/if}
        </div>
    </main>
    
    <!-- Floating Terminal (z-index superior, no mueve el layout) -->
    <FloatingTerminal 
        show={showTerminal} 
        sidebarWidth={sidebarCollapsed ? 60 : 220}
        activeTab={terminalTab}
        on:toggle={() => showTerminal = !showTerminal}
        on:tabChange={(e) => terminalTab = e.detail}
    />
</div>

<style>
    .app { display: flex; height: 100vh; overflow: hidden; background: var(--bg-main, #0a0a0a); }
    .main { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
    .header { display: flex; justify-content: space-between; align-items: center; padding: 1rem 1.5rem; background: var(--bg-card, #1a1a1a); border-bottom: 1px solid var(--border, #2d2d2d); }
    .header-title { display: flex; flex-direction: column; gap: 0.15rem; }
    .header h1 { font-size: 1.375rem; font-weight: 600; color: var(--text-primary, #fff); margin: 0; }
    .header-subtitle { font-size: 0.7rem; color: var(--text-muted, #666); }
    .header-actions { display: flex; align-items: center; gap: 0.75rem; }
    .adapter-badge { font-size: 0.75rem; color: var(--primary, #00d4aa); background: rgba(0, 212, 170, 0.1); padding: 0.35rem 0.75rem; border-radius: 20px; border: 1px solid rgba(0, 212, 170, 0.3); }
    .dryrun-badge { font-size: 0.75rem; color: var(--warning, #fdcb6e); background: rgba(253, 203, 110, 0.1); padding: 0.35rem 0.75rem; border-radius: 20px; border: 1px solid rgba(253, 203, 110, 0.3); }
    .content { flex: 1; padding: 1.5rem; overflow-y: auto; }
    .btn { display: inline-flex; align-items: center; gap: 0.5rem; padding: 0.625rem 1rem; border-radius: 8px; font-size: 0.875rem; font-weight: 500; cursor: pointer; border: none; transition: all 0.15s; }
    .btn-ghost { background: transparent; color: var(--text-secondary, #a0a0a0); }
    .btn-ghost:hover { background: var(--bg-elevated, #2b2b2b); color: var(--text-primary, #fff); }
    .alert { display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1rem; border-radius: 8px; margin: 0 1.5rem 1rem; }
    .alert-error { background: rgba(255, 107, 107, 0.15); border: 1px solid var(--error, #ff6b6b); color: var(--error, #ff6b6b); }
    .alert button { background: none; border: none; color: inherit; cursor: pointer; padding: 0.25rem; }
</style>
