use clap::{Parser, Subcommand};
use rnazar::app::run_tray_application;
use rnazar::messaging::trigger_crash_notification;

#[derive(Parser, Debug)]
#[command(
    name = "rnazar",
    version,
    about = "System tray evil eye ward and crash notifier",
    long_about = "A cross-platform system tray application that wards off bad luck\nand alerts you with 'nazar değdi ve icabına bakıldı' when an application crashes."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Quick trigger flag for reporting an application crash
    #[arg(long, value_name = "APP_NAME")]
    crash: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Trigger a crash notification via desktop environment messaging (D-Bus / IPC)
    Notify {
        /// Optional name of the crashed application
        #[arg(short, long, value_name = "APP_NAME")]
        app: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
