<script lang="ts">
    /**
     * NetBoozt - Network Diagnostics (5 Phases)
     * Componente principal que orquesta el diagnóstico modular
     * 
     * Ponderación del score por fase:
     * - Adaptador (1.0) = 100% controlable por usuario
     * - Router (0.9) = 90% controlable  
     * - ISP (0.3) = NO depende del usuario
     * - DNS (0.7) = 70% configurable
     * - Speed (0.2) = Informativo, depende del ISP
     * 
     * By LOUST (www.loust.pro)
     */
    
    import { onMount, createEventDispatcher } from 'svelte';
    import { invoke } from '$lib/tauri-bridge';
    import Icon from './Icon.svelte';
    import Tooltip from './Tooltip.svelte';
    
    // Componentes de fase modularizados
    import PhaseAdapter from './diagnostics/PhaseAdapter.svelte';
    import PhaseRouter from './diagnostics/PhaseRouter.svelte';
    import PhaseISP from './diagnostics/PhaseISP.svelte';
    import PhaseDNS from './diagnostics/PhaseDNS.svelte';
    import PhaseSpeed from './diagnostics/PhaseSpeed.svelte';
    
    const dispatch = createEventDispatcher<{
        toast: { type: 'success' | 'error' | 'info' | 'warning'; message: string };
    }>();
    
    interface PhaseResult {
        status: 'pending' | 'running' | 'success' | 'warning' | 'error';
        details: string[];
        latency?: number;
        recommendation?: string;
        weight: number;
        speed?: { download: number; upload: number; ping: number };
    }
    
    // Estado
    let isRunning = false;
    let currentPhase = -1;
    let results: Record<string, PhaseResult> = {};
    let finalScore: number | null = null;
    let healthStatus: string = '';
    let platform: 'windows' | 'linux' | 'other' = 'other';
    
    // Referencias a componentes
    let phaseAdapter: PhaseAdapter;
    let phaseRouter: PhaseRouter;
    let phaseISP: PhaseISP;
    let phaseDNS: PhaseDNS;
    let phaseSpeed: PhaseSpeed;

    onMount(() => {
        const ua = navigator.userAgent.toLowerCase();
        if (ua.includes('windows')) {
            platform = 'windows';
        } else if (ua.includes('linux')) {
            platform = 'linux';
        }
    });

    $: toolsTitle = platform === 'windows' ? 'Herramientas de Windows' : 'Herramientas del sistema';
    $: troubleshooterLabel = platform === 'windows' ? 'Troubleshooter' : 'Asistente de red';
    $: networkLabel = platform === 'windows' ? 'Conexiones' : 'Red';
    $: deviceLabel = platform === 'windows' ? 'Dispositivos' : 'Ajustes';
    $: summaryLabel = platform === 'windows' ? 'ipconfig /all' : 'Resumen IP';
    $: terminalLabel = platform === 'windows' ? 'CMD' : 'Terminal';
    
    async function runDiagnostics() {
        if (isRunning) return;
        
        isRunning = true;
        currentPhase = -1;
        results = {};
        finalScore = null;
        healthStatus = '';
        
        try {
            // Fase 1: Adaptador
            currentPhase = 0;
            results['adapter'] = await phaseAdapter.run();
            await sleep(300);
            
            // Fase 2: Router
            if (results['adapter'].status === 'success' || results['adapter'].status === 'warning') {
                currentPhase = 1;
                results['router'] = await phaseRouter.run();
                await sleep(300);
            }
            
            // Fase 3: ISP
            if (results['router']?.status === 'success' || results['router']?.status === 'warning') {
                currentPhase = 2;
                results['isp'] = await phaseISP.run();
                await sleep(300);
            }
            
            // Fase 4: DNS
            if (results['isp']?.status === 'success' || results['isp']?.status === 'warning') {
                currentPhase = 3;
                results['dns'] = await phaseDNS.run();
                await sleep(300);
            }
            
            // Fase 5: Speed Test
            if (results['dns']?.status === 'success' || results['dns']?.status === 'warning') {
                currentPhase = 4;
                results['speed'] = await phaseSpeed.run();
            }
            
            // Calcular score final
            calculateScore();
            currentPhase = 5;
            
        } catch (e) {
            dispatch('toast', { type: 'error', message: `Error: ${e}` });
        } finally {
            isRunning = false;
        }
    }
    
    function calculateScore() {
        let totalWeight = 0;
        let weightedScore = 0;
        
        Object.values(results).forEach(r => {
            if (!r) return;
            
            const weight = r.weight;
            totalWeight += weight;
            
            let phaseScore = 0;
            switch (r.status) {
                case 'success': phaseScore = 100; break;
                case 'warning': phaseScore = 70; break;
                case 'error': phaseScore = 0; break;
                default: phaseScore = 50;
            }
            
            if (r.latency && r.latency > 100) {
                phaseScore -= Math.min(20, (r.latency - 100) / 10);
            }
            
            weightedScore += phaseScore * weight;
        });
        
        finalScore = totalWeight > 0 ? Math.round(weightedScore / totalWeight) : 0;
        
        if (finalScore >= 90) healthStatus = 'Excelente';
        else if (finalScore >= 75) healthStatus = 'Bueno';
        else if (finalScore >= 50) healthStatus = 'Regular';
        else if (finalScore >= 25) healthStatus = 'Malo';
        else healthStatus = 'Crítico';
    }
    
    function sleep(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
    
    function getScoreColor(score: number): string {
        if (score >= 90) return '#00ff88';
        if (score >= 75) return '#00d4aa';
        if (score >= 50) return '#ffc107';
        if (score >= 25) return '#ff9800';
        return '#ff4444';
    }
    
    async function openSystemTool(tool: string) {
        try {
            switch (tool) {
                case 'troubleshooter':
                    dispatch('toast', { type: 'info', message: await invoke<string>('run_windows_network_troubleshooter') });
                    break;
                case 'network':
                    dispatch('toast', { type: 'info', message: await invoke<string>('open_system_tool', { tool: 'network_connections' }) });
                    break;
                case 'device':
                    dispatch('toast', { type: 'info', message: await invoke<string>('open_system_tool', { tool: 'device_settings' }) });
                    break;
                case 'terminal':
                    dispatch('toast', { type: 'info', message: await invoke<string>('open_system_tool', { tool: 'terminal' }) });
                    break;
                case 'network_snapshot':
                    dispatch('toast', { type: 'info', message: await invoke<string>('open_system_tool', { tool: 'network_snapshot' }) });
                    break;
            }
        } catch (e) {
            dispatch('toast', { type: 'error', message: `Error: ${e}` });
        }
    }
</script>

<div class="diagnostics-container">
    <!-- Grid Background -->
    <div class="grid-bg"></div>
    
    <!-- Header -->
    <div class="diag-header">
        <div class="title-row">
            <Icon name="activity" size={24} />
            <h2>Diagnóstico de Red</h2>
        </div>
        <p class="subtitle">Análisis de 5 fases con ponderación inteligente</p>
    </div>
    
    <!-- Start Button + Score -->
    <div class="start-section">
        <button 
            class="start-btn" 
            class:running={isRunning}
            on:click={runDiagnostics}
            disabled={isRunning}
        >
            {#if isRunning}
                <div class="spinner"></div>
                <span>Fase {currentPhase + 1} de 5...</span>
            {:else}
                <Icon name="play" size={20} />
                <span>Ejecutar Diagnóstico</span>
            {/if}
        </button>
        
        {#if finalScore !== null}
            <div class="score-badge" style="--score-color: {getScoreColor(finalScore)}">
                <span class="score-num">{finalScore}</span>
                <span class="score-label">{healthStatus}</span>
            </div>
        {/if}
    </div>
    
    <!-- Score Explanation -->
    {#if finalScore !== null}
        <div class="score-explanation">
            <Icon name="info" size={14} />
            <span>
                El score pondera más las fases que <strong>tú controlas</strong> 
                (Adaptador, Router, DNS) y menos las que dependen del ISP.
            </span>
        </div>
    {/if}
    
    <!-- Phases Grid -->
    <div class="phases-grid">
        <PhaseAdapter 
            bind:this={phaseAdapter}
            isActive={currentPhase === 0}
            result={results['adapter']}
        />
        
        <PhaseRouter 
            bind:this={phaseRouter}
            isActive={currentPhase === 1}
            result={results['router']}
        />
        
        <PhaseISP 
            bind:this={phaseISP}
            isActive={currentPhase === 2}
            result={results['isp']}
        />
        
        <PhaseDNS 
            bind:this={phaseDNS}
            isActive={currentPhase === 3}
            result={results['dns']}
        />
        
        <PhaseSpeed 
            bind:this={phaseSpeed}
            isActive={currentPhase === 4}
            result={results['speed']}
        />
    </div>
    
    <!-- Weight Legend -->
    <div class="weight-legend">
        <span class="legend-title">Ponderación:</span>
        <div class="legend-items">
            <span class="legend-item user">
                <span class="dot"></span>
                Tu equipo (100%)
            </span>
            <span class="legend-item config">
                <span class="dot"></span>
                Configurable (70%)
            </span>
            <span class="legend-item external">
                <span class="dot"></span>
                ISP/Externo (20-30%)
            </span>
        </div>
    </div>
    
    <!-- Windows Tools -->
    <div class="tools-section">
        <span class="tools-title">
            <Icon name="settings" size={14} />
            {toolsTitle}
        </span>
        <div class="tools-grid">
            <Tooltip text={platform === 'windows' ? 'Solucionador de problemas de red' : 'Abrir la herramienta de ayuda o ajustes de red del sistema'}>
                <button class="tool-btn" on:click={() => openSystemTool('troubleshooter')}>
                    <Icon name="settings" size={14} />
                    {troubleshooterLabel}
                </button>
            </Tooltip>
            <Tooltip text={platform === 'windows' ? 'Panel de conexiones de red' : 'Abrir conexiones o ajustes de red'}>
                <button class="tool-btn" on:click={() => openSystemTool('network')}>
                    <Icon name="wifi" size={14} />
                    {networkLabel}
                </button>
            </Tooltip>
            <Tooltip text={platform === 'windows' ? 'Administrador de dispositivos' : 'Abrir ajustes del sistema'}>
                <button class="tool-btn" on:click={() => openSystemTool('device')}>
                    <Icon name="cpu" size={14} />
                    {deviceLabel}
                </button>
            </Tooltip>
            <Tooltip text={platform === 'windows' ? 'Ver configuración IP' : 'Abrir un resumen actual de IP, rutas y DNS'}>
                <button class="tool-btn" on:click={() => openSystemTool('network_snapshot')}>
                    <Icon name="terminal" size={14} />
                    {summaryLabel}
                </button>
            </Tooltip>
            <Tooltip text={platform === 'windows' ? 'Abrir CMD' : 'Abrir una terminal del sistema'}>
                <button class="tool-btn" on:click={() => openSystemTool('terminal')}>
                    <Icon name="terminal" size={14} />
                    {terminalLabel}
                </button>
            </Tooltip>
        </div>
    </div>
</div>

<style>
    .diagnostics-container {
        position: relative;
        padding: 1.5rem;
        overflow: hidden;
    }
    
    .grid-bg {
        position: absolute;
        inset: 0;
        background-image: 
            linear-gradient(rgba(0, 212, 170, 0.03) 1px, transparent 1px),
            linear-gradient(90deg, rgba(0, 212, 170, 0.03) 1px, transparent 1px);
        background-size: 25px 25px;
        pointer-events: none;
        z-index: 0;
    }
    
    .diag-header {
        position: relative;
        z-index: 1;
        text-align: center;
        margin-bottom: 1.5rem;
    }
    
    .title-row {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        margin-bottom: 0.25rem;
    }
    
    .diag-header h2 {
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        color: var(--text-primary, #fff);
    }
    
    .subtitle {
        font-size: 0.8rem;
        color: var(--text-muted, #888);
        margin: 0;
    }
    
    .start-section {
        position: relative;
        z-index: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 1.5rem;
        margin-bottom: 1.5rem;
    }
    
    .start-btn {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        padding: 0.9rem 2rem;
        background: linear-gradient(135deg, var(--primary, #00d4aa), #00a896);
        border: none;
        border-radius: 50px;
        color: #000;
        font-size: 0.95rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s ease;
    }
    
    .start-btn:hover:not(:disabled) {
        transform: scale(1.03);
        box-shadow: 0 0 25px rgba(0, 212, 170, 0.4);
    }
    
    .start-btn:disabled {
        opacity: 0.9;
        cursor: wait;
    }
    
    .start-btn.running {
        background: linear-gradient(135deg, #2d2d2d, #3d3d3d);
        color: var(--primary, #00d4aa);
    }
    
    .spinner {
        width: 18px;
        height: 18px;
        border: 2px solid var(--border, #3d3d3d);
        border-top-color: var(--primary, #00d4aa);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }
    
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
    
    .score-badge {
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 0.75rem 1.25rem;
        background: rgba(0, 0, 0, 0.3);
        border: 2px solid var(--score-color, #00d4aa);
        border-radius: 12px;
    }
    
    .score-num {
        font-size: 1.75rem;
        font-weight: 700;
        color: var(--score-color, #00d4aa);
        line-height: 1;
    }
    
    .score-label {
        font-size: 0.7rem;
        color: var(--text-muted, #888);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    
    .score-explanation {
        position: relative;
        z-index: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.75rem 1rem;
        background: rgba(0, 212, 170, 0.05);
        border: 1px solid rgba(0, 212, 170, 0.2);
        border-radius: 8px;
        margin-bottom: 1.5rem;
        font-size: 0.75rem;
        color: var(--text-secondary, #a0a0a0);
        text-align: center;
    }
    
    .score-explanation strong {
        color: var(--primary, #00d4aa);
    }
    
    .phases-grid {
        position: relative;
        z-index: 1;
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
        gap: 1rem;
        margin-bottom: 1.5rem;
    }
    
    .weight-legend {
        position: relative;
        z-index: 1;
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 0.75rem 1rem;
        background: rgba(30, 30, 30, 0.5);
        border-radius: 8px;
        margin-bottom: 1.5rem;
        flex-wrap: wrap;
    }
    
    .legend-title {
        font-size: 0.75rem;
        color: var(--text-muted, #888);
    }
    
    .legend-items {
        display: flex;
        gap: 1rem;
        flex-wrap: wrap;
    }
    
    .legend-item {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.7rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .legend-item .dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
    }
    
    .legend-item.user .dot { background: #00d4aa; }
    .legend-item.config .dot { background: #a855f7; }
    .legend-item.external .dot { background: #ff9800; }
    
    .tools-section {
        position: relative;
        z-index: 1;
        padding-top: 1rem;
        border-top: 1px solid var(--border, #3d3d3d);
    }
    
    .tools-title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.8rem;
        color: var(--text-muted, #888);
        margin-bottom: 0.75rem;
    }
    
    .tools-grid {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }
    
    .tool-btn {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.5rem 0.75rem;
        background: var(--bg-elevated, #2b2b2b);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 6px;
        color: var(--text-secondary, #a0a0a0);
        font-size: 0.75rem;
        cursor: pointer;
        transition: all 0.15s ease;
    }
    
    .tool-btn:hover {
        background: rgba(0, 212, 170, 0.1);
        border-color: var(--primary, #00d4aa);
        color: var(--text-primary, #fff);
    }
</style>
