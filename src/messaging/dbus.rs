#[cfg(target_os = "linux")]
use std::sync::mpsc::Sender;
#[cfg(target_os = "linux")]
use zbus::interface;

#[cfg(target_os = "linux")]
pub struct DbusCrashService {
    tx: Sender<Option<String>>,
}

#[cfg(target_os = "linux")]
impl DbusCrashService {
    pub fn new(tx: Sender<Option<String>>) -> Self {
        Self { tx }
    }
}

#[cfg(target_os = "linux")]
#[interface(name = "org.rnazar.Notifier")]
impl DbusCrashService {
    /// D-Bus method: NotifyCrash(app_name: String)
    fn notify_crash(&self, app_name: String) {
        let app = if app_name.trim().is_empty() {
            None
        } else {
            Some(app_name)
        };
        let _ = self.tx.send(app);
    }
}

#[cfg(target_os = "linux")]
pub fn run_dbus_server(tx: Sender<Option<String>>) {
    std::thread::spawn(move || {
        let service = DbusCrashService::new(tx);
        match zbus::blocking::connection::Builder::session() {
            Ok(builder) => {
                let conn_res = builder
                    .name("org.rnazar.Notifier")
                    .and_then(|b| b.serve_at("/org/rnazar/Notifier", service))
                    .and_then(|b| b.build());

                match conn_res {
                    Ok(_conn) => {
                        log::debug!("D-Bus service 'org.rnazar.Notifier' registered successfully.");
                        // Keep connection alive in thread loop
                        loop {
                            std::thread::park();
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to register D-Bus service: {}", e);
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to connect to D-Bus session bus: {}", e);
            }
        }
    });
}

#[cfg(target_os = "linux")]
pub fn trigger_dbus_crash(app_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::session()?;
    let proxy = zbus::blocking::Proxy::new(
        &conn,
        "org.rnazar.Notifier",
        "/org/rnazar/Notifier",
        "org.rnazar.Notifier",
    )?;
    let app_str = app_name.unwrap_or("");
    proxy.call_method("NotifyCrash", &(app_str,))?;
    Ok(())
}
