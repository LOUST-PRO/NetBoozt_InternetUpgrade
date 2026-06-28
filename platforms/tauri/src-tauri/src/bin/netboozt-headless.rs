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

    // Shared flag for graceful shutdown — Arc allows both handlers to reference it
    let running = Arc::new(AtomicBool::new(true));
    let running_sigint = running.clone();
    let running_sigterm = running.clone();

    // Start DNS Intelligence service
    netboozt::start_dns_intelligence();

    // Register signal handlers for graceful shutdown using signal-hook
    signal_hook::flag::register(signal_hook::consts::SIGINT, running_sigint)
        .expect("Failed to register SIGINT handler");
    signal_hook::flag::register(signal_hook::consts::SIGTERM, running_sigterm)
        .expect("Failed to register SIGTERM handler");

    // Wait until signaled to stop
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_secs(1));
    }

    // Graceful shutdown
    log::info!("Stopping DNS Intelligence service...");
    netboozt::stop_dns_intelligence();
    log::info!("netboozt-headless stopped");
    process::exit(0);
}
