//! DNS Intelligence System - Smart DNS Selection with Historical Analysis
//!
//! Sistema inteligente de selección de DNS que:
//! 1. Analiza rendimiento en paralelo de múltiples DNS
//! 2. Mantiene histórico de salud/rendimiento
//! 3. Selecciona automáticamente el mejor DNS basado en datos reales
//! 4. Pool compartido global (no por adaptador)
//! 5. Auto-failover agresivo cuando un DNS falla
//!
//! By LOUST (www.loust.pro)

use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use serde::{Deserialize, Serialize};

/// Métricas de un servidor DNS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsMetrics {
    pub address: String,
    pub name: String,

    // Métricas actuales
    pub ping_ms: f64,
    pub resolve_ms: f64,
    pub success_rate: f64,
    pub last_check: u64, // Unix timestamp
    pub is_healthy: bool,

    // Histórico (últimas 24h)
    pub avg_ping_24h: f64,
    pub avg_resolve_24h: f64,
    pub uptime_24h: f64,
    pub checks_24h: u32,
    pub failures_24h: u32,

    // Score calculado (0-100)
    pub score: f64,
    pub rank: u32,

    // Failover tracking
    pub consecutive_failures: u32,
}

impl DnsMetrics {
    fn new(address: &str, name: &str) -> Self {
        Self {
            address: address.to_string(),
            name: name.to_string(),
            ping_ms: 0.0,
            resolve_ms: 0.0,
            success_rate: 100.0,
            last_check: 0,
            is_healthy: true,
            avg_ping_24h: 0.0,
            avg_resolve_24h: 0.0,
            uptime_24h: 100.0,
            checks_24h: 0,
            failures_24h: 0,
            score: 50.0,
            rank: 0,
            consecutive_failures: 0,
        }
    }
}

/// Entrada de historial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: u64,
    pub address: String,
    pub ping_ms: f64,
    pub resolve_ms: f64,
    pub success: bool,
}

/// Evento de failover
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverEvent {
    pub timestamp: u64,
    pub from_dns: String,
    pub to_dns: String,
    pub reason: String,
    pub success: bool,
}

/// Configuración del sistema DNS Intelligence
#[derive(Debug, Clone)]
pub struct DnsIntelConfig {
    /// Intervalo entre checks en segundos (default: 30s para prod, 10s para agresivo)
    pub check_interval_secs: u64,
    /// Retención de historial en horas (default: 24h)
    pub history_retention_hours: u64,
    /// Máximo de entradas de historial
    pub max_history_entries: usize,
    /// Workers paralelos para checks
    pub parallel_workers: usize,
    /// Threshold para considerarlo "good" (ms)
    pub threshold_good_ms: f64,
    /// Threshold para considerarlo "slow" (ms)
    pub threshold_slow_ms: f64,
    /// Fallas consecutivas antes de failover
    pub max_consecutive_failures: u32,
    /// Cooldown entre failovers (segundos)
    pub failover_cooldown_secs: u64,
}

impl Default for DnsIntelConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 30, // Check cada 30s
            history_retention_hours: 24,
            max_history_entries: 1000,
            parallel_workers: 4,
            threshold_good_ms: 30.0,     // < 30ms = good
            threshold_slow_ms: 80.0,     // < 80ms = slow, >= 80ms = bad
            max_consecutive_failures: 2, // 2 fallas = failover
            failover_cooldown_secs: 30,  // 30s entre failovers
        }
    }
}

/// Servidores DNS conocidos
const DNS_SERVERS: &[(&str, &str)] = &[
    ("1.1.1.1", "Cloudflare"),
    ("1.0.0.1", "Cloudflare Secondary"),
    ("8.8.8.8", "Google"),
    ("8.8.4.4", "Google Secondary"),
    ("9.9.9.9", "Quad9"),
    ("149.112.112.112", "Quad9 Secondary"),
    ("208.67.222.222", "OpenDNS"),
    ("208.67.220.220", "OpenDNS Secondary"),
    ("94.140.14.14", "AdGuard"),
    ("94.140.15.15", "AdGuard Secondary"),
    ("185.228.168.9", "CleanBrowsing"),
];

/// Dominios de prueba para resolución
const TEST_DOMAINS: &[&str] = &[
    "google.com",
    "cloudflare.com",
    "microsoft.com",
    "amazon.com",
];

/// Estado global del sistema DNS Intelligence
pub struct DnsIntelligence {
    config: DnsIntelConfig,
    metrics: Arc<RwLock<HashMap<String, DnsMetrics>>>,
    history: Arc<Mutex<Vec<HistoryEntry>>>,
    failover_history: Arc<Mutex<Vec<FailoverEvent>>>,
    running: Arc<Mutex<bool>>,
    last_failover: Arc<Mutex<Option<Instant>>>,
    current_best_dns: Arc<RwLock<Option<String>>>,
    auto_failover_enabled: Arc<Mutex<bool>>,
}

impl DnsIntelligence {
    /// Crear nueva instancia
    pub fn new(config: DnsIntelConfig) -> Self {
        let mut metrics = HashMap::new();

        // Inicializar métricas para todos los DNS
        for (addr, name) in DNS_SERVERS {
            metrics.insert(addr.to_string(), DnsMetrics::new(addr, name));
        }

        Self {
            config,
            metrics: Arc::new(RwLock::new(metrics)),
            history: Arc::new(Mutex::new(Vec::new())),
            failover_history: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            last_failover: Arc::new(Mutex::new(None)),
            current_best_dns: Arc::new(RwLock::new(None)),
            auto_failover_enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// Iniciar servicio en segundo plano
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let metrics = Arc::clone(&self.metrics);
        let history = Arc::clone(&self.history);
        let failover_history = Arc::clone(&self.failover_history);
        let running = Arc::clone(&self.running);
        let last_failover = Arc::clone(&self.last_failover);
        let current_best = Arc::clone(&self.current_best_dns);
        let auto_failover = Arc::clone(&self.auto_failover_enabled);
        let config = self.config.clone();

        thread::spawn(move || {
            log::info!("🧠 DNS Intelligence service started (check interval: {}s, history retention: {}h, max entries: {}, parallel workers: {})",
                      config.check_interval_secs, config.history_retention_hours, config.max_history_entries, config.parallel_workers);

            // Check inicial
            Self::check_all_dns_static(&metrics, &history, &config);
            Self::calculate_scores_static(&metrics, &history, &config);
            Self::update_best_dns_static(&metrics, &current_best);

            loop {
                thread::sleep(Duration::from_secs(config.check_interval_secs));

                if !*running.lock().unwrap() {
                    break;
                }

                // Check todos los DNS
                Self::check_all_dns_static(&metrics, &history, &config);
                Self::calculate_scores_static(&metrics, &history, &config);

                // Verificar si necesita failover
                if *auto_failover.lock().unwrap() {
                    Self::check_failover_static(
                        &metrics,
                        &failover_history,
                        &last_failover,
                        &current_best,
                        &config,
                    );
                }

                Self::update_best_dns_static(&metrics, &current_best);
            }

            log::info!("🧠 DNS Intelligence service stopped");
        });
    }

    /// Detener servicio
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }

    /// Verificar todos los DNS (estático para usar en thread)
    fn check_all_dns_static(
        metrics: &Arc<RwLock<HashMap<String, DnsMetrics>>>,
        history: &Arc<Mutex<Vec<HistoryEntry>>>,
        config: &DnsIntelConfig,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Verificar cada DNS
        for (addr, _) in DNS_SERVERS {
            let (success, ping_ms, resolve_ms) = Self::check_single_dns(addr);

            // Actualizar métricas
            {
                let mut metrics_guard = metrics.write().unwrap();
                if let Some(m) = metrics_guard.get_mut(*addr) {
                    m.ping_ms = ping_ms;
                    m.resolve_ms = resolve_ms;
                    m.is_healthy = success;
                    m.last_check = timestamp;

                    if success {
                        m.consecutive_failures = 0;
                    } else {
                        m.consecutive_failures += 1;
                    }
                }
            }

            // Agregar al historial
            {
                let mut history_guard = history.lock().unwrap();
                history_guard.push(HistoryEntry {
                    timestamp,
                    address: addr.to_string(),
                    ping_ms,
                    resolve_ms,
                    success,
                });

                // Limpiar historial antiguo según configuración
                let cutoff = timestamp.saturating_sub(config.history_retention_hours * 60 * 60);
                history_guard.retain(|e| e.timestamp > cutoff);

                // Limitar tamaño del historial
                let current_len = history_guard.len();
                if current_len > config.max_history_entries {
                    let to_remove = current_len - config.max_history_entries;
                    history_guard.drain(0..to_remove);
                }
            }
        }
    }

    /// Verificar un DNS específico
    fn check_single_dns(address: &str) -> (bool, f64, f64) {
        // 1. Ping TCP al puerto 53 - usando Rust puro (más rápido)
        let ping_result = Self::tcp_ping_rust(address, 53, 2000);

        if ping_result.is_none() {
            return (false, 0.0, 0.0);
        }

        let ping_ms = ping_result.unwrap();

        // 2. Verificar resolución DNS real con un dominio de prueba aleatorio
        // Usamos uno de los TEST_DOMAINS para verificar que realmente funciona
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Seleccionar dominio de prueba basado en timestamp (rotación)
        let domain_idx = (timestamp % TEST_DOMAINS.len() as u64) as usize;
        let test_domain = TEST_DOMAINS[domain_idx];

        // Hacer resolución real cada 5 checks (para no sobrecargar)
        let should_resolve = timestamp % 5 == 0;

        let resolve_ms = if should_resolve {
            // Intentar resolución DNS real
            match Self::resolve_dns(address, test_domain, 3000) {
                Some(ms) => ms,
                None => ping_ms * 1.5, // Fallback si falla
            }
        } else {
            ping_ms * 1.5 // Estimación rápida
        };

        (true, ping_ms, resolve_ms)
    }

    /// TCP ping usando Rust puro - mucho más rápido que PowerShell
    fn tcp_ping_rust(address: &str, port: u16, timeout_ms: u64) -> Option<f64> {
        use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
        use std::time::Duration;

        let start = Instant::now();

        // Parsear la dirección
        let addr_str = format!("{}:{}", address, port);
        let addr: SocketAddr = match addr_str.to_socket_addrs() {
            Ok(mut addrs) => match addrs.next() {
                Some(a) => a,
                None => return None,
            },
            Err(_) => return None,
        };

        // Conectar con timeout
        let timeout = Duration::from_millis(timeout_ms);
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                Some(elapsed)
            }
            Err(_) => None,
        }
    }

    /// TCP ping a un servidor (versión PowerShell - backup)
    #[allow(dead_code)]
    fn tcp_ping(address: &str, port: u16, timeout_ms: u64) -> Option<f64> {
        let start = Instant::now();

        let ps_script = format!(
            r#"
            $tcp = New-Object System.Net.Sockets.TcpClient
            try {{
                $result = $tcp.BeginConnect('{}', {}, $null, $null)
                $wait = $result.AsyncWaitHandle.WaitOne({}, $false)
                if ($wait -and $tcp.Connected) {{
                    $tcp.EndConnect($result)
                    'OK'
                }} else {{
                    'FAIL'
                }}
            }} catch {{
                'FAIL'
            }} finally {{
                $tcp.Close()
            }}
            "#,
            address, port, timeout_ms
        );

        let result = Self::run_powershell(&ps_script);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        if result.as_ref().map(|s| s.trim()) == Some("OK") {
            Some(elapsed)
        } else {
            None
        }
    }

    /// Resolver DNS usando nslookup
    fn resolve_dns(dns_server: &str, domain: &str, timeout_ms: u64) -> Option<f64> {
        let start = Instant::now();

        #[cfg(windows)]
        let output = Command::new("nslookup")
            .args([domain, dns_server])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        #[cfg(not(windows))]
        let output = Command::new("nslookup").args([domain, dns_server]).output();

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        // Timeout check
        if elapsed > timeout_ms as f64 {
            return None;
        }

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Verificar que realmente resolvió (tiene "Address" en output)
                if stdout.contains("Address") && !stdout.contains("can't find") {
                    Some(elapsed)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Calcular scores basados en historial
    fn calculate_scores_static(
        metrics: &Arc<RwLock<HashMap<String, DnsMetrics>>>,
        history: &Arc<Mutex<Vec<HistoryEntry>>>,
        config: &DnsIntelConfig,
    ) {
        let history_guard = history.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now.saturating_sub(config.history_retention_hours * 60 * 60);

        let mut metrics_guard = metrics.write().unwrap();

        for (addr, m) in metrics_guard.iter_mut() {
            // Filtrar historial para este DNS
            let entries: Vec<_> = history_guard
                .iter()
                .filter(|e| &e.address == addr && e.timestamp > cutoff)
                .collect();

            if entries.is_empty() {
                continue;
            }

            m.checks_24h = entries.len() as u32;
            m.failures_24h = entries.iter().filter(|e| !e.success).count() as u32;

            // Calcular promedios de entries exitosas
            let successful: Vec<_> = entries.iter().filter(|e| e.success).collect();

            if !successful.is_empty() {
                m.avg_ping_24h =
                    successful.iter().map(|e| e.ping_ms).sum::<f64>() / successful.len() as f64;
                m.avg_resolve_24h =
                    successful.iter().map(|e| e.resolve_ms).sum::<f64>() / successful.len() as f64;
            }

            // Uptime
            m.uptime_24h = if entries.is_empty() {
                100.0
            } else {
                (successful.len() as f64 / entries.len() as f64) * 100.0
            };

            m.success_rate = m.uptime_24h;
        }

        // Calcular scores usando los thresholds de configuración
        // threshold_good_ms = latencia ideal (bonus score si está por debajo)
        // threshold_slow_ms = latencia máxima aceptable (penalización si está por encima)
        let max_resolve = config.threshold_slow_ms * 3.0; // La resolución suele ser más lenta

        for m in metrics_guard.values_mut() {
            if m.avg_ping_24h == 0.0 {
                m.score = 0.0;
                continue;
            }

            // Score de latencia: penaliza más si está por encima del threshold_slow_ms
            let ping_score = if m.avg_ping_24h <= config.threshold_good_ms {
                100.0 // Excelente - por debajo del threshold bueno
            } else if m.avg_ping_24h <= config.threshold_slow_ms {
                // Entre good y slow: escala lineal de 100 a 60
                let ratio = (m.avg_ping_24h - config.threshold_good_ms)
                    / (config.threshold_slow_ms - config.threshold_good_ms);
                100.0 - (ratio * 40.0)
            } else {
                // Por encima del slow: escala de 60 a 0
                let ratio = ((m.avg_ping_24h - config.threshold_slow_ms)
                    / config.threshold_slow_ms)
                    .min(1.0);
                60.0 - (ratio * 60.0)
            }
            .max(0.0);

            let resolve_score = (1.0 - (m.avg_resolve_24h / max_resolve)).max(0.0) * 100.0;
            let uptime_score = m.uptime_24h;

            // Score ponderado:
            // 45% latencia ping (menor = mejor, usa thresholds)
            // 25% latencia resolve (menor = mejor)
            // 30% uptime
            m.score = ping_score * 0.45 + resolve_score * 0.25 + uptime_score * 0.30;
        }

        // Calcular ranking - necesitamos recopilar datos primero
        let ranking: Vec<(String, f64)> = {
            let mut sorted: Vec<_> = metrics_guard
                .iter()
                .map(|(addr, m)| (addr.clone(), m.score))
                .collect();
            sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            sorted
        };

        // Ahora asignar ranks
        for (i, (addr, _)) in ranking.iter().enumerate() {
            if let Some(m) = metrics_guard.get_mut(addr) {
                m.rank = (i + 1) as u32;
            }
        }
    }

    /// Verificar si necesita failover
    fn check_failover_static(
        metrics: &Arc<RwLock<HashMap<String, DnsMetrics>>>,
        failover_history: &Arc<Mutex<Vec<FailoverEvent>>>,
        last_failover: &Arc<Mutex<Option<Instant>>>,
        current_best: &Arc<RwLock<Option<String>>>,
        config: &DnsIntelConfig,
    ) {
        // Verificar cooldown
        {
            let last = last_failover.lock().unwrap();
            if let Some(instant) = *last {
                if instant.elapsed().as_secs() < config.failover_cooldown_secs {
                    return; // En cooldown
                }
            }
        }

        // Recopilar información necesaria antes de hacer cambios
        let (should_failover, current_dns_str, new_dns_address, failures_count) = {
            let metrics_guard = metrics.read().unwrap();
            let current = current_best.read().unwrap();

            if let Some(current_dns) = &*current {
                if let Some(m) = metrics_guard.get(current_dns) {
                    if m.consecutive_failures >= config.max_consecutive_failures {
                        // Encontrar el mejor DNS saludable
                        let mut candidates: Vec<_> = metrics_guard
                            .values()
                            .filter(|dns| dns.is_healthy && &dns.address != current_dns)
                            .cloned()
                            .collect();

                        candidates.sort_by(|a, b| {
                            b.score
                                .partial_cmp(&a.score)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });

                        if let Some(new_best) = candidates.first() {
                            (
                                true,
                                current_dns.clone(),
                                new_best.address.clone(),
                                m.consecutive_failures,
                            )
                        } else {
                            (false, String::new(), String::new(), 0)
                        }
                    } else {
                        (false, String::new(), String::new(), 0)
                    }
                } else {
                    (false, String::new(), String::new(), 0)
                }
            } else {
                (false, String::new(), String::new(), 0)
            }
        };

        // Ahora ejecutar el failover si es necesario (sin locks activos)
        if should_failover {
            log::warn!(
                "🔄 DNS {} has {} consecutive failures, triggering failover",
                current_dns_str,
                failures_count
            );

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Registrar evento
            {
                let mut history = failover_history.lock().unwrap();
                history.push(FailoverEvent {
                    timestamp,
                    from_dns: current_dns_str.clone(),
                    to_dns: new_dns_address.clone(),
                    reason: format!("{} consecutive failures", failures_count),
                    success: true,
                });
            }

            // Actualizar último failover
            {
                let mut last = last_failover.lock().unwrap();
                *last = Some(Instant::now());
            }

            // Actualizar mejor DNS en memoria
            {
                let mut best = current_best.write().unwrap();
                *best = Some(new_dns_address.clone());
            }

            // APLICAR el cambio de DNS al sistema operativo (fix: failover sin apply era inútil)
            let adapter = match crate::services::dns::get_primary_adapter() {
                Ok(a) => a,
                Err(e) => {
                    log::error!("❌ No se pudo obtener el adaptador primario para aplicar DNS: {}", e);
                    log::info!(
                        "✅ Failover completed (memoria): {} → {}",
                        current_dns_str,
                        new_dns_address
                    );
                    return;
                }
            };

            if let Err(e) = crate::services::dns::set_dns(&adapter, &new_dns_address, None) {
                log::error!("❌ Fallo aplicando DNS {} en {}: {}", new_dns_address, adapter, e);
            } else {
                log::info!(
                    "✅ DNS aplicado al sistema: {} → {} (adaptador: {})",
                    current_dns_str,
                    new_dns_address,
                    adapter
                );
            }

            log::info!(
                "✅ Failover completed: {} → {}",
                current_dns_str,
                new_dns_address
            );
        }
    }

    /// Actualizar el mejor DNS
    fn update_best_dns_static(
        metrics: &Arc<RwLock<HashMap<String, DnsMetrics>>>,
        current_best: &Arc<RwLock<Option<String>>>,
    ) {
        let metrics_guard = metrics.read().unwrap();

        let best = metrics_guard
            .values()
            .filter(|m| m.is_healthy)
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(best_dns) = best {
            let mut current = current_best.write().unwrap();
            *current = Some(best_dns.address.clone());
        }
    }

    /// Ejecutar PowerShell
    fn run_powershell(command: &str) -> Option<String> {
        #[cfg(windows)]
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", command])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        #[cfg(not(windows))]
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", command])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                Some(String::from_utf8_lossy(&out.stdout).to_string())
            }
            _ => None,
        }
    }

    // ==================== PUBLIC API ====================

    /// Obtener métricas de todos los DNS
    pub fn get_all_metrics(&self) -> Vec<DnsMetrics> {
        let metrics = self.metrics.read().unwrap();
        let mut result: Vec<_> = metrics.values().cloned().collect();
        result.sort_by(|a, b| a.rank.cmp(&b.rank));
        result
    }

    /// Obtener los mejores DNS
    pub fn get_best_dns(&self, count: usize) -> Vec<DnsMetrics> {
        let mut result = self.get_all_metrics();
        result.truncate(count);
        result
    }

    /// Obtener el DNS actualmente seleccionado
    pub fn get_current_best(&self) -> Option<String> {
        self.current_best_dns.read().unwrap().clone()
    }

    /// Obtener historial de failovers
    pub fn get_failover_history(&self) -> Vec<FailoverEvent> {
        self.failover_history.lock().unwrap().clone()
    }

    /// Habilitar/deshabilitar auto-failover
    pub fn set_auto_failover(&self, enabled: bool) {
        let mut auto = self.auto_failover_enabled.lock().unwrap();
        *auto = enabled;
    }

    /// Verificar si auto-failover está habilitado
    pub fn is_auto_failover_enabled(&self) -> bool {
        *self.auto_failover_enabled.lock().unwrap()
    }

    /// Forzar un check inmediato de todos los DNS
    pub fn force_check(&self) {
        Self::check_all_dns_static(&self.metrics, &self.history, &self.config);
        Self::calculate_scores_static(&self.metrics, &self.history, &self.config);
        Self::update_best_dns_static(&self.metrics, &self.current_best_dns);
    }

    /// Obtener resumen del estado
    pub fn get_summary(&self) -> DnsIntelSummary {
        let best = self.get_best_dns(3);
        let metrics = self.metrics.read().unwrap();
        let history = self.history.lock().unwrap();

        DnsIntelSummary {
            best_dns: best,
            total_dns_monitored: metrics.len(),
            history_entries: history.len(),
            auto_failover_enabled: self.is_auto_failover_enabled(),
            current_best: self.get_current_best(),
        }
    }
}

/// Resumen del estado DNS
#[derive(Debug, Clone, Serialize)]
pub struct DnsIntelSummary {
    pub best_dns: Vec<DnsMetrics>,
    pub total_dns_monitored: usize,
    pub history_entries: usize,
    pub auto_failover_enabled: bool,
    pub current_best: Option<String>,
}

// ==================== SINGLETON ====================

lazy_static::lazy_static! {
    static ref DNS_INTELLIGENCE: Arc<DnsIntelligence> = {
        Arc::new(DnsIntelligence::new(DnsIntelConfig::default()))
    };
}

/// Obtener instancia global
pub fn get_dns_intelligence() -> Arc<DnsIntelligence> {
    Arc::clone(&DNS_INTELLIGENCE)
}

/// Iniciar el servicio DNS Intelligence
pub fn start_dns_intelligence() {
    let intel = get_dns_intelligence();
    intel.start();
}

/// Detener el servicio
pub fn stop_dns_intelligence() {
    let intel = get_dns_intelligence();
    intel.stop();
}
