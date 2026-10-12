use crate::i18n::Language;
use notify_rust::Notification;

pub fn show_crash_notification(language: Language, app_name: Option<&str>) {
    #[cfg(target_os = "macos")]
    {
        let bundle = notify_rust::get_bundle_identifier_or_default("org.rnazar.app");
        let _ = notify_rust::set_application(&bundle);
    }

    let title = language.crash_title();
    let body = language.crash_body(app_name);

    let mut notification = Notification::new();
    notification.summary(title).body(&body).appname("rnazar");

    #[cfg(all(unix, not(target_os = "macos")))]
    notification.icon("rnazar");

    log::debug!("Calling notification.show() with title='{}', body='{}'", title, body);
    let res = notification.show();

    match res {
        Ok(handle) => {
            log::debug!("Desktop notification shown successfully (id={})", handle.id());
            #[cfg(all(unix, not(target_os = "macos")))]
            keep_handle_alive(handle);
        }
        Err(e) => log::warn!("Failed to show desktop notification: {}", e),
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn keep_handle_alive(handle: notify_rust::NotificationHandle) {
    use std::sync::{Mutex, OnceLock};
    static HANDLES: OnceLock<Mutex<Vec<notify_rust::NotificationHandle>>> = OnceLock::new();
    if let Ok(mut list) = HANDLES.get_or_init(|| Mutex::new(Vec::new())).lock() {
        // Keep up to 16 recent handles alive (especially the first one, which GNOME Shell watches)
        if list.len() >= 16 {
            list.remove(1);
        }
        list.push(handle);
    }
}

pub fn show_about_notification(language: Language) {
    #[cfg(target_os = "macos")]
    {
        let bundle = notify_rust::get_bundle_identifier_or_default("org.rnazar.app");
        let _ = notify_rust::set_application(&bundle);
    }

    let title = language.about_title();
    let body = language.about_body();

    let mut notification = Notification::new();
    notification.summary(title).body(&body).appname("rnazar");

    #[cfg(all(unix, not(target_os = "macos")))]
    notification.icon("rnazar");

    let res = notification.show();

    match res {
        Ok(handle) => {
            #[cfg(all(unix, not(target_os = "macos")))]
            keep_handle_alive(handle);
        }
        Err(e) => log::warn!("Failed to show about notification: {}", e),
    }
}
