//! `netboozt-headless` — headless DNS failover daemon.
//!
//! Starts the DNS Intelligence background service and waits for SIGINT/SIGTERM
//! to shut down gracefully. Designed to run as a systemd user service
//! (`netboozt-dns.service`) so DNS failover survives lid-close / logout on Linux.
//!
//! Usage:
//!     netboozt-headless
//!
//! The binary reads no configuration — it uses `DnsIntelConfig::default()`.
//! Override env vars (future): `NETBOOZT_CHECK_INTERVAL`, `NETBOOZT_PARALLEL_WORKERS`.

use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("netboozt-headless v{} starting", env!("CARGO_PKG_VERSION"));

    // Shared flag for graceful shutdown — Arc allows both handlers to reference it.
    //
    // SEMÁNTICA DE signal-hook (ver signal-hook-0.3.18 src/flag.rs:153):
    //   `signal_hook::flag::register` hace `flag.store(true, SeqCst)` — el
    //   flag pasa a TRUE cuando LLEGA la señal. El patrón canónico es por
    //   eso un flag que empieza en FALSE y un bucle `while !flag.load()`.
    //
    // BUG CORREGIDO (2026-10-25): este binario usaba `running = true` con
    //   `while running.load()`. La señal ponía el flag en TRUE, que es
    //   EXACTAMENTE el valor que el bucle esperaba → el shutdown no podía
    //   dispararse NUNCA. Medido: `kill -TERM <pid>` no hacía nada (el
    //   handler corría — SigCgt bit 14 — pero no cambiaba la condición de
    //   salida), y systemd tenía que SIGKILLear a los TimeoutStopSec=30.
    //   El worker de DNS tampoco es el culpable: el hilo principal seguía
    //   en hrtimer_nanosleep durante los 30 s completos.
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let shutdown_sigint = shutdown_requested.clone();
    let shutdown_sigterm = shutdown_requested.clone();

    // Start DNS Intelligence service
    netboozt::start_dns_intelligence();

    // Register signal handlers for graceful shutdown using signal-hook
    signal_hook::flag::register(signal_hook::consts::SIGINT, shutdown_sigint)
        .expect("Failed to register SIGINT handler");
    signal_hook::flag::register(signal_hook::consts::SIGTERM, shutdown_sigterm)
        .expect("Failed to register SIGTERM handler");

    // Wait until signaled to stop
    while !shutdown_requested.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_secs(1));
    }

    // Graceful shutdown
    log::info!("Stopping DNS Intelligence service...");
    netboozt::stop_dns_intelligence();
    log::info!("netboozt-headless stopped");
    process::exit(0);
}
