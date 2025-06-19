//! # Proton Prefix Manager
//!
//! A tool to find and manage Proton prefixes for Steam games on Linux.
//!
//! ## Features
//!
//! - Search for installed Steam games
//! - Find Proton prefixes for specific games
//! - Open prefixes in your file manager
//! - GUI and CLI interfaces
//! - Back up and restore prefixes
//!
//! ## Usage
//!
//! Run without arguments to launch the GUI:
//!
//! ```
//! proton-prefix-manager
//! ```
//!
//! Search for games by name:
//!
//! ```
//! proton-prefix-manager search "portal"
//! ```
//!
//! Find a prefix for a specific AppID:
//!
//! ```
//! proton-prefix-manager prefix 620
//! ```
//!
//! Open a prefix in your file manager:
//!
//! ```
//! proton-prefix-manager open 620
//! ```

use clap::Parser;
use eframe::{egui, NativeOptions};

mod cli;
mod core;
mod error;
mod gui;
mod utils;

#[cfg(test)]
mod test_helpers;

use cli::{Cli, Commands};
use gui::ProtonPrefixManagerApp;
use utils::logging;
use utils::output::determine_format;

fn main() {
    let cli = Cli::parse();
    logging::init(cli.debug);

    match &cli.command {
        Some(Commands::Search {
            name,
            json,
            plain,
            delimiter,
        }) => {
            let format = determine_format(*json, *plain, delimiter);
            cli::search::execute(name, &format);
        }
        Some(Commands::Prefix {
            appid,
            json,
            plain,
            delimiter,
        }) => {
            let format = determine_format(*json, *plain, delimiter);
            cli::prefix::execute(*appid, &format);
        }
        Some(Commands::Open { appid }) => {
            cli::open::execute(*appid);
        }
        Some(Commands::Userdata { appid }) => {
            cli::userdata::execute(*appid);
        }
        Some(Commands::Backup { appid }) => {
            cli::backup::execute(*appid);
        }
        Some(Commands::Restore { appid, path }) => {
            cli::restore::execute(*appid, path.clone());
        }
        Some(Commands::ListBackups { appid }) => {
            cli::list_backups::execute(*appid);
        }
        Some(Commands::DeleteBackup { backup }) => {
            cli::delete_backup::execute(backup.clone());
        }
        Some(Commands::Repair { appid }) => {
            cli::repair::execute(*appid);
        }
        Some(Commands::Reset { appid }) => {
            cli::reset::execute(*appid);
        }
        Some(Commands::ClearCache { appid }) => {
            cli::clear_cache::execute(*appid);
        }
        Some(Commands::Protontricks { appid, args }) => {
            cli::protontricks::execute(*appid, args);
        }
        Some(Commands::Winecfg { appid }) => {
            cli::winecfg::execute(*appid);
        }
        Some(Commands::Config {
            appid,
            launch,
            proton,
            cloud,
            auto_update,
        }) => {
            cli::config::execute(
                *appid,
                launch.clone(),
                proton.clone(),
                *cloud,
                auto_update.clone(),
            );
        }
        Some(Commands::ConfigPaths) => {
            cli::config_paths::execute();
        }
        None => {
            log::info!("Launching GUI...");
            let mut native_options = NativeOptions::default();
            native_options.viewport = native_options
                .viewport
                .with_decorations(true)
                .with_inner_size(egui::vec2(1200.0, 800.0));
            // Let the OS decide where to place the window
            native_options.persist_window = false;
            eframe::run_native(
                "Proton Prefix Manager",
                native_options,
                Box::new(|_cc| Ok(Box::new(ProtonPrefixManagerApp::new()))),
            )
            .expect("Failed to start GUI");
        }
    }
}
