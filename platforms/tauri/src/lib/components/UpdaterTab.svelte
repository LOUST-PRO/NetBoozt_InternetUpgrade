<script lang="ts">
    /**
     * UpdaterTab - Sistema de Actualización Automática
     * Detecta nuevas versiones desde GitHub y permite descargar/instalar
     * Incluye comandos para terminal y LLM
     * 
     * By LOUST (www.loust.pro)
     */
    import { onMount } from 'svelte';
    import { invoke } from '$lib/tauri-bridge';
    import Icon from './Icon.svelte';
    
    const REPO_OWNER = 'louzt';
    const REPO_NAME = 'NetBoozt_InternetUpgrade';
    const REPO_URL = `https://github.com/${REPO_OWNER}/${REPO_NAME}`;
    const DOWNLOAD_BASE_URL = 'https://loust.pro/opensource/netboozt';
    
    interface Release {
        tag_name: string;
        name: string;
        body: string;
        published_at: string;
        prerelease: boolean;
        html_url: string;
        assets: {
            name: string;
            size: number;
            browser_download_url: string;
        }[];
    }
    
    interface UpdateInfo {
        hasUpdate: boolean;
        currentVersion: string;
        latestVersion: string;
        release: Release | null;
        changelog: string;
    }
    
    // State
    let checking = false;
    let downloading = false;
    let downloadProgress = 0;
    let updateInfo: UpdateInfo | null = null;
    let error: string | null = null;
    let lastCheck: Date | null = null;
    let autoCheckEnabled = true;
    let currentVersion = '3.0.1';
    let copiedCommand = '';
    let platform: 'windows' | 'linux' | 'other' = 'other';
    
    // Settings
    let checkInterval = 24; // horas
    let downloadPath = '';
    
    // Comandos de instalación
    const installCommands = {
        gitClone: `git clone ${REPO_URL}.git
cd NetBoozt_InternetUpgrade
cd platforms/tauri
npm install
npm run tauri build`,
        
        gitPull: `cd NetBoozt_InternetUpgrade
git pull origin main
cd platforms/tauri
npm install
npm run tauri build`,
        
        quickStart: `# Clonar y ejecutar en modo desarrollo
git clone ${REPO_URL}.git && cd NetBoozt_InternetUpgrade/platforms/tauri && npm install && npm run tauri dev`,
        
        buildOnly: `# Solo compilar (después de git pull)
cd platforms/tauri && npm run tauri build`,
        
        releaseDownload: `# Descargar última release desde GitHub
curl -L -o netboozt-setup.exe "${REPO_URL}/releases/latest/download/NetBoozt_x64-setup.exe"`,
    };
    
    const llmInstructions = `# NetBoozt - Instrucciones para AI/LLM

## Repositorio
- URL: ${REPO_URL}
- Branch principal: main
- Versión actual: v${currentVersion}

## Estructura del proyecto
\`\`\`
NetBoozt/
├── platforms/
│   ├── tauri/          ← Versión PRODUCCIÓN (Rust + SvelteKit)
│   │   ├── src/        ← Frontend Svelte
│   │   └── src-tauri/  ← Backend Rust
│   └── python/         ← Versión LEGACY
└── windows/            ← CLI Python legacy
\`\`\`

## Comandos importantes
\`\`\`bash
# Desarrollo
cd platforms/tauri && npm run tauri dev

# Build producción
cd platforms/tauri && npm run tauri build

# Output: src-tauri/target/release/bundle/
\`\`\`

## Tecnologías
- Frontend: SvelteKit + TypeScript
- Backend: Rust + Tauri
- Optimizaciones: TCP/IP BBR-like para Windows

## Archivos clave
- platforms/tauri/src/routes/+page.svelte (App principal)
- platforms/tauri/src-tauri/src/main.rs (Entry Rust)
- .github/instructions/*.md (Docs para AI)
`;

    function detectPlatform(): 'windows' | 'linux' | 'other' {
        const ua = navigator.userAgent.toLowerCase();
        if (ua.includes('windows')) return 'windows';
        if (ua.includes('linux')) return 'linux';
        return 'other';
    }

    function isWindowsAsset(name: string): boolean {
        const lower = name.toLowerCase();
        return lower.endsWith('.exe') || lower.endsWith('.msi');
    }

    function isLinuxAsset(name: string): boolean {
        const lower = name.toLowerCase();
        return lower.endsWith('.appimage') ||
            lower.endsWith('.deb') ||
            lower.endsWith('.rpm') ||
            lower.endsWith('.tar.gz') ||
            lower.endsWith('.tar.xz') ||
            lower.endsWith('.zip');
    }

    function pickInstallerAsset(assets: Release['assets']) {
        const predicates = platform === 'linux'
            ? [
                (asset: Release['assets'][number]) => asset.name.toLowerCase().endsWith('.appimage'),
                (asset: Release['assets'][number]) => asset.name.toLowerCase().endsWith('.deb'),
                (asset: Release['assets'][number]) => asset.name.toLowerCase().endsWith('.rpm'),
                (asset: Release['assets'][number]) => asset.name.toLowerCase().endsWith('.tar.gz'),
                (asset: Release['assets'][number]) => asset.name.toLowerCase().endsWith('.tar.xz'),
                (asset: Release['assets'][number]) => asset.name.toLowerCase().endsWith('.zip')
            ]
            : platform === 'windows'
                ? [
                    (asset: Release['assets'][number]) => asset.name.toLowerCase().endsWith('.exe'),
                    (asset: Release['assets'][number]) => asset.name.toLowerCase().endsWith('.msi')
                ]
                : [
                    (asset: Release['assets'][number]) => isWindowsAsset(asset.name),
                    (asset: Release['assets'][number]) => isLinuxAsset(asset.name)
                ];

        for (const predicate of predicates) {
            const match = assets.find(predicate);
            if (match) {
                return match;
            }
        }

        return null;
    }

    $: downloadButtonLabel = platform === 'linux' ? 'Descargar paquete Linux' : 'Descargar instalador';
    $: installButtonLabel = platform === 'linux' ? 'Abrir paquete descargado' : 'Instalar ahora';

    onMount(async () => {
        platform = detectPlatform();

        // Obtener versión actual desde Rust
        try {
            currentVersion = await invoke<string>('get_app_version');
        } catch (e) {
            console.error('Error getting version:', e);
        }
        
        // Auto-check al iniciar
        if (autoCheckEnabled) {
            await checkForUpdates();
        }
    });
    
    async function checkForUpdates() {
        checking = true;
        error = null;
        
        try {
            // Fetch latest release desde GitHub
            const response = await fetch(
                `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest`
            );
            
            if (!response.ok) {
                // Si no hay releases, buscar tags
                const tagsResponse = await fetch(
                    `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/tags`
                );
                if (tagsResponse.ok) {
                    const tags = await tagsResponse.json();
                    if (tags.length > 0) {
                        const latestTag = tags[0].name;
                        updateInfo = {
                            hasUpdate: compareVersions(latestTag, currentVersion) > 0,
                            currentVersion,
                            latestVersion: latestTag,
                            release: null,
                            changelog: 'Ver releases en GitHub para más información.'
                        };
                        lastCheck = new Date();
                        return;
                    }
                }
                throw new Error('No hay releases disponibles');
            }
            
            const release: Release = await response.json();
            const latestVersion = release.tag_name.replace(/^v/, '');
            
            updateInfo = {
                hasUpdate: compareVersions(latestVersion, currentVersion) > 0,
                currentVersion,
                latestVersion,
                release,
                changelog: release.body || 'Sin notas de versión'
            };
            
            lastCheck = new Date();
            
        } catch (e: any) {
            error = `Error verificando actualizaciones: ${e.message}`;
        } finally {
            checking = false;
        }
    }
    
    function compareVersions(v1: string, v2: string): number {
        v1 = v1.replace(/^v/, '');
        v2 = v2.replace(/^v/, '');
        
        const parts1 = v1.split('.').map(Number);
        const parts2 = v2.split('.').map(Number);
        
        for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
            const p1 = parts1[i] || 0;
            const p2 = parts2[i] || 0;
            if (p1 > p2) return 1;
            if (p1 < p2) return -1;
        }
        return 0;
    }
    
    async function copyToClipboard(text: string, label: string) {
        try {
            await navigator.clipboard.writeText(text);
            copiedCommand = label;
            setTimeout(() => copiedCommand = '', 2000);
        } catch (e) {
            console.error('Error copying:', e);
        }
    }
    
    async function downloadUpdate() {
        if (!updateInfo?.release) return;
        
        downloading = true;
        downloadProgress = 0;
        error = null;
        
        try {
            const assets = updateInfo.release.assets;
            const installer = pickInstallerAsset(assets);
            
            if (installer) {
                downloadProgress = 10;
                const result = await invoke<string>('download_update', {
                    url: installer.browser_download_url,
                    filename: installer.name
                });
                
                downloadProgress = 100;
                downloadPath = result;
                
            } else {
                openDownloadPage();
            }
            
        } catch (e: any) {
            error = `Error descargando: ${e.message || e}`;
        } finally {
            downloading = false;
        }
    }
    
    async function installUpdate() {
        if (!downloadPath) return;
        
        try {
            await invoke('install_update', { path: downloadPath });
        } catch (e: any) {
            error = `Error instalando: ${e.message || e}`;
        }
    }
    
    function openReleasePage() {
        if (updateInfo?.release) {
            window.open(updateInfo.release.html_url, '_blank');
        } else {
            window.open(`${REPO_URL}/releases`, '_blank');
        }
    }
    
    function openDownloadPage() {
        const version = updateInfo?.latestVersion || currentVersion;
        window.open(`${DOWNLOAD_BASE_URL}/tauri_v${version}`, '_blank');
    }
    
    function formatDate(dateStr: string): string {
        return new Date(dateStr).toLocaleDateString('es-ES', {
            year: 'numeric',
            month: 'long',
            day: 'numeric'
        });
    }
    
    function formatBytes(bytes: number): string {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
    
    function parseChangelog(body: string): string[] {
        if (!body) return [];
        return body
            .split('\n')
            .filter(line => line.trim().match(/^[-*•]/))
            .map(line => line.replace(/^[-*•]\s*/, '').trim())
            .filter(line => line.length > 0)
            .slice(0, 10);
    }
</script>

<div class="updater-page">
    <!-- Header con versión actual -->
    <section class="version-header card">
        <div class="version-info">
            <div class="version-icon">🚀</div>
            <div class="version-details">
                <h2>NetBoozt</h2>
                <div class="version-number">
                    <span class="label">Versión instalada:</span>
                    <span class="value">v{currentVersion}</span>
                    <span class="badge production">Tauri</span>
                </div>
            </div>
        </div>
        
        <button 
            class="btn btn-primary" 
            on:click={checkForUpdates}
            disabled={checking}
        >
            {#if checking}
                <span class="spinner-sm"></span>
                Verificando...
            {:else}
                <Icon name="refresh" size={16} />
                Buscar actualizaciones
            {/if}
        </button>
    </section>
    
    <!-- Estado de actualización -->
    {#if error}
        <section class="card error-card">
            <Icon name="warning" size={24} />
            <div class="error-content">
                <h3>Error</h3>
                <p>{error}</p>
            </div>
            <button class="btn btn-ghost" on:click={() => error = null}>✕</button>
        </section>
    {/if}
    
    {#if updateInfo}
        {#if updateInfo.hasUpdate}
            <!-- Nueva versión disponible -->
            <section class="card update-available">
                <div class="update-header">
                    <div class="update-icon">🎉</div>
                    <div class="update-info">
                        <h3>¡Nueva versión disponible!</h3>
                        <p class="version-comparison">
                            <span class="old">v{updateInfo.currentVersion}</span>
                            <span class="arrow">→</span>
                            <span class="new">v{updateInfo.latestVersion}</span>
                        </p>
                        {#if updateInfo.release}
                            <p class="release-date">
                                Publicado: {formatDate(updateInfo.release.published_at)}
                            </p>
                        {/if}
                    </div>
                </div>
                
                <!-- Changelog -->
                {#if updateInfo.changelog}
                    <div class="changelog">
                        <h4>📋 Novedades</h4>
                        <div class="changelog-content">
                            {#each parseChangelog(updateInfo.changelog) as item}
                                <div class="changelog-item">
                                    <span class="bullet">•</span>
                                    {item}
                                </div>
                            {:else}
                                <p class="changelog-raw">{updateInfo.changelog.slice(0, 500)}</p>
                            {/each}
                        </div>
                    </div>
                {/if}
                
                <!-- Assets disponibles -->
                {#if updateInfo.release?.assets && updateInfo.release.assets.length > 0}
                    <div class="assets-section">
                        <h4>📦 Archivos de instalación</h4>
                        <div class="assets-grid">
                            {#each updateInfo.release.assets as asset}
                                <a 
                                    href={asset.browser_download_url} 
                                    target="_blank" 
                                    class="asset-item"
                                >
                                    <span class="asset-icon">
                                        {#if asset.name.includes('.exe')}🪟
                                        {:else if asset.name.includes('.msi')}📦
                                        {:else if asset.name.toLowerCase().includes('.appimage')}🐧
                                        {:else if asset.name.toLowerCase().includes('.deb') || asset.name.toLowerCase().includes('.rpm')}📦
                                        {:else if asset.name.includes('.zip')}🗜️
                                        {:else}📄{/if}
                                    </span>
                                    <div class="asset-info">
                                        <span class="asset-name">{asset.name}</span>
                                        <span class="asset-size">{formatBytes(asset.size)}</span>
                                    </div>
                                    <Icon name="download" size={16} />
                                </a>
                            {/each}
                        </div>
                    </div>
                {/if}
                
                <!-- Botones de acción -->
                <div class="update-actions">
                    <button 
                        class="btn btn-primary btn-lg"
                        on:click={downloadUpdate}
                        disabled={downloading}
                    >
                        <Icon name="download" size={18} />
                        {downloadButtonLabel}
                    </button>
                    
                    <button 
                        class="btn btn-secondary"
                        on:click={openReleasePage}
                    >
                        <Icon name="external" size={16} />
                        Ver en GitHub
                    </button>
                </div>
                
                <!-- Barra de progreso si está descargando -->
                {#if downloading}
                    <div class="download-progress">
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: {downloadProgress}%"></div>
                        </div>
                        <span class="progress-text">{downloadProgress}%</span>
                    </div>
                {/if}
                
                <!-- Instalador descargado -->
                {#if downloadPath}
                    <div class="download-complete">
                        <Icon name="check" size={20} />
                        <span>Descargado: {downloadPath}</span>
                        <button class="btn btn-primary" on:click={installUpdate}>
                            {installButtonLabel}
                        </button>
                    </div>
                {/if}
            </section>
        {:else}
            <!-- Ya tienes la última versión -->
            <section class="card up-to-date">
                <div class="status-icon">✅</div>
                <div class="status-info">
                    <h3>¡Estás al día!</h3>
                    <p>Tienes la versión más reciente de NetBoozt</p>
                    {#if lastCheck}
                        <span class="last-check">
                            Última verificación: {lastCheck.toLocaleString('es-ES')}
                        </span>
                    {/if}
                </div>
            </section>
        {/if}
    {:else if !checking && !error}
        <!-- Estado inicial -->
        <section class="card initial-state">
            <div class="status-icon">🔍</div>
            <div class="status-info">
                <h3>Verificar actualizaciones</h3>
                <p>Haz clic en el botón para buscar nuevas versiones</p>
            </div>
        </section>
    {/if}
    
    <!-- Configuración de actualizaciones -->
    <section class="card settings-section">
        <h3>⚙️ Configuración</h3>
        
        <div class="setting-item">
            <label class="toggle-label">
                <input type="checkbox" bind:checked={autoCheckEnabled} />
                <span class="toggle-switch"></span>
                <span class="toggle-text">Verificar automáticamente al iniciar</span>
            </label>
        </div>
        
        <div class="setting-item">
            <label class="select-label">
                <span>Intervalo de verificación:</span>
                <select bind:value={checkInterval}>
                    <option value={6}>Cada 6 horas</option>
                    <option value={12}>Cada 12 horas</option>
                    <option value={24}>Cada día</option>
                    <option value={168}>Cada semana</option>
                </select>
            </label>
        </div>
    </section>
    
    <!-- Info adicional -->
    <section class="card info-section">
        <h3>ℹ️ Información</h3>
        <div class="info-grid">
            <div class="info-item">
                <span class="info-label">Canal:</span>
                <span class="info-value">Estable (Tauri)</span>
            </div>
            <div class="info-item">
                <span class="info-label">Arquitectura:</span>
                <span class="info-value">x64</span>
            </div>
            <div class="info-item">
                <span class="info-label">Repositorio:</span>
                <a href="https://github.com/{REPO_OWNER}/{REPO_NAME}" target="_blank">
                    GitHub
                </a>
            </div>
            <div class="info-item">
                <span class="info-label">Página de descargas:</span>
                <a href="https://loust.pro/opensource/netboozt" target="_blank">
                    loust.pro
                </a>
            </div>
        </div>
    </section>
    
    <!-- Comandos de Terminal -->
    <section class="card commands-section">
        <h3>💻 Comandos de Terminal</h3>
        <p class="commands-desc">Copia estos comandos para instalar o actualizar desde terminal:</p>
        
        <div class="command-group">
            <div class="command-header">
                <span class="command-title">🚀 Instalación rápida (desarrollo)</span>
                <button 
                    class="copy-btn" 
                    class:copied={copiedCommand === 'quickStart'}
                    on:click={() => copyToClipboard(installCommands.quickStart, 'quickStart')}
                >
                    {copiedCommand === 'quickStart' ? '✓ Copiado' : 'Copiar'}
                </button>
            </div>
            <pre class="command-code">{installCommands.quickStart}</pre>
        </div>
        
        <div class="command-group">
            <div class="command-header">
                <span class="command-title">📦 Clonar y compilar</span>
                <button 
                    class="copy-btn"
                    class:copied={copiedCommand === 'gitClone'}
                    on:click={() => copyToClipboard(installCommands.gitClone, 'gitClone')}
                >
                    {copiedCommand === 'gitClone' ? '✓ Copiado' : 'Copiar'}
                </button>
            </div>
            <pre class="command-code">{installCommands.gitClone}</pre>
        </div>
        
        <div class="command-group">
            <div class="command-header">
                <span class="command-title">🔄 Actualizar existente (git pull)</span>
                <button 
                    class="copy-btn"
                    class:copied={copiedCommand === 'gitPull'}
                    on:click={() => copyToClipboard(installCommands.gitPull, 'gitPull')}
                >
                    {copiedCommand === 'gitPull' ? '✓ Copiado' : 'Copiar'}
                </button>
            </div>
            <pre class="command-code">{installCommands.gitPull}</pre>
        </div>
        
        <div class="command-group">
            <div class="command-header">
                <span class="command-title">⚡ Solo build (después de pull)</span>
                <button 
                    class="copy-btn"
                    class:copied={copiedCommand === 'buildOnly'}
                    on:click={() => copyToClipboard(installCommands.buildOnly, 'buildOnly')}
                >
                    {copiedCommand === 'buildOnly' ? '✓ Copiado' : 'Copiar'}
                </button>
            </div>
            <pre class="command-code">{installCommands.buildOnly}</pre>
        </div>
    </section>
    
    <!-- Instrucciones para LLM/AI -->
    <section class="card llm-section">
        <div class="llm-header">
            <h3>🤖 Instrucciones para AI/LLM</h3>
            <button 
                class="copy-btn btn-lg"
                class:copied={copiedCommand === 'llm'}
                on:click={() => copyToClipboard(llmInstructions, 'llm')}
            >
                <Icon name="clipboard" size={14} />
                {copiedCommand === 'llm' ? '✓ Copiado' : 'Copiar todo'}
            </button>
        </div>
        <p class="llm-desc">
            Copia estas instrucciones para que Copilot, ChatGPT u otro LLM 
            tenga contexto sobre el proyecto:
        </p>
        <pre class="llm-code">{llmInstructions}</pre>
        
        <div class="llm-tips">
            <h4>💡 Tips para AI</h4>
            <ul>
                <li>Pega estas instrucciones al inicio de tu conversación</li>
                <li>El archivo <code>.github/instructions/</code> contiene más contexto</li>
                <li>La versión Tauri es la de producción, Python es legacy</li>
                <li>Frontend en Svelte, Backend en Rust</li>
            </ul>
        </div>
    </section>
</div>

<style>
    .updater-page {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        max-width: 800px;
        margin: 0 auto;
    }
    
    .card {
        background: var(--bg-card, #1a1a1a);
        border-radius: 12px;
        padding: 1.25rem;
    }
    
    /* Version Header */
    .version-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 1rem;
    }
    
    .version-info {
        display: flex;
        align-items: center;
        gap: 1rem;
    }
    
    .version-icon {
        font-size: 2.5rem;
    }
    
    .version-details h2 {
        margin: 0;
        font-size: 1.25rem;
        color: var(--text-primary, #fff);
    }
    
    .version-number {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-top: 0.25rem;
    }
    
    .version-number .label {
        font-size: 0.8rem;
        color: var(--text-muted, #666);
    }
    
    .version-number .value {
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--primary, #00d4aa);
    }
    
    .badge {
        font-size: 0.65rem;
        padding: 0.15rem 0.5rem;
        border-radius: 4px;
        font-weight: 600;
    }
    
    .badge.production {
        background: rgba(0, 212, 170, 0.2);
        color: var(--primary, #00d4aa);
    }
    
    /* Error Card */
    .error-card {
        display: flex;
        align-items: center;
        gap: 1rem;
        background: rgba(255, 107, 107, 0.1);
        border: 1px solid var(--error, #ff6b6b);
        color: var(--error, #ff6b6b);
    }
    
    .error-content h3 {
        margin: 0;
        font-size: 0.9rem;
    }
    
    .error-content p {
        margin: 0.25rem 0 0;
        font-size: 0.8rem;
        opacity: 0.9;
    }
    
    /* Update Available */
    .update-available {
        border: 2px solid var(--primary, #00d4aa);
        background: linear-gradient(135deg, rgba(0, 212, 170, 0.1), transparent);
    }
    
    .update-header {
        display: flex;
        align-items: flex-start;
        gap: 1rem;
        margin-bottom: 1rem;
    }
    
    .update-icon {
        font-size: 2rem;
    }
    
    .update-info h3 {
        margin: 0;
        font-size: 1.1rem;
        color: var(--primary, #00d4aa);
    }
    
    .version-comparison {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin: 0.5rem 0;
        font-size: 0.9rem;
    }
    
    .version-comparison .old {
        color: var(--text-muted, #666);
        text-decoration: line-through;
    }
    
    .version-comparison .arrow {
        color: var(--primary, #00d4aa);
    }
    
    .version-comparison .new {
        color: var(--primary, #00d4aa);
        font-weight: 600;
    }
    
    .release-date {
        font-size: 0.75rem;
        color: var(--text-muted, #666);
        margin: 0;
    }
    
    /* Changelog */
    .changelog {
        background: rgba(0, 0, 0, 0.2);
        border-radius: 8px;
        padding: 1rem;
        margin: 1rem 0;
    }
    
    .changelog h4 {
        margin: 0 0 0.75rem;
        font-size: 0.85rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .changelog-content {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
    
    .changelog-item {
        display: flex;
        gap: 0.5rem;
        font-size: 0.8rem;
        color: var(--text-primary, #fff);
    }
    
    .changelog-item .bullet {
        color: var(--primary, #00d4aa);
    }
    
    .changelog-raw {
        font-size: 0.8rem;
        color: var(--text-secondary, #a0a0a0);
        white-space: pre-wrap;
        margin: 0;
    }
    
    /* Assets */
    .assets-section {
        margin: 1rem 0;
    }
    
    .assets-section h4 {
        margin: 0 0 0.75rem;
        font-size: 0.85rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .assets-grid {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
    
    .asset-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem;
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 8px;
        text-decoration: none;
        color: var(--text-primary, #fff);
        transition: all 0.15s;
    }
    
    .asset-item:hover {
        background: var(--primary, #00d4aa);
        color: #000;
    }
    
    .asset-icon {
        font-size: 1.25rem;
    }
    
    .asset-info {
        flex: 1;
        display: flex;
        flex-direction: column;
    }
    
    .asset-name {
        font-size: 0.8rem;
        font-weight: 500;
    }
    
    .asset-size {
        font-size: 0.7rem;
        opacity: 0.7;
    }
    
    /* Actions */
    .update-actions {
        display: flex;
        gap: 0.75rem;
        margin-top: 1rem;
        flex-wrap: wrap;
    }
    
    /* Progress */
    .download-progress {
        display: flex;
        align-items: center;
        gap: 1rem;
        margin-top: 1rem;
    }
    
    .progress-bar {
        flex: 1;
        height: 8px;
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 4px;
        overflow: hidden;
    }
    
    .progress-fill {
        height: 100%;
        background: var(--primary, #00d4aa);
        transition: width 0.3s;
    }
    
    .progress-text {
        font-size: 0.8rem;
        color: var(--text-secondary, #a0a0a0);
        min-width: 40px;
    }
    
    /* Download Complete */
    .download-complete {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-top: 1rem;
        padding: 0.75rem;
        background: rgba(0, 212, 170, 0.1);
        border-radius: 8px;
        color: var(--primary, #00d4aa);
    }
    
    /* Up to Date */
    .up-to-date, .initial-state {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 2rem;
    }
    
    .status-icon {
        font-size: 2.5rem;
    }
    
    .status-info h3 {
        margin: 0;
        font-size: 1.1rem;
        color: var(--text-primary, #fff);
    }
    
    .status-info p {
        margin: 0.25rem 0 0;
        font-size: 0.85rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .last-check {
        font-size: 0.7rem;
        color: var(--text-muted, #666);
        margin-top: 0.5rem;
        display: block;
    }
    
    /* Settings */
    .settings-section h3 {
        margin: 0 0 1rem;
        font-size: 0.9rem;
        color: var(--text-primary, #fff);
    }
    
    .setting-item {
        margin-bottom: 0.75rem;
    }
    
    .toggle-label {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        cursor: pointer;
    }
    
    .toggle-label input {
        display: none;
    }
    
    .toggle-switch {
        width: 40px;
        height: 20px;
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 10px;
        position: relative;
        transition: background 0.2s;
    }
    
    .toggle-switch::after {
        content: '';
        position: absolute;
        width: 16px;
        height: 16px;
        background: var(--text-muted, #666);
        border-radius: 50%;
        top: 2px;
        left: 2px;
        transition: all 0.2s;
    }
    
    .toggle-label input:checked + .toggle-switch {
        background: var(--primary, #00d4aa);
    }
    
    .toggle-label input:checked + .toggle-switch::after {
        left: 22px;
        background: #000;
    }
    
    .toggle-text {
        font-size: 0.85rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .select-label {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 0.85rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .select-label select {
        padding: 0.5rem;
        background: var(--bg-elevated, #2b2b2b);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 6px;
        color: var(--text-primary, #fff);
        font-size: 0.8rem;
    }
    
    /* Info Section */
    .info-section h3 {
        margin: 0 0 1rem;
        font-size: 0.9rem;
        color: var(--text-primary, #fff);
    }
    
    .info-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 0.75rem;
    }
    
    @media (max-width: 500px) {
        .info-grid {
            grid-template-columns: 1fr;
        }
    }
    
    .info-item {
        display: flex;
        gap: 0.5rem;
        font-size: 0.8rem;
    }
    
    .info-label {
        color: var(--text-muted, #666);
    }
    
    .info-value {
        color: var(--text-primary, #fff);
    }
    
    .info-item a {
        color: var(--primary, #00d4aa);
        text-decoration: none;
    }
    
    .info-item a:hover {
        text-decoration: underline;
    }
    
    /* Buttons */
    .btn {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.625rem 1rem;
        border-radius: 8px;
        font-size: 0.875rem;
        font-weight: 500;
        cursor: pointer;
        border: none;
        transition: all 0.15s ease;
    }
    
    .btn-lg {
        padding: 0.75rem 1.25rem;
        font-size: 0.9rem;
    }
    
    .btn-primary {
        background: var(--primary, #00d4aa);
        color: #000;
    }
    
    .btn-primary:hover:not(:disabled) {
        background: #00e6b8;
    }
    
    .btn-primary:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
    
    .btn-secondary {
        background: var(--bg-elevated, #2b2b2b);
        color: var(--text-primary, #fff);
        border: 1px solid var(--border, #3d3d3d);
    }
    
    .btn-secondary:hover {
        background: var(--bg-card, #1a1a1a);
        border-color: var(--primary, #00d4aa);
    }
    
    .btn-ghost {
        background: transparent;
        color: var(--text-secondary, #a0a0a0);
        padding: 0.25rem;
    }
    
    .spinner-sm {
        width: 16px;
        height: 16px;
        border: 2px solid transparent;
        border-top-color: currentColor;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }
    
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
    
    /* Commands Section */
    .commands-section h3 {
        margin: 0 0 0.5rem;
        font-size: 0.9rem;
        color: var(--text-primary, #fff);
    }
    
    .commands-desc {
        font-size: 0.8rem;
        color: var(--text-muted, #666);
        margin: 0 0 1rem;
    }
    
    .command-group {
        margin-bottom: 1rem;
        background: rgba(0, 0, 0, 0.2);
        border-radius: 8px;
        overflow: hidden;
    }
    
    .command-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1rem;
        background: rgba(0, 212, 170, 0.1);
        border-bottom: 1px solid rgba(0, 212, 170, 0.2);
    }
    
    .command-title {
        font-size: 0.8rem;
        font-weight: 500;
        color: var(--text-primary, #fff);
    }
    
    .copy-btn {
        padding: 0.35rem 0.75rem;
        background: var(--bg-elevated, #2b2b2b);
        border: 1px solid var(--border, #3d3d3d);
        border-radius: 6px;
        color: var(--text-secondary, #a0a0a0);
        font-size: 0.7rem;
        cursor: pointer;
        transition: all 0.15s;
        display: flex;
        align-items: center;
        gap: 0.35rem;
    }
    
    .copy-btn:hover {
        background: var(--primary, #00d4aa);
        color: #000;
        border-color: var(--primary, #00d4aa);
    }
    
    .copy-btn.copied {
        background: rgba(0, 212, 170, 0.2);
        color: var(--primary, #00d4aa);
        border-color: var(--primary, #00d4aa);
    }
    
    .copy-btn.btn-lg {
        padding: 0.5rem 1rem;
        font-size: 0.75rem;
    }
    
    .command-code {
        margin: 0;
        padding: 1rem;
        font-family: 'JetBrains Mono', 'Fira Code', monospace;
        font-size: 0.75rem;
        line-height: 1.6;
        color: var(--text-secondary, #a0a0a0);
        white-space: pre-wrap;
        word-break: break-all;
        overflow-x: auto;
    }
    
    /* LLM Section */
    .llm-section h3 {
        margin: 0;
        font-size: 0.9rem;
        color: var(--text-primary, #fff);
    }
    
    .llm-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.75rem;
    }
    
    .llm-desc {
        font-size: 0.8rem;
        color: var(--text-muted, #666);
        margin: 0 0 1rem;
    }
    
    .llm-code {
        margin: 0;
        padding: 1rem;
        background: rgba(0, 0, 0, 0.3);
        border-radius: 8px;
        font-family: 'JetBrains Mono', 'Fira Code', monospace;
        font-size: 0.7rem;
        line-height: 1.5;
        color: var(--text-secondary, #a0a0a0);
        white-space: pre-wrap;
        max-height: 300px;
        overflow-y: auto;
    }
    
    .llm-tips {
        margin-top: 1rem;
        padding: 1rem;
        background: rgba(0, 212, 170, 0.08);
        border-radius: 8px;
        border-left: 3px solid var(--primary, #00d4aa);
    }
    
    .llm-tips h4 {
        margin: 0 0 0.5rem;
        font-size: 0.85rem;
        color: var(--primary, #00d4aa);
    }
    
    .llm-tips ul {
        margin: 0;
        padding-left: 1.25rem;
        font-size: 0.75rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .llm-tips li {
        margin-bottom: 0.35rem;
    }
    
    .llm-tips code {
        background: rgba(0, 0, 0, 0.3);
        padding: 0.1rem 0.35rem;
        border-radius: 3px;
        font-size: 0.7rem;
    }
</style>
