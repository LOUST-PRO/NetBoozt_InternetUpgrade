//! `netboozt-service` — Windows Service wrapper for DNS failover.
//!
//! Usage (after install):
//!     sc start netboozt-dns
//!     sc stop netboozt-dns
//!
//! Build on Windows with:
//!     cargo build --release --bin netboozt-service

fn main() {
    #[cfg(windows)]
    {
        netboozt_service_win::run_service();
    }

    #[cfg(not(windows))]
    {
        eprintln!("netboozt-service is only available on Windows.");
        eprintln!("On Linux, use netboozt-headless with systemd instead.");
        std::process::exit(1);
    }
}
