<script lang="ts">
    /**
     * GitHub Integration Component
     * Muestra estadísticas del repositorio con datos dinámicos desde GitHub API
     */
    import { onMount, createEventDispatcher } from 'svelte';
    import Tooltip from './Tooltip.svelte';
    import Icon from './Icon.svelte';
    
    const dispatch = createEventDispatcher();
    
    const REPO_OWNER = 'louzt';
    const REPO_NAME = 'NetBoozt_InternetUpgrade';
    
    function goToDocs() {
        dispatch('navigate', { tab: 'docs', anchor: 'readme' });
    }
    
    interface RepoStats {
        stars: number;
        forks: number;
        watchers: number;
        issues: number;
        lastUpdate: string;
        description: string;
        license: string;
        languages: Record<string, number>;
    }
    
    interface Release {
        tag: string;
        name: string;
        date: string;
        downloads: number;
        prerelease: boolean;
        assets: { name: string; size: string; downloadUrl: string; platform: string }[];
    }
    
    interface Contributor {
        login: string;
        avatar: string;
        contributions: number;
        profileUrl: string;
    }
    
    // Versiones del proyecto con lenguajes asociados
    const VERSIONS = [
        {
            name: 'Tauri (Rust + Svelte)',
            version: '3.0.0',
            status: 'production',
            icon: '⚡',
            description: 'Versión nativa multiplataforma. Inicio instantáneo (<1s), ~8MB de tamaño.',
            features: ['Inicio ultra-rápido', 'Bajo consumo RAM', 'Multiplataforma nativa', 'UI moderna con Svelte'],
            languages: ['Rust', 'TypeScript', 'Svelte', 'CSS', 'HTML'],
            downloadUrl: 'https://loust.pro/opensource/netboozt/tauri_v3.0.0'
        },
        {
            name: 'Python (CustomTkinter)',
            version: '2.2.0',
            status: 'legacy',
            icon: '🐍',
            description: 'Versión original en Python. Inicio lento (5-8s), ~25MB. Solo para CLI.',
            features: ['CLI interactivo', 'Nuitka build', 'Solo Windows', 'Mantenimiento mínimo'],
            languages: ['Python', 'PowerShell', 'Batchfile'],
            downloadUrl: 'https://loust.pro/opensource/netboozt/python_v2.2.0_legacy'
        }
    ];
    
    let stats: RepoStats | null = null;
    let releases: Release[] = [];
    let contributors: Contributor[] = [];
    let loading = true;
    let error: string | null = null;
    
    onMount(async () => {
        await fetchRepoData();
    });
    
    async function fetchRepoData() {
        loading = true;
        error = null;
        
        try {
            // Fetch repo info
            const repoRes = await fetch(`https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}`);
            if (repoRes.ok) {
                const repo = await repoRes.json();
                
                // Fetch languages desde GitHub API
                let languages: Record<string, number> = {};
                try {
                    const langRes = await fetch(`https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/languages`);
                    if (langRes.ok) {
                        languages = await langRes.json();
                    }
                } catch (e) { /* ignore */ }
                
                stats = {
                    stars: repo.stargazers_count,
                    forks: repo.forks_count,
                    watchers: repo.subscribers_count,
                    issues: repo.open_issues_count,
                    lastUpdate: new Date(repo.updated_at).toLocaleDateString(),
                    description: repo.description || 'Network Optimization Tool for Windows',
                    license: repo.license?.name || 'Apache-2.0 License',
                    languages
                };
            }
            
            // Fetch releases - solo mostramos los que existen en GitHub
            const releasesRes = await fetch(`https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases?per_page=10`);
            if (releasesRes.ok) {
                const releasesData = await releasesRes.json();
                releases = releasesData.map((r: any) => ({
                    tag: r.tag_name,
                    name: r.name || r.tag_name,
                    date: new Date(r.published_at).toLocaleDateString(),
                    prerelease: r.prerelease,
                    downloads: r.assets.reduce((sum: number, a: any) => sum + a.download_count, 0),
                    assets: r.assets.map((a: any) => ({
                        name: a.name,
                        size: (a.size / 1024 / 1024).toFixed(1) + ' MB',
                        downloadUrl: a.browser_download_url,
                        platform: detectPlatform(a.name)
                    }))
                }));
            }
            
            // Fetch contributors
            const contribRes = await fetch(`https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/contributors?per_page=20`);
            if (contribRes.ok) {
                const contribData = await contribRes.json();
                contributors = contribData.map((c: any) => ({
                    login: c.login,
                    avatar: c.avatar_url,
                    contributions: c.contributions,
                    profileUrl: c.html_url
                }));
            }
        } catch (e) {
            error = 'No se pudo conectar con GitHub API';
            console.error(e);
        } finally {
            loading = false;
        }
    }
    
    function detectPlatform(filename: string): string {
        const lower = filename.toLowerCase();
        if (lower.includes('.exe') || lower.includes('windows') || lower.includes('-win')) return 'windows';
        if (lower.includes('.dmg') || lower.includes('macos') || lower.includes('darwin')) return 'macos';
        if (lower.includes('.appimage') || lower.includes('.deb') || lower.includes('linux')) return 'linux';
        return 'other';
    }
    
    function getPlatformIcon(platform: string): string {
        const icons: Record<string, string> = {
            'windows': '🪟',
            'macos': '🍎',
            'linux': '🐧',
            'other': '📦'
        };
        return icons[platform] || '📦';
    }
    
    function openGitHub(path = '') {
        window.open(`https://github.com/${REPO_OWNER}/${REPO_NAME}${path}`, '_blank');
    }
    
    function formatBytes(bytes: number): string {
        const total = Object.values(stats?.languages || {}).reduce((a, b) => a + b, 0);
        if (total === 0) return '0%';
        const percentage = (bytes / total) * 100;
        return percentage.toFixed(1) + '%';
    }
    
    // Calcular porcentaje numérico para CSS - basado en bytes propios
    function getLanguagePercent(bytes: number): number {
        const total = Object.values(stats?.languages || {}).reduce((a, b) => a + b, 0);
        if (total === 0) return 0;
        return (bytes / total) * 100;
    }
    
    // Para las barras de versión, calcular porcentaje relativo a los lenguajes de esa versión
    function getVersionLanguagePercent(lang: string, versionLanguages: string[]): number {
        if (!stats?.languages) return 0;
        const languages = stats.languages;
        const versionBytes = versionLanguages.reduce((sum, l) => sum + (languages[l] || 0), 0);
        if (versionBytes === 0) return 0;
        const bytes = languages[lang] || 0;
        return (bytes / versionBytes) * 100;
    }
    
    // Obtener porcentaje formateado de un lenguaje específico
    function getLanguagePercentFormatted(lang: string): string {
        if (!stats?.languages) return '0%';
        const total = Object.values(stats.languages).reduce((a, b) => a + b, 0);
        if (total === 0) return '0%';
        const bytes = stats.languages[lang] || 0;
        return ((bytes / total) * 100).toFixed(1) + '%';
    }
    
    function getLanguageColor(lang: string): string {
        const colors: Record<string, string> = {
            'Python': '#3572A5',
            'Rust': '#dea584',
            'TypeScript': '#3178c6',
            'JavaScript': '#f1e05a',
            'Svelte': '#ff3e00',
            'HTML': '#e34c26',
            'CSS': '#563d7c',
            'PowerShell': '#012456',
            'Shell': '#89e051',
            'Batchfile': '#C1F12E'
        };
        return colors[lang] || '#666';
    }
</script>

<div class="github-page">
    {#if loading}
        <div class="loading-container">
            <div class="spinner"></div>
            <p>Conectando con GitHub...</p>
        </div>
    {:else if error}
        <div class="error-container">
            <span class="error-icon">⚠️</span>
            <p>{error}</p>
            <button class="btn btn-primary" on:click={fetchRepoData}>Reintentar</button>
        </div>
    {:else}
        <!-- Repo Header -->
        <section class="repo-header card">
            <div class="repo-info">
                <div class="repo-name">
                    <Icon name="github" size={24} />
                    <span>{REPO_OWNER}/{REPO_NAME}</span>
                </div>
                {#if stats}
                    <p class="repo-desc">{stats.description}</p>
                {/if}
            </div>
            <div class="repo-actions">
                <button class="btn btn-primary" on:click={() => openGitHub()}>
                    <Icon name="star" size={16} />
                    Ver en GitHub
                </button>
            </div>
        </section>
        
        <!-- Stats Cards - Grid uniforme -->
        {#if stats}
            <section class="stats-grid">
                <Tooltip text="Usuarios que marcaron el repo como favorito">
                    <button class="stat-card" on:click={() => openGitHub('/stargazers')}>
                        <span class="stat-icon">⭐</span>
                        <div class="stat-content">
                            <span class="stat-value">{stats.stars}</span>
                            <span class="stat-label">Stars</span>
                        </div>
                    </button>
                </Tooltip>
                
                <Tooltip text="Copias del repositorio">
                    <button class="stat-card" on:click={() => openGitHub('/forks')}>
                        <span class="stat-icon">🍴</span>
                        <div class="stat-content">
                            <span class="stat-value">{stats.forks}</span>
                            <span class="stat-label">Forks</span>
                        </div>
                    </button>
                </Tooltip>
                
                <Tooltip text="Observadores">
                    <button class="stat-card" on:click={() => openGitHub('/watchers')}>
                        <span class="stat-icon">👁️</span>
                        <div class="stat-content">
                            <span class="stat-value">{stats.watchers}</span>
                            <span class="stat-label">Watchers</span>
                        </div>
                    </button>
                </Tooltip>
                
                <Tooltip text="Issues abiertos">
                    <button class="stat-card" on:click={() => openGitHub('/issues')}>
                        <span class="stat-icon">🐛</span>
                        <div class="stat-content">
                            <span class="stat-value">{stats.issues}</span>
                            <span class="stat-label">Issues</span>
                        </div>
                    </button>
                </Tooltip>
            </section>
            
            <!-- Meta Info + Languages -->
            <section class="info-row">
                <div class="meta-card card">
                    <div class="meta-item">
                        <span class="meta-icon">📜</span>
                        <span class="meta-value">{stats.license}</span>
                    </div>
                    <div class="meta-item">
                        <span class="meta-icon">📅</span>
                        <span class="meta-value">{stats.lastUpdate}</span>
                    </div>
                </div>
            </section>
            
            <!-- Languages Bar - Full Width -->
            {#if Object.keys(stats.languages).length > 0}
                {@const totalBytes = Object.values(stats.languages).reduce((a, b) => a + b, 0)}
                <section class="card languages-section">
                    <span class="card-title">Lenguajes del Repositorio</span>
                    <div class="languages-bar-full">
                        {#each Object.entries(stats.languages).sort((a, b) => b[1] - a[1]) as [lang, bytes]}
                            {@const percent = totalBytes > 0 ? (bytes / totalBytes) * 100 : 0}
                            <div 
                                class="lang-segment-full" 
                                style="flex: {percent} 0 0%; background: {getLanguageColor(lang)};"
                                title="{lang}: {percent.toFixed(1)}%"
                            ></div>
                        {/each}
                    </div>
                    <div class="languages-legend-full">
                        {#each Object.entries(stats.languages).sort((a, b) => b[1] - a[1]) as [lang, bytes]}
                            {@const percent = totalBytes > 0 ? (bytes / totalBytes) * 100 : 0}
                            <span class="lang-item-full">
                                <span class="lang-dot" style="background: {getLanguageColor(lang)}"></span>
                                <span class="lang-name">{lang}</span>
                                <span class="lang-percent">{percent.toFixed(1)}%</span>
                            </span>
                        {/each}
                    </div>
                </section>
            {/if}
        {/if}
        
        <!-- Versiones del Proyecto con Lenguajes -->
        <section class="card versions-section">
            <div class="card-header">
                <h2>🚀 Versiones Disponibles</h2>
            </div>
            <div class="versions-grid">
                {#each VERSIONS as ver}
                    <div class="version-card" class:production={ver.status === 'production'} class:legacy={ver.status === 'legacy'}>
                        <div class="version-header">
                            <span class="version-icon">{ver.icon}</span>
                            <div class="version-info">
                                <span class="version-name">{ver.name}</span>
                                <span class="version-tag">
                                    v{ver.version}
                                    {#if ver.status === 'production'}
                                        <span class="status-badge production">✓ Producción</span>
                                    {:else if ver.status === 'legacy'}
                                        <span class="status-badge legacy">📦 Legacy</span>
                                    {/if}
                                </span>
                            </div>
                        </div>
                        <p class="version-desc">{ver.description}</p>
                        
                        <!-- Lenguajes de esta versión -->
                        {#if ver.languages && stats?.languages}
                            {@const languages = stats.languages}
                            {@const versionBytes = ver.languages.reduce((sum, l) => sum + (languages[l] || 0), 0)}
                            <div class="version-languages">
                                <span class="lang-title">Lenguajes:</span>
                                <div class="lang-bar-mini">
                                    {#each ver.languages as lang}
                                        {@const bytes = languages[lang] || 0}
                                        {@const percent = versionBytes > 0 ? (bytes / versionBytes) * 100 : 0}
                                        {#if percent > 0}
                                            <div 
                                                class="lang-seg" 
                                                style="flex: {percent} 0 0%; background: {getLanguageColor(lang)};"
                                                title="{lang}: {percent.toFixed(1)}%"
                                            ></div>
                                        {/if}
                                    {/each}
                                </div>
                                <div class="lang-tags">
                                    {#each ver.languages as lang}
                                        {@const bytes = languages[lang] || 0}
                                        {@const percent = versionBytes > 0 ? (bytes / versionBytes) * 100 : 0}
                                        <span class="lang-tag" style="--color: {getLanguageColor(lang)}">
                                            <span class="tag-dot"></span>
                                            {lang}
                                            {#if percent > 0}
                                                <span class="tag-percent">{percent.toFixed(0)}%</span>
                                            {/if}
                                        </span>
                                    {/each}
                                </div>
                            </div>
                        {/if}
                        
                        <ul class="version-features">
                            {#each ver.features as feat}
                                <li>{feat}</li>
                            {/each}
                        </ul>
                        
                        <!-- Botón de descarga -->
                        <a href={ver.downloadUrl} target="_blank" class="version-download-btn" class:primary={ver.status === 'production'}>
                            <Icon name="download" size={14} />
                            Descargar v{ver.version}
                        </a>
                    </div>
                {/each}
            </div>
        </section>
        
        <!-- Releases desde GitHub -->
        <section class="card">
            <div class="card-header">
                <h2>📦 Releases en GitHub</h2>
                <button class="btn btn-ghost" on:click={() => openGitHub('/releases')}>
                    Ver todos →
                </button>
            </div>
            
            {#if releases.length === 0}
                <div class="empty-releases">
                    <Icon name="download" size={32} />
                    <p>No hay releases publicados aún</p>
                    <span class="empty-hint">Los releases estarán disponibles en GitHub cuando se publiquen versiones estables.</span>
                    <button class="btn btn-primary" on:click={() => openGitHub('/releases')}>
                        Ir a GitHub Releases
                    </button>
                </div>
            {:else}
                <div class="releases-list">
                    {#each releases as release}
                        {@const isTauri = release.tag.startsWith('v3')}
                        {@const downloadUrl = isTauri 
                            ? 'https://loust.pro/opensource/netboozt/tauri_v3.0.0' 
                            : 'https://loust.pro/opensource/netboozt/python_v2.2.0_legacy'}
                        <div class="release-item" class:prerelease={release.prerelease}>
                            <div class="release-header">
                                <div class="release-info">
                                    <a href="https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/tag/{release.tag}" 
                                       target="_blank" 
                                       class="release-tag-link">
                                        <span class="release-tag">{release.tag}</span>
                                    </a>
                                    <a href="https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/tag/{release.tag}" 
                                       target="_blank"
                                       class="release-name-link">
                                        {release.name}
                                    </a>
                                    {#if release.prerelease}
                                        <span class="prerelease-badge">Pre-release</span>
                                    {/if}
                                </div>
                                <div class="release-meta">
                                    <span class="release-date">{release.date}</span>
                                    <span class="release-downloads">⬇️ {release.downloads}</span>
                                </div>
                            </div>
                            
                            <!-- Botones de descarga -->
                            <div class="release-actions">
                                <a href={downloadUrl} target="_blank" class="btn btn-primary btn-sm">
                                    <Icon name="download" size={14} />
                                    Descargar desde LOUST
                                </a>
                                <a href="https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/tag/{release.tag}" 
                                   target="_blank" 
                                   class="btn btn-ghost btn-sm">
                                    Ver en GitHub
                                </a>
                            </div>
                            
                            {#if release.assets.length > 0}
                                <div class="release-assets">
                                    {#each release.assets as asset}
                                        <a href={asset.downloadUrl} class="asset-item" target="_blank">
                                            <span class="asset-platform">{getPlatformIcon(asset.platform)}</span>
                                            <span class="asset-name">{asset.name}</span>
                                            <span class="asset-size">{asset.size}</span>
                                            <Icon name="download" size={14} />
                                        </a>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/each}
                </div>
            {/if}
        </section>
        
        <!-- Quick Links - 5 columns -->
        <section class="quick-links">
            <button class="quick-link" on:click={() => openGitHub('/issues/new')}>
                <Icon name="bug" size={16} />
                Reportar Bug
            </button>
            <button class="quick-link" on:click={() => openGitHub('/discussions')}>
                <Icon name="chat" size={16} />
                Discusiones
            </button>
            <button class="quick-link" on:click={() => openGitHub('/wiki')}>
                <Icon name="book" size={16} />
                Wiki
            </button>
            <button class="quick-link" on:click={() => openGitHub('/blob/main/CONTRIBUTING.md')}>
                <Icon name="code" size={16} />
                Contribuir
            </button>
            <button class="quick-link" on:click={() => openGitHub('/graphs/contributors')}>
                <Icon name="people" size={16} />
                Contribuidores
            </button>
        </section>
        
        <!-- CTA to Docs -->
        <section class="docs-cta card">
            <div class="cta-content">
                <div class="cta-icon">📖</div>
                <div class="cta-text">
                    <h3>¿Quieres ver la documentación completa?</h3>
                    <p>El README y guías de uso están disponibles en la pestaña de Documentación</p>
                </div>
            </div>
            <button class="btn btn-primary cta-button" on:click={goToDocs}>
                <Icon name="book" size={16} />
                Ir a Documentación
            </button>
        </section>
    {/if}
</div>

<style>
    .github-page {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        height: 100%;
        overflow-y: auto;
    }
    
    .loading-container, .error-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        flex: 1;
        gap: 1rem;
        color: var(--text-muted, #666);
    }
    
    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid var(--border, #3d3d3d);
        border-top-color: var(--primary, #00d4aa);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }
    
    @keyframes spin { to { transform: rotate(360deg); } }
    
    .error-icon { font-size: 3rem; }
    
    .card {
        background: var(--bg-card, #1a1a1a);
        border-radius: 12px;
        padding: 1.25rem;
    }
    
    .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }
    
    .card-header h2 {
        font-size: 1rem;
        font-weight: 600;
        margin: 0;
        color: var(--text-primary, #fff);
    }
    
    .card-title {
        font-size: 0.75rem;
        color: var(--text-muted, #666);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin-bottom: 0.5rem;
        display: block;
    }
    
    /* Repo Header */
    .repo-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 1rem;
    }
    
    .repo-name {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 1.125rem;
        font-weight: 600;
        color: var(--primary, #00d4aa);
    }
    
    .repo-desc {
        margin: 0.5rem 0 0 0;
        font-size: 0.875rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    /* Stats Grid - Igual ancho */
    .stats-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 1rem;
    }
    
    @media (max-width: 600px) {
        .stats-grid {
            grid-template-columns: repeat(2, 1fr);
        }
    }
    
    .stat-card {
        background: var(--bg-card, #1a1a1a);
        border-radius: 12px;
        padding: 1rem;
        display: flex;
        align-items: center;
        gap: 0.75rem;
        cursor: pointer;
        transition: all 0.2s ease;
        border: 1px solid transparent;
        width: 100%;
    }
    
    .stat-card:hover {
        border-color: var(--primary, #00d4aa);
        transform: translateY(-2px);
    }
    
    .stat-icon {
        font-size: 1.5rem;
        flex-shrink: 0;
    }
    
    .stat-content {
        display: flex;
        flex-direction: column;
        min-width: 0;
    }
    
    .stat-value {
        font-size: 1.25rem;
        font-weight: 700;
        color: var(--text-primary, #fff);
    }
    
    .stat-label {
        font-size: 0.7rem;
        color: var(--text-muted, #666);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    
    .meta-card {
        display: flex;
        gap: 2rem;
        padding: 1rem 1.25rem;
        flex: 1;
        flex-wrap: wrap;
    }
    
    .meta-item {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.875rem;
    }
    
    .meta-icon { font-size: 1rem; }
    
    .meta-value {
        color: var(--text-primary, #fff);
        font-weight: 500;
    }
    
    /* Languages - Full Width Bar */
    .languages-section {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
    
    .languages-bar-full {
        display: flex;
        width: 100%;
        height: 10px;
        border-radius: 5px;
        overflow: hidden;
        background: var(--bg-elevated, #2b2b2b);
    }
    
    .lang-segment-full {
        height: 100%;
        min-width: 2px;
        flex-shrink: 1;
        flex-grow: 0;
        transition: opacity 0.2s, filter 0.2s;
    }
    
    .lang-segment-full:hover {
        filter: brightness(1.3);
    }
    
    .languages-legend-full {
        display: flex;
        flex-wrap: wrap;
        gap: 0.75rem 1rem;
    }
    
    .lang-item-full {
        display: flex;
        align-items: center;
        gap: 0.375rem;
        font-size: 0.75rem;
        color: var(--text-secondary, #a0a0a0);
        cursor: pointer;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        transition: background 0.15s;
    }
    
    .lang-item-full:hover {
        background: var(--bg-elevated, #2b2b2b);
    }
    
    .lang-name {
        font-weight: 500;
        color: var(--text-primary, #fff);
    }
    
    .lang-dot {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        flex-shrink: 0;
    }
    
    .lang-percent {
        color: var(--text-muted, #666);
        font-size: 0.7rem;
    }
    
    /* Versions Section */
    .versions-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 1rem;
    }
    
    @media (max-width: 700px) {
        .versions-grid {
            grid-template-columns: 1fr;
        }
    }
    
    .version-card {
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 12px;
        padding: 1.25rem;
        border: 1px solid transparent;
        transition: all 0.2s;
        display: flex;
        flex-direction: column;
    }
    
    .version-card.production {
        border-color: var(--primary, #00d4aa);
        background: linear-gradient(135deg, rgba(0, 212, 170, 0.08), transparent);
    }
    
    .version-card.legacy {
        border-color: var(--border, #3d3d3d);
        opacity: 0.85;
    }
    
    .version-header {
        display: flex;
        align-items: flex-start;
        gap: 0.75rem;
        margin-bottom: 0.75rem;
    }
    
    .version-icon {
        font-size: 1.75rem;
    }
    
    .version-info {
        display: flex;
        flex-direction: column;
    }
    
    .version-name {
        font-weight: 600;
        color: var(--text-primary, #fff);
        font-size: 0.9rem;
    }
    
    .version-tag {
        font-size: 0.7rem;
        color: var(--primary, #00d4aa);
    }
    
    .version-desc {
        font-size: 0.8rem;
        color: var(--text-secondary, #a0a0a0);
        margin: 0 0 0.75rem 0;
        line-height: 1.5;
    }
    
    .version-features {
        margin: 0;
        padding-left: 1.25rem;
        font-size: 0.75rem;
        color: var(--text-muted, #888);
    }
    
    .version-features li {
        margin-bottom: 0.25rem;
    }
    
    /* Status badges */
    .status-badge {
        display: inline-block;
        padding: 0.15rem 0.5rem;
        border-radius: 4px;
        font-size: 0.65rem;
        font-weight: 600;
        margin-left: 0.5rem;
    }
    
    .status-badge.production {
        background: rgba(0, 212, 170, 0.2);
        color: var(--primary, #00d4aa);
    }
    
    .status-badge.legacy {
        background: rgba(255, 255, 255, 0.1);
        color: var(--text-muted, #666);
    }
    
    /* Version download button */
    .version-download-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.625rem 1rem;
        border-radius: 8px;
        font-size: 0.8rem;
        font-weight: 500;
        text-decoration: none;
        margin-top: auto;
        transition: all 0.15s ease;
        background: var(--bg-card, #1a1a1a);
        color: var(--text-secondary, #a0a0a0);
        border: 1px solid var(--border, #3d3d3d);
    }
    
    .version-download-btn:hover {
        background: var(--bg-elevated, #3b3b3b);
        color: var(--text-primary, #fff);
    }
    
    .version-download-btn.primary {
        background: var(--primary, #00d4aa);
        color: #000;
        border-color: var(--primary, #00d4aa);
    }
    
    .version-download-btn.primary:hover {
        background: #00e6b8;
    }
    
    /* Version languages */
    .version-languages {
        margin: 0.75rem 0;
        padding: 0.75rem;
        background: rgba(0, 0, 0, 0.2);
        border-radius: 8px;
    }
    
    .lang-title {
        font-size: 0.65rem;
        color: var(--text-muted, #666);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        display: block;
        margin-bottom: 0.5rem;
    }
    
    .lang-bar-mini {
        display: flex;
        width: 100%;
        height: 8px;
        border-radius: 4px;
        overflow: hidden;
        background: rgba(255, 255, 255, 0.1);
        margin-bottom: 0.5rem;
    }
    
    .lang-seg {
        height: 100%;
        min-width: 2px;
        transition: filter 0.15s;
    }
    
    .lang-seg:hover {
        filter: brightness(1.3);
    }
    
    .lang-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 0.35rem;
    }
    
    .lang-tag {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        font-size: 0.65rem;
        color: var(--text-secondary, #a0a0a0);
        padding: 0.15rem 0.4rem;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 4px;
    }
    
    .tag-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: var(--color);
    }
    
    .tag-percent {
        font-size: 0.6rem;
        color: var(--text-muted, #666);
        margin-left: 0.15rem;
    }
    
    /* Releases */
    .empty-releases {
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 2rem;
        color: var(--text-muted, #666);
        text-align: center;
        gap: 0.75rem;
    }
    
    .empty-releases p {
        margin: 0;
        font-size: 1rem;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .empty-hint {
        font-size: 0.8rem;
        max-width: 300px;
    }
    
    .releases-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
    
    .release-item {
        background: var(--bg-elevated, #2b2b2b);
        border-radius: 10px;
        padding: 1rem;
        border: 1px solid transparent;
    }
    
    .release-item:hover {
        border-color: var(--border, #3d3d3d);
    }
    
    .release-item.prerelease {
        border-left: 3px solid #fdcb6e;
    }
    
    .release-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-bottom: 0.75rem;
    }
    
    .release-info {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-wrap: wrap;
    }
    
    .release-tag-link {
        text-decoration: none;
    }
    
    .release-tag-link:hover .release-tag {
        background: #00e6b8;
    }
    
    .release-tag {
        background: var(--primary, #00d4aa);
        color: #000;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-size: 0.75rem;
        font-weight: 600;
        transition: background 0.15s;
    }
    
    .release-name-link {
        font-size: 0.875rem;
        color: var(--text-primary, #fff);
        font-weight: 500;
        text-decoration: none;
        transition: color 0.15s;
    }
    
    .release-name-link:hover {
        color: var(--primary, #00d4aa);
    }
    
    .release-actions {
        display: flex;
        gap: 0.5rem;
        margin: 0.75rem 0;
        flex-wrap: wrap;
    }
    
    .btn-sm {
        padding: 0.5rem 0.875rem;
        font-size: 0.75rem;
    }
    
    .prerelease-badge {
        font-size: 0.65rem;
        background: rgba(253, 203, 110, 0.2);
        color: #fdcb6e;
        padding: 0.2rem 0.4rem;
        border-radius: 3px;
    }
    
    .release-meta {
        display: flex;
        align-items: center;
        gap: 1rem;
        font-size: 0.75rem;
        color: var(--text-muted, #666);
    }
    
    .release-assets {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        padding-top: 0.75rem;
        border-top: 1px solid var(--border, #3d3d3d);
    }
    
    .asset-item {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 0.75rem;
        background: var(--bg-card, #1a1a1a);
        border-radius: 6px;
        font-size: 0.75rem;
        color: var(--text-secondary, #a0a0a0);
        text-decoration: none;
        transition: all 0.15s;
    }
    
    .asset-item:hover {
        background: var(--primary, #00d4aa);
        color: #000;
    }
    
    .asset-platform {
        font-size: 1rem;
    }
    
    .asset-name {
        font-weight: 500;
    }
    
    .asset-size {
        color: var(--text-muted, #666);
    }
    
    .asset-item:hover .asset-size {
        color: rgba(0,0,0,0.6);
    }
    
    /* Quick Links - 5 columns */
    .quick-links {
        display: grid;
        grid-template-columns: repeat(5, 1fr);
        gap: 0.75rem;
    }
    
    @media (max-width: 700px) {
        .quick-links {
            grid-template-columns: repeat(3, 1fr);
        }
    }
    
    @media (max-width: 500px) {
        .quick-links {
            grid-template-columns: repeat(2, 1fr);
        }
    }
    
    .quick-link {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.875rem 1rem;
        background: var(--bg-card, #1a1a1a);
        border: 1px solid var(--border, #2d2d2d);
        border-radius: 8px;
        color: var(--text-secondary, #a0a0a0);
        font-size: 0.8rem;
        cursor: pointer;
        transition: all 0.2s ease;
    }
    
    .quick-link:hover {
        background: var(--bg-elevated, #2b2b2b);
        color: var(--text-primary, #fff);
        border-color: var(--primary, #00d4aa);
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
    
    .btn-primary {
        background: var(--primary, #00d4aa);
        color: #000;
    }
    
    .btn-primary:hover {
        background: #00e6b8;
    }
    
    .btn-ghost {
        background: transparent;
        color: var(--text-secondary, #a0a0a0);
    }
    
    .btn-ghost:hover {
        background: var(--bg-elevated, #2b2b2b);
        color: var(--text-primary, #fff);
    }
    
    /* Docs CTA */
    .docs-cta {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1.5rem;
        padding: 1.25rem 1.5rem;
        background: linear-gradient(135deg, rgba(0, 212, 170, 0.1), rgba(0, 212, 170, 0.02));
        border: 1px solid rgba(0, 212, 170, 0.3);
    }
    
    .cta-content {
        display: flex;
        align-items: center;
        gap: 1rem;
    }
    
    .cta-icon {
        font-size: 2rem;
        flex-shrink: 0;
    }
    
    .cta-text h3 {
        font-size: 0.9375rem;
        font-weight: 600;
        color: var(--text-primary, #fff);
        margin: 0 0 0.25rem 0;
    }
    
    .cta-text p {
        font-size: 0.8125rem;
        color: var(--text-secondary, #a0a0a0);
        margin: 0;
    }
    
    .cta-button {
        flex-shrink: 0;
        white-space: nowrap;
    }
    
    @media (max-width: 600px) {
        .docs-cta {
            flex-direction: column;
            text-align: center;
        }
        
        .cta-content {
            flex-direction: column;
        }
    }
</style>
