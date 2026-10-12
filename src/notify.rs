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

    let res = notification.show();

    if let Err(e) = res {
        log::warn!("Failed to show desktop notification: {}", e);
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

    if let Err(e) = res {
        log::warn!("Failed to show about notification: {}", e);
    }
}
