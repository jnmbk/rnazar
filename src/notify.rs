use crate::i18n::Language;
use notify_rust::Notification;

#[cfg(target_os = "macos")]
fn show_macos_notification(title: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    let script = "on run argv\n  display notification (item 1 of argv) with title (item 2 of argv) sound name \"default\"\nend run";
    let output = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", script, body, title])
        .output()
        .or_else(|_| {
            std::process::Command::new("osascript")
                .args(["-e", script, body, title])
                .output()
        })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("osascript failed (status {}): {}", output.status, err.trim()).into());
    }
    Ok(())
}

pub fn show_crash_notification(language: Language, app_name: Option<&str>) {
    let title = language.crash_title();
    let body = language.crash_body(app_name);

    #[cfg(target_os = "macos")]
    {
        log::debug!(
            "Calling macOS native notification with title='{}', body='{}'",
            title,
            body
        );
        match show_macos_notification(title, &body) {
            Ok(()) => {
                log::debug!("Desktop notification shown successfully via osascript");
                return;
            }
            Err(e) => {
                log::warn!(
                    "osascript notification failed: {}, falling back to notify-rust",
                    e
                );
            }
        }
    }

    let mut notification = Notification::new();
    notification.summary(title).body(&body).appname("rnazar");

    #[cfg(all(unix, not(target_os = "macos")))]
    notification.icon("rnazar");

    log::debug!(
        "Calling notification.show() with title='{}', body='{}'",
        title,
        body
    );
    let res = notification.show();

    match res {
        #[cfg(all(unix, not(target_os = "macos")))]
        Ok(handle) => {
            log::debug!(
                "Desktop notification shown successfully (id={})",
                handle.id()
            );
            keep_handle_alive(handle);
        }
        #[cfg(not(all(unix, not(target_os = "macos"))))]
        Ok(_) => {
            log::debug!("Desktop notification shown successfully");
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
    let title = language.about_title();
    let body = language.about_body();

    #[cfg(target_os = "macos")]
    {
        match show_macos_notification(title, &body) {
            Ok(()) => return,
            Err(e) => log::warn!("osascript about notification failed: {}", e),
        }
    }

    let mut notification = Notification::new();
    notification.summary(title).body(&body).appname("rnazar");

    #[cfg(all(unix, not(target_os = "macos")))]
    notification.icon("rnazar");

    let res = notification.show();

    match res {
        #[cfg(all(unix, not(target_os = "macos")))]
        Ok(handle) => {
            keep_handle_alive(handle);
        }
        #[cfg(not(all(unix, not(target_os = "macos"))))]
        Ok(_) => {}
        Err(e) => log::warn!("Failed to show about notification: {}", e),
    }
}
