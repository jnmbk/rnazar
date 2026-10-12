use crate::i18n::Language;
use notify_rust::Notification;

pub fn show_crash_notification(language: Language, app_name: Option<&str>) {
    let title = language.crash_title();
    let body = language.crash_body(app_name);

    let res = Notification::new()
        .summary(title)
        .body(&body)
        .appname("rnazar")
        .icon("rnazar")
        .show();

    if let Err(e) = res {
        eprintln!("Warning: Failed to show desktop notification: {}", e);
    }
}

pub fn show_about_notification(language: Language) {
    let title = language.about_title();
    let body = language.about_body();

    let res = Notification::new()
        .summary(title)
        .body(&body)
        .appname("rnazar")
        .icon("rnazar")
        .show();

    if let Err(e) = res {
        eprintln!("Warning: Failed to show about notification: {}", e);
    }
}
