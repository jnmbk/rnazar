use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(
    name = "rnazar",
    version,
    about = "System tray evil eye ward and crash notifier",
    long_about = "A cross-platform system tray application that wards off bad luck\nand alerts you with 'an evil eye struck and it was taken care of' when an application crashes."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Quick trigger flag for reporting an application crash
    #[arg(long, value_name = "APP_NAME")]
    pub crash: Option<String>,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Commands {
    /// Trigger a crash notification via desktop environment messaging (D-Bus / IPC)
    Notify {
        /// Optional name of the crashed application
        #[arg(short, long, value_name = "APP_NAME")]
        app: Option<String>,
    },
}

