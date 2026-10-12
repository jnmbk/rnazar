pub mod dbus;
pub mod ipc;

use std::sync::mpsc::Sender;

pub fn start_desktop_messaging(tx: Sender<Option<String>>) {
    #[cfg(target_os = "linux")]
    dbus::run_dbus_server(tx.clone());

    ipc::start_ipc_listener(tx);
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
