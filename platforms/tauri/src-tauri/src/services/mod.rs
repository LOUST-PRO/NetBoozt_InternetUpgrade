//! Services Module
//!
//! Servicios compartidos para la aplicación.
//! Estos servicios son usados por los comandos y el system tray.
//!
//! By LOUST (www.loust.pro)

pub mod diagnostics;
pub mod dns;
pub mod dns_intelligence;
pub mod notifications;

// Re-export principales
pub use dns_intelligence::{
    get_dns_intelligence, start_dns_intelligence, stop_dns_intelligence, DnsIntelSummary,
    DnsMetrics, FailoverEvent,
};
