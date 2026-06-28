//! Windows Service implementation — only compiles on Windows.

#[cfg(windows)]
pub fn run_service() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

    static STOP_EVENT: AtomicBool = AtomicBool::new(false);

    fn service_main(_arguments: Vec<std::ffi::OsString>) {
        let status_handle =
            service_control_handler::register("netboozt-dns", |event| match event {
                ServiceControl::Stop => {
                    STOP_EVENT.store(true, Ordering::SeqCst);
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::Default,
            })
            .expect("Failed to register service control handler");

        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
            })
            .expect("Failed to set service status");

        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .init();

        log::info!(
            "netboozt-service v{} starting as Windows Service",
            env!("CARGO_PKG_VERSION")
        );

        std::thread::spawn(|| {
            netboozt::start_dns_intelligence();
        });

        while !STOP_EVENT.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_secs(1));
        }

        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::StopPending,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
            })
            .ok();

        log::info!("Stopping DNS Intelligence service...");
        netboozt::stop_dns_intelligence();
        log::info!("netboozt-service stopped");
    }

    windows_service::define_windows_service!(ffi_service_main, service_main);
}

#[cfg(not(windows))]
pub fn run_service() {
    // Stub: this function is never called on non-Windows platforms.
    // The main binary already exits early on non-Windows.
}
