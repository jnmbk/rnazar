use crate::config::AppConfig;
use crate::i18n::Language;
use crate::messaging::start_desktop_messaging;
use crate::notify::show_crash_notification;
use crate::tray::{build_tray_icon, TrayMenuController};
use std::error::Error;
use std::sync::{Arc, RwLock};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};

#[derive(Debug)]
pub enum UserAppEvent {
    MenuEvent(muda::MenuEvent),
}

pub fn run_tray_application() -> Result<(), Box<dyn Error>> {
    let config = Arc::new(RwLock::new(AppConfig::load()));
    let initial_lang = config
        .read()
        .map(|c| c.language.clone())
        .unwrap_or(Language::Auto);
    let menu_controller = TrayMenuController::new(initial_lang)?;

    // Build event loop with custom user events
    let event_loop = EventLoopBuilder::<UserAppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Hook muda menu events to wake up tao event loop
    let menu_proxy = proxy.clone();
    muda::MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserAppEvent::MenuEvent(event));
    }));

    // Start desktop messaging listeners (D-Bus on Linux, IPC on all platforms)
    // Run notification directly from listener thread so headless tray event loop isn't blocked
    let msg_config = Arc::clone(&config);
    start_desktop_messaging(move |app_name| {
        let lang = msg_config
            .read()
            .map(|c| c.language.clone())
            .unwrap_or(Language::Auto);
        log::debug!(
            "Crash notification triggered for: {:?}",
            app_name.as_deref().unwrap_or("<unspecified>")
        );
        show_crash_notification(lang, app_name.as_deref());
    });

    // Build tray icon (keep instance alive for event loop lifetime)
    let _tray_icon = build_tray_icon(menu_controller.menu.clone())?;

    log::debug!("rnazar system tray active. Running event loop...");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                // Initial setup complete
            }
            Event::UserEvent(UserAppEvent::MenuEvent(menu_event)) => {
                let id = menu_event.id();
                if id == menu_controller.item_lang_auto.id() {
                    if let Ok(mut cfg) = config.write() {
                        cfg.language = Language::Auto;
                        cfg.save();
                        menu_controller.update_language(&cfg.language);
                    }
                } else if let Some(target) = menu_controller
                    .lang_items
                    .iter()
                    .find(|item| item.item.id() == id)
                {
                    if let Ok(mut cfg) = config.write() {
                        cfg.language = target.language.clone();
                        cfg.save();
                        menu_controller.update_language(&cfg.language);
                    }
                } else if id == menu_controller.item_about.id() {
                    let lang = config
                        .read()
                        .map(|c| c.language.clone())
                        .unwrap_or(Language::Auto);
                    log::debug!("About menu clicked, displaying about window...");
                    crate::about::show_about_window(lang);
                } else if id == menu_controller.item_exit.id() {
                    let lang = config
                        .read()
                        .map(|c| c.language.clone())
                        .unwrap_or(Language::Auto);

                    #[cfg(target_os = "linux")]
                    let confirmed = {
                        use gtk::prelude::*;
                        let dialog = gtk::MessageDialog::new(
                            gtk::Window::NONE,
                            gtk::DialogFlags::MODAL,
                            gtk::MessageType::Warning,
                            gtk::ButtonsType::YesNo,
                            lang.exit_confirm_message(),
                        );
                        dialog.set_title(lang.exit_confirm_title());
                        let response = dialog.run();
                        dialog.close();
                        response == gtk::ResponseType::Yes
                    };

                    #[cfg(not(target_os = "linux"))]
                    let confirmed = {
                        let res = rfd::MessageDialog::new()
                            .set_title(lang.exit_confirm_title())
                            .set_description(lang.exit_confirm_message())
                            .set_buttons(rfd::MessageButtons::YesNo)
                            .set_level(rfd::MessageLevel::Warning)
                            .show();
                        res == rfd::MessageDialogResult::Yes
                    };

                    if confirmed {
                        log::debug!("Exit menu clicked and confirmed! Exiting rnazar...");
                        *control_flow = ControlFlow::Exit;
                        std::process::exit(0);
                    }
                }
            }
            _ => {}
        }
    });
}
