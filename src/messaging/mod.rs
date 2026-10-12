pub mod dbus;
pub mod ipc;
pub mod monitor;

use std::sync::Arc;

pub type CrashCallback = Arc<dyn Fn(Option<String>) + Send + Sync>;

pub fn start_desktop_messaging<F>(callback: F)
where
    F: Fn(Option<String>) + Send + Sync + 'static,
{
    let cb: CrashCallback = Arc::new(callback);

    #[cfg(target_os = "linux")]
    dbus::run_dbus_server(cb.clone());

    ipc::start_ipc_listener(cb.clone());
    monitor::start_crash_monitor(cb);
}

pub fn trigger_crash_notification(app: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        if dbus::trigger_dbus_crash(app).is_ok() {
            return Ok(());
        }
    }

    if ipc::trigger_ipc_crash(app).is_ok() {
        return Ok(());
    }

    // Fallback: If no background daemon is reachable, trigger directly via desktop notification
    let config = crate::config::AppConfig::load();
    crate::notify::show_crash_notification(config.language, app);
    Ok(())
}
