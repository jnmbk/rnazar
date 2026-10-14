#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use rnazar::app::run_tray_application;
use rnazar::cli::{Cli, Commands};
use rnazar::messaging::trigger_crash_notification;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("NO_AT_BRIDGE", "1");
        std::env::set_var("MALLOC_ARENA_MAX", "2");
        std::env::set_var("MALLOC_TRIM_THRESHOLD_", "65536");
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let cli = Cli::parse();

    if let Some(app_name) = cli.crash {
        log::debug!("Triggering crash notification for: {}", app_name);
        trigger_crash_notification(Some(&app_name))?;
        return Ok(());
    }

    if let Some(Commands::Notify { app }) = cli.command {
        log::debug!(
            "Triggering crash notification for: {}",
            app.as_deref().unwrap_or("<unspecified>")
        );
        trigger_crash_notification(app.as_deref())?;
        return Ok(());
    }

    // Default action: Launch the system tray daemon
    run_tray_application()
}
