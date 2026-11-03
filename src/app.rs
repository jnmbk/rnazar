use crate::config::AppConfig;
use crate::i18n::Language;
use crate::messaging::monitor::CrashMonitorController;
use crate::messaging::start_desktop_messaging;
use crate::notify::show_crash_notification;
use crate::tray::{build_tray_icon, load_tray_icon, TrayMenuController};
use std::error::Error;
use std::sync::{Arc, RwLock};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::TrayIcon;

#[derive(Debug)]
pub enum UserAppEvent {
    MenuEvent(muda::MenuEvent),
}

fn confirm_exit(lang: &Language) -> bool {
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::*;
        let dialog = gtk::MessageDialog::new(
            gtk::Window::NONE,
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Warning,
            gtk::ButtonsType::None,
            lang.exit_confirm_message(),
        );
        dialog.set_title(lang.exit_confirm_title());
        dialog.add_button(lang.dialog_no(), gtk::ResponseType::No);
        dialog.add_button(lang.dialog_yes(), gtk::ResponseType::Yes);
        let response = dialog.run();
        dialog.close();
        response == gtk::ResponseType::Yes
    }

    #[cfg(not(target_os = "linux"))]
    {
        let res = rfd::MessageDialog::new()
            .set_title(lang.exit_confirm_title())
            .set_description(lang.exit_confirm_message())
            .set_buttons(rfd::MessageButtons::YesNo)
            .set_level(rfd::MessageLevel::Warning)
            .show();
        res == rfd::MessageDialogResult::Yes
    }
}

fn handle_menu_event(
    event: muda::MenuEvent,
    config: &Arc<RwLock<AppConfig>>,
    menu_controller: &TrayMenuController,
    tray_icon: &TrayIcon,
    monitor_controller: &CrashMonitorController,
    control_flow: &mut ControlFlow,
) {
    let id = event.id();
    if id == menu_controller.item_protection.id() {
        let new_state = !config.read().map_or(true, |c| c.protection);
        if let Ok(mut cfg) = config.write() {
            cfg.protection = new_state;
            cfg.save();
        }
        menu_controller.update_protection(new_state);
        if let Ok(icon) = load_tray_icon(new_state) {
            let _ = tray_icon.set_icon(Some(icon));
        }
        let current_lang = config.read().map_or(Language::Auto, |c| c.language.clone());
        let _ = tray_icon.set_tooltip(Some(current_lang.tray_tooltip(new_state)));
        if new_state {
            log::debug!("Protection enabled: starting crash log monitor");
            monitor_controller.start();
        } else {
            log::debug!("Protection disabled: stopping crash log monitor");
            monitor_controller.stop();
        }
    } else if id == menu_controller.item_lang_auto.id() {
        if let Ok(mut cfg) = config.write() {
            cfg.language = Language::Auto;
            cfg.save();
            menu_controller.update_language(&cfg.language);
            let _ = tray_icon.set_tooltip(Some(cfg.language.tray_tooltip(cfg.protection)));
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
            let _ = tray_icon.set_tooltip(Some(cfg.language.tray_tooltip(cfg.protection)));
        }
    } else if id == menu_controller.item_about.id() {
        let lang = config.read().map_or(Language::Auto, |c| c.language.clone());
        log::debug!("About menu clicked, displaying about window...");
        crate::about::show_about_window(lang);
    } else if id == menu_controller.item_exit.id() {
        let lang = config.read().map_or(Language::Auto, |c| c.language.clone());

        if confirm_exit(&lang) {
            log::debug!("Exit menu clicked and confirmed! Exiting rnazar...");
            monitor_controller.stop();
            crate::messaging::ipc::cleanup_ipc_socket();
            *control_flow = ControlFlow::Exit;
        }
    }
}

/// Runs the main system tray application and starts crash monitoring.
///
/// # Errors
///
/// Returns an error if the tray menu, event loop, or tray icon cannot be initialized.
pub fn run_tray_application() -> Result<(), Box<dyn Error>> {
    let config = Arc::new(RwLock::new(AppConfig::load()));
    let (initial_lang, initial_protection) = config.read().map_or((Language::Auto, true), |c| {
        (c.language.clone(), c.protection)
    });
    let menu_controller = TrayMenuController::new(initial_lang.clone(), initial_protection)?;

    // Build event loop with custom user events
    let event_loop = EventLoopBuilder::<UserAppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Hook muda menu events to wake up tao event loop
    muda::MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserAppEvent::MenuEvent(event));
    }));

    // Start desktop messaging listeners (D-Bus on Linux, IPC on all platforms)
    // Run notification directly from listener thread so headless tray event loop isn't blocked
    let msg_config = Arc::clone(&config);
    let monitor_controller = start_desktop_messaging(initial_protection, move |app_name| {
        let (lang, protection) = msg_config.read().map_or((Language::Auto, true), |c| {
            (c.language.clone(), c.protection)
        });
        if !protection {
            log::debug!("Protection is disabled: ignoring crash notification");
            return;
        }
        log::debug!(
            "Crash notification triggered for: {:?}",
            app_name.as_deref().unwrap_or("<unspecified>")
        );
        show_crash_notification(lang, app_name.as_deref());
    });

    // Build tray icon (keep instance alive for event loop lifetime)
    let tray_icon = build_tray_icon(
        menu_controller.menu.clone(),
        &initial_lang,
        initial_protection,
    )?;

    log::debug!("rnazar system tray active. Running event loop...");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::UserEvent(UserAppEvent::MenuEvent(menu_event)) = event {
            handle_menu_event(
                menu_event,
                &config,
                &menu_controller,
                &tray_icon,
                &monitor_controller,
                control_flow,
            );
        }
    });
}
