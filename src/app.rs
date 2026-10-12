use crate::config::AppConfig;
use crate::i18n::Language;
use crate::messaging::start_desktop_messaging;
use crate::notify::show_crash_notification;
use crate::tray::{build_tray_icon, TrayMenuController};
use std::error::Error;
use std::sync::mpsc::channel;
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};

#[derive(Debug)]
pub enum UserAppEvent {
    CrashReport(Option<String>),
    MenuEvent,
}

pub fn run_tray_application() -> Result<(), Box<dyn Error>> {
    let mut config = AppConfig::load();
    let menu_controller = TrayMenuController::new(config.language)?;

    // Build event loop with custom user events
    let event_loop = EventLoopBuilder::<UserAppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Hook muda menu events to wake up tao event loop
    let menu_proxy = proxy.clone();
    muda::MenuEvent::set_event_handler(Some(move |_| {
        let _ = menu_proxy.send_event(UserAppEvent::MenuEvent);
    }));

    // Start desktop messaging listeners (D-Bus on Linux, IPC on all platforms)
    let (tx, rx) = channel::<Option<String>>();
    start_desktop_messaging(tx);

    // Forward incoming messaging crashes to event loop
    let crash_proxy = proxy.clone();
    std::thread::spawn(move || {
        while let Ok(app_name) = rx.recv() {
            let _ = crash_proxy.send_event(UserAppEvent::CrashReport(app_name));
        }
    });

    // Build tray icon (keep instance alive for event loop lifetime)
    let _tray_icon = build_tray_icon(menu_controller.menu.clone())?;

    println!("rnazar system tray active. Running event loop...");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                // Initial setup complete
            }
            Event::UserEvent(UserAppEvent::CrashReport(app_name)) => {
                println!(
                    "Crash notification triggered for: {:?}",
                    app_name.as_deref().unwrap_or("<unspecified>")
                );
                show_crash_notification(config.language, app_name.as_deref());
            }
            Event::UserEvent(UserAppEvent::MenuEvent) => {
                while let Ok(menu_event) = muda::MenuEvent::receiver().try_recv() {
                    let id = menu_event.id();
                    if id == menu_controller.item_lang_auto.id() {
                        config.language = Language::Auto;
                        config.save();
                        menu_controller.update_language(config.language);
                    } else if id == menu_controller.item_lang_en.id() {
                        config.language = Language::English;
                        config.save();
                        menu_controller.update_language(config.language);
                    } else if id == menu_controller.item_lang_tr.id() {
                        config.language = Language::Turkish;
                        config.save();
                        menu_controller.update_language(config.language);
                    } else if id == menu_controller.item_about.id() {
                        crate::about::show_about_window(config.language);
                    } else if id == menu_controller.item_exit.id() {
                        println!("Exiting rnazar...");
                        *control_flow = ControlFlow::Exit;
                        std::process::exit(0);
                    }
                }
            }
            _ => {}
        }
    });
}
