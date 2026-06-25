<!--
    SettingsTab.svelte - Tab de Configuración Mejorada
    Mejor distribución visual, iconos de biblioteca, sin duplicados
    By LOUST
-->
<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import Icon from '../Icon.svelte';
    
    const dispatch = createEventDispatcher();
    
    // Configuraciones
    export let settings = {
        // Comportamiento general
        minimizeToTray: true,
        startWithWindows: false,
        showNotifications: true,
        soundOnAlerts: true,
        
        // Monitoreo
        updateInterval: 1000,
        autoStartMonitoring: true,
        monitorPackets: true,
        monitorErrors: true,
        
        // DNS
        dnsAutoFailover: true,
        dnsCheckInterval: 30000,
        preferredDnsProvider: 'cloudflare',
        
        // Apariencia
        theme: 'dark',
        accentColor: '#00d4aa',
        backgroundImage: '',
        compactMode: false,
        animationsEnabled: true,
        glassEffect: true,
        
        // Avanzado
        debugMode: false,
        logLevel: 'info',
        cacheEnabled: true,
    };
    
    // Temas = fondos (glassmorphism)
    const BACKGROUNDS = [
        { id: 'dark', name: 'Negro puro', preview: '#0a0a0a', description: 'Fondo oscuro sin imagen' },
        { id: 'gradient-dark', name: 'Degradado oscuro', preview: 'linear-gradient(135deg, #0a0a0a, #1a1a2e)', description: 'Degradado sutil' },
        { id: 'gradient-purple', name: 'Nebula', preview: 'linear-gradient(135deg, #1a0a2e, #2d1b4e)', description: 'Tonos púrpura' },
        { id: 'gradient-ocean', name: 'Océano', preview: 'linear-gradient(135deg, #0a1a2e, #0d2d4d)', description: 'Tonos azules' },
        { id: 'custom', name: 'Personalizado', preview: 'url()', description: 'Usa tu propia imagen' },
    ];
    
    // Colores de acento
    const ACCENT_COLORS = [
        { color: '#00d4aa', name: 'NetBoozt' },
        { color: '#3b82f6', name: 'Azul' },
        { color: '#8b5cf6', name: 'Púrpura' },
        { color: '#f59e0b', name: 'Naranja' },
        { color: '#ef4444', name: 'Rojo' },
        { color: '#ec4899', name: 'Rosa' },
        { color: '#06b6d4', name: 'Cyan' },
        { color: '#84cc16', name: 'Lima' },
    ];
    
    // DNS Providers
    const DNS_PROVIDERS = [
        { id: 'cloudflare', name: 'Cloudflare', ip: '1.1.1.1', icon: '⚡' },
        { id: 'google', name: 'Google', ip: '8.8.8.8', icon: '🔍' },
        { id: 'quad9', name: 'Quad9', ip: '9.9.9.9', icon: '🛡️' },
        { id: 'opendns', name: 'OpenDNS', ip: '208.67.222.222', icon: '🔒' },
        { id: 'adguard', name: 'AdGuard', ip: '94.140.14.14', icon: '🚫' },
    ];
    
    const STORAGE_KEY = 'netboozt_settings';
    
    // Referencia para input de imagen custom
    let customBgInput: HTMLInputElement;
    
    onMount(() => {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved) {
            try {
                settings = { ...settings, ...JSON.parse(saved) };
            } catch (e) { console.warn('Error loading settings:', e); }
        }
        applyBackground(settings.theme);
        applyAccentColor(settings.accentColor);
    });
    
    function saveSettings() {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
        dispatch('settingsChanged', settings);
    }
    
    function applyBackground(themeId: string) {
        const body = document.body;
        
        // Limpiar estilos previos
        body.style.backgroundImage = '';
        body.style.backgroundColor = '';
        
        switch (themeId) {
            case 'dark':
                body.style.backgroundColor = '#0a0a0a';
                break;
            case 'gradient-dark':
                body.style.background = 'linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #0a0a0a 100%)';
                break;
            case 'gradient-purple':
                body.style.background = 'linear-gradient(135deg, #0a0a0a 0%, #1a0a2e 30%, #2d1b4e 60%, #0a0a0a 100%)';
                break;
            case 'gradient-ocean':
                body.style.background = 'linear-gradient(135deg, #0a0a0a 0%, #0a1a2e 30%, #0d2d4d 60%, #0a0a0a 100%)';
                break;
            case 'custom':
                if (settings.backgroundImage) {
                    body.style.backgroundImage = `url(${settings.backgroundImage})`;
                    body.style.backgroundSize = 'cover';
                    body.style.backgroundPosition = 'center';
                    body.style.backgroundAttachment = 'fixed';
                }
                break;
        }
        
        settings.theme = themeId;
        document.documentElement.setAttribute('data-theme', themeId);
        saveSettings();
    }
    
    function applyAccentColor(color: string) {
        document.documentElement.style.setProperty('--primary', color);
        document.documentElement.style.setProperty('--accent', color);
        // Actualizar colores derivados
        document.documentElement.style.setProperty('--primary-hover', adjustColor(color, -20));
        saveSettings();
    }
    
    function adjustColor(hex: string, percent: number): string {
        const num = parseInt(hex.replace('#', ''), 16);
        const amt = Math.round(2.55 * percent);
        const R = Math.max(0, Math.min(255, (num >> 16) + amt));
        const G = Math.max(0, Math.min(255, ((num >> 8) & 0x00FF) + amt));
        const B = Math.max(0, Math.min(255, (num & 0x0000FF) + amt));
        return `#${((1 << 24) | (R << 16) | (G << 8) | B).toString(16).slice(1)}`;
    }
    
    async function handleCustomBackground(event: Event) {
        const file = (event.target as HTMLInputElement).files?.[0];
        if (!file) return;
        
        // Verificar tipo
        if (!file.type.startsWith('image/')) {
            dispatch('toast', { type: 'error', message: 'Solo se permiten imágenes' });
            return;
        }
        
        // Verificar tamaño (max 5MB)
        if (file.size > 5 * 1024 * 1024) {
            dispatch('toast', { type: 'error', message: 'Imagen muy grande (máx 5MB)' });
            return;
        }
        
        // Convertir a base64 y guardar
        const reader = new FileReader();
        reader.onload = (e) => {
            settings.backgroundImage = e.target?.result as string;
            settings.theme = 'custom';
            applyBackground('custom');
            dispatch('toast', { type: 'success', message: 'Fondo personalizado aplicado' });
        };
        reader.readAsDataURL(file);
    }
    
    function resetToDefaults() {
        if (confirm('¿Restaurar configuraciones a valores por defecto?')) {
            localStorage.removeItem(STORAGE_KEY);
            location.reload();
        }
    }
    
    function exportSettings() {
        const blob = new Blob([JSON.stringify(settings, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url; a.download = 'netboozt-settings.json'; a.click();
        URL.revokeObjectURL(url);
    }
    
    function importSettings(event: Event) {
        const file = (event.target as HTMLInputElement).files?.[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = (e) => {
            try {
                settings = { ...settings, ...JSON.parse(e.target?.result as string) };
                saveSettings();
                dispatch('toast', { type: 'success', message: 'Configuración importada' });
            } catch {
                dispatch('toast', { type: 'error', message: 'Error al importar' });
            }
        };
        reader.readAsText(file);
    }
    
    $: if (settings) saveSettings();
</script>

<div class="settings-page">
    <!-- Grid principal 2 columnas -->
    <div class="settings-layout">
        <!-- Columna izquierda -->
        <div class="settings-column">
            <!-- Apariencia -->
            <section class="settings-card full-height">
                <div class="card-header">
                    <Icon name="palette" size={18} />
                    <h2>Apariencia</h2>
                </div>
                
                <!-- Fondos/Temas -->
                <div class="setting-group">
                    <span class="setting-label">
                        <Icon name="image" size={12} />
                        Fondo
                    </span>
                    <div class="background-grid">
                        {#each BACKGROUNDS as bg}
                            {#if bg.id !== 'custom'}
                                <button 
                                    class="bg-option" 
                                    class:active={settings.theme === bg.id}
                                    style="--preview: {bg.preview}"
                                    on:click={() => applyBackground(bg.id)}
                                    title={bg.description}
                                >
                                    <span class="bg-preview"></span>
                                    <span class="bg-name">{bg.name}</span>
                                </button>
                            {:else}
                                <button 
                                    class="bg-option custom-bg" 
                                    class:active={settings.theme === 'custom'}
                                    on:click={() => customBgInput?.click()}
                                    title="Subir imagen personalizada"
                                >
                                    <span class="bg-preview custom">
                                        <Icon name="upload" size={16} />
                                    </span>
                                    <span class="bg-name">Subir imagen</span>
                                </button>
                            {/if}
                        {/each}
                    </div>
                    <input 
                        bind:this={customBgInput}
                        type="file" 
                        accept="image/*" 
                        on:change={handleCustomBackground}
                        hidden 
                    />
                </div>
                
                <!-- Colores de acento -->
                <div class="setting-group">
                    <span class="setting-label">
                        <Icon name="droplet" size={12} />
                        Color de acento
                    </span>
                    <div class="color-grid">
                        {#each ACCENT_COLORS as { color, name }}
                            <button 
                                class="color-option" 
                                class:active={settings.accentColor === color}
                                style="--color: {color}"
                                title={name}
                                on:click={() => { settings.accentColor = color; applyAccentColor(color); }}
                            >
                                {#if settings.accentColor === color}
                                    <Icon name="check" size={14} color="#fff" />
                                {/if}
                            </button>
                        {/each}
                    </div>
                </div>
                
                <!-- Efectos visuales -->
                <div class="toggles-group">
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="sparkles" size={14} />
                            <span>Efecto glassmorphism</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.glassEffect} />
                    </label>
                    
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="zap" size={14} />
                            <span>Animaciones</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.animationsEnabled} />
                    </label>
                    
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="minimize-2" size={14} />
                            <span>Modo compacto</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.compactMode} />
                    </label>
                </div>
            </section>
            
            <!-- DNS -->
            <section class="settings-card">
                <div class="card-header">
                    <Icon name="globe" size={18} />
                    <h2>DNS</h2>
                </div>
                
                <div class="setting-group">
                    <span class="setting-label">
                        <Icon name="server" size={12} />
                        Proveedor preferido
                    </span>
                    <div class="dns-options">
                        {#each DNS_PROVIDERS as provider}
                            <button 
                                class="dns-option" 
                                class:active={settings.preferredDnsProvider === provider.id}
                                on:click={() => settings.preferredDnsProvider = provider.id}
                            >
                                <span class="dns-icon">{provider.icon}</span>
                                <span class="dns-name">{provider.name}</span>
                                <span class="dns-ip">{provider.ip}</span>
                            </button>
                        {/each}
                    </div>
                </div>
                
                <label class="toggle-row">
                    <div class="toggle-info">
                        <Icon name="refresh-cw" size={14} />
                        <span>Auto-failover</span>
                    </div>
                    <input type="checkbox" bind:checked={settings.dnsAutoFailover} />
                </label>
                
                <div class="setting-group">
                    <span class="setting-label">
                        <Icon name="clock" size={12} />
                        Verificar cada
                    </span>
                    <select bind:value={settings.dnsCheckInterval} class="setting-select">
                        <option value={15000}>15 segundos</option>
                        <option value={30000}>30 segundos</option>
                        <option value={60000}>1 minuto</option>
                        <option value={300000}>5 minutos</option>
                    </select>
                </div>
            </section>
        </div>
        
        <!-- Columna derecha -->
        <div class="settings-column">
            <!-- Comportamiento -->
            <section class="settings-card">
                <div class="card-header">
                    <Icon name="sliders" size={18} />
                    <h2>Comportamiento</h2>
                </div>
                
                <div class="toggles-group">
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="minus-square" size={14} />
                            <span>Minimizar a bandeja</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.minimizeToTray} />
                    </label>
                    
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="power" size={14} />
                            <span>Iniciar con Windows</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.startWithWindows} />
                    </label>
                    
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="bell" size={14} />
                            <span>Notificaciones</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.showNotifications} />
                    </label>
                    
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="volume-2" size={14} />
                            <span>Sonido en alertas</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.soundOnAlerts} />
                    </label>
                </div>
            </section>
            
            <!-- Monitoreo -->
            <section class="settings-card">
                <div class="card-header">
                    <Icon name="activity" size={18} />
                    <h2>Monitoreo</h2>
                </div>
                
                <div class="setting-group">
                    <span class="setting-label">
                        <Icon name="timer" size={12} />
                        Intervalo de actualización
                    </span>
                    <select bind:value={settings.updateInterval} class="setting-select">
                        <option value={500}>0.5s (alto consumo)</option>
                        <option value={1000}>1 segundo</option>
                        <option value={2000}>2 segundos</option>
                        <option value={5000}>5 segundos (bajo consumo)</option>
                    </select>
                </div>
                
                <div class="toggles-group">
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="play" size={14} />
                            <span>Auto-iniciar monitoreo</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.autoStartMonitoring} />
                    </label>
                    
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="package" size={14} />
                            <span>Contar paquetes</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.monitorPackets} />
                    </label>
                    
                    <label class="toggle-row">
                        <div class="toggle-info">
                            <Icon name="alert-circle" size={14} />
                            <span>Detectar errores</span>
                        </div>
                        <input type="checkbox" bind:checked={settings.monitorErrors} />
                    </label>
                </div>
            </section>
            
            <!-- Avanzado -->
            <section class="settings-card">
                <div class="card-header">
                    <Icon name="terminal" size={18} />
                    <h2>Avanzado</h2>
                </div>
                
                <label class="toggle-row">
                    <div class="toggle-info">
                        <Icon name="bug" size={14} />
                        <span>Modo debug</span>
                    </div>
                    <input type="checkbox" bind:checked={settings.debugMode} />
                </label>
                
                <div class="setting-group">
                    <span class="setting-label">
                        <Icon name="file-text" size={12} />
                        Nivel de log
                    </span>
                    <select bind:value={settings.logLevel} class="setting-select">
                        <option value="error">Solo errores</option>
                        <option value="warn">Warnings</option>
                        <option value="info">Info</option>
                        <option value="debug">Debug</option>
                    </select>
                </div>
                
                <label class="toggle-row">
                    <div class="toggle-info">
                        <Icon name="database" size={14} />
                        <span>Caché habilitado</span>
                    </div>
                    <input type="checkbox" bind:checked={settings.cacheEnabled} />
                </label>
            </section>
        </div>
    </div>
    
    <!-- Footer Actions -->
    <footer class="settings-footer">
        <div class="action-group">
            <button class="btn btn-outline" on:click={exportSettings}>
                <Icon name="download" size={14} />
                Exportar
            </button>
            <label class="btn btn-outline file-btn">
                <Icon name="upload" size={14} />
                Importar
                <input type="file" accept=".json" on:change={importSettings} hidden />
            </label>
        </div>
        
        <div class="action-group">
            <a class="btn btn-ghost" href="https://github.com/LOUST-PRO/NetBoozt_InternetUpgrade/discussions" target="_blank">
                <Icon name="help-circle" size={14} />
                Ayuda
            </a>
            <button class="btn btn-danger" on:click={resetToDefaults}>
                <Icon name="rotate-ccw" size={14} />
                Restaurar
            </button>
        </div>
    </footer>
    
    <!-- About mini -->
    <div class="about-mini">
        <span class="app-name">⚡ NetBoozt</span>
        <span class="app-version">v3.0.0</span>
        <span class="divider">•</span>
        <a href="https://loust.pro" target="_blank">LOUST</a>
    </div>
</div>

<style>
    .settings-page {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        height: 100%;
        overflow-y: auto;
        padding-bottom: 1rem;
    }
    
    .settings-layout {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }
    
    @media (max-width: 800px) {
        .settings-layout { grid-template-columns: 1fr; }
    }
    
    .settings-column {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
    
    .settings-card {
        background: var(--bg-card, #1a1a1a);
        border-radius: 12px;
        padding: 1.25rem;
    }
    
    .settings-card.full-height {
        flex: 1;
    }
    
    .card-header {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 1rem;
        color: var(--primary, #00d4aa);
    }
    
    .card-header h2 {
        margin: 0;
        font-size: 1rem;
        font-weight: 600;
        color: var(--text-primary, #fff);
    }
    
    .setting-group {
        margin-bottom: 1rem;
    }
    
    .setting-label {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.7rem;
        font-weight: 600;
        color: var(--text-muted, #888);
        margin-bottom: 0.5rem;
        text-transform: uppercase;
        letter-spacing: 0.03em;
    }
    
    .setting-select {
        width: 100%;
        padding: 0.5rem 0.75rem;
        background: var(--bg-elevated, #2b2b2b);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 8px;
        color: var(--text-primary, #fff);
        font-size: 0.8rem;
    }
    
    .setting-select:focus {
        outline: none;
        border-color: var(--primary, #00d4aa);
    }
    
    /* Background Grid */
    .background-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 0.5rem;
    }
    
    .bg-option {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.35rem;
        padding: 0.5rem;
        background: var(--bg-elevated, #2b2b2b);
        border: 2px solid transparent;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.2s;
    }
    
    .bg-option:hover { background: rgba(255, 255, 255, 0.08); }
    .bg-option.active { border-color: var(--primary, #00d4aa); }
    
    .bg-preview {
        width: 100%;
        height: 32px;
        border-radius: 4px;
        background: var(--preview);
    }
    
    .bg-preview.custom {
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(255, 255, 255, 0.05);
        border: 1px dashed var(--text-muted, #666);
        color: var(--text-muted, #666);
    }
    
    .custom-bg.active .bg-preview.custom {
        border-color: var(--primary, #00d4aa);
        color: var(--primary, #00d4aa);
    }
    
    .bg-name {
        font-size: 0.65rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    /* Color Grid */
    .color-grid {
        display: flex;
        gap: 0.5rem;
        flex-wrap: wrap;
    }
    
    .color-option {
        width: 32px;
        height: 32px;
        border-radius: 50%;
        background: var(--color);
        border: 2px solid transparent;
        cursor: pointer;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    
    .color-option:hover { transform: scale(1.1); }
    .color-option.active { border-color: #fff; box-shadow: 0 0 0 2px var(--color); }
    
    /* DNS Options */
    .dns-options {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }
    
    .dns-option {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 0.75rem;
        background: var(--bg-elevated, #2b2b2b);
        border: 1px solid transparent;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.15s;
        text-align: left;
    }
    
    .dns-option:hover { background: rgba(255, 255, 255, 0.08); }
    .dns-option.active { border-color: var(--primary, #00d4aa); background: rgba(0, 212, 170, 0.1); }
    
    .dns-icon { font-size: 1rem; }
    .dns-name { flex: 1; font-size: 0.8rem; color: var(--text-primary, #fff); font-weight: 500; }
    .dns-ip { font-size: 0.7rem; color: var(--text-muted, #666); font-family: monospace; }
    
    /* Toggle Rows */
    .toggles-group {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }
    
    .toggle-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.625rem 0.75rem;
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 8px;
        cursor: pointer;
        transition: background 0.15s;
    }
    
    .toggle-row:hover { background: rgba(255, 255, 255, 0.08); }
    
    .toggle-info {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.8rem;
        color: var(--text-primary, #fff);
    }
    
    .toggle-row input[type="checkbox"] {
        width: 38px;
        height: 20px;
        accent-color: var(--primary, #00d4aa);
        cursor: pointer;
    }
    
    /* Footer */
    .settings-footer {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        background: var(--bg-card, #1a1a1a);
        border-radius: 12px;
        flex-wrap: wrap;
        gap: 0.75rem;
    }
    
    .action-group {
        display: flex;
        gap: 0.5rem;
    }
    
    .btn {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.5rem 0.875rem;
        border-radius: 8px;
        font-size: 0.75rem;
        font-weight: 500;
        cursor: pointer;
        border: none;
        transition: all 0.15s;
        text-decoration: none;
    }
    
    .btn-outline {
        background: transparent;
        color: var(--text-secondary, #a0a0a0);
        border: 1px solid var(--border, #3d3d3d);
    }
    
    .btn-outline:hover { background: rgba(255, 255, 255, 0.08); color: var(--text-primary, #fff); }
    
    .btn-ghost {
        background: transparent;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .btn-ghost:hover { background: rgba(255, 255, 255, 0.08); color: var(--text-primary, #fff); }
    
    .btn-danger {
        background: rgba(239, 68, 68, 0.1);
        color: #ef4444;
        border: 1px solid rgba(239, 68, 68, 0.3);
    }
    
    .btn-danger:hover { background: rgba(239, 68, 68, 0.2); }
    
    .file-btn { cursor: pointer; }
    
    /* About Mini */
    .about-mini {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.75rem;
        font-size: 0.7rem;
        color: var(--text-muted, #666);
    }
    
    .app-name {
        font-weight: 600;
        color: var(--primary, #00d4aa);
    }
    
    .divider { opacity: 0.3; }
    
    .about-mini a {
        color: var(--text-secondary, #a0a0a0);
        text-decoration: none;
    }
    
    .about-mini a:hover { color: var(--primary, #00d4aa); }
</style>
