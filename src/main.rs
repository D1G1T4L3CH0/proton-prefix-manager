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
use eframe::NativeOptions;

mod cli;
mod gui;
mod utils;
mod core;
mod error;

use cli::{Cli, Commands};
use gui::ProtonPrefixManagerApp;
use utils::output::determine_format;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Search { name, json, plain, delimiter }) => {
            let format = determine_format(*json, *plain, delimiter);
            cli::search::execute(name, &format);
        }
        Some(Commands::Prefix { appid, json, plain, delimiter }) => {
            let format = determine_format(*json, *plain, delimiter);
            cli::prefix::execute(*appid, &format);
        }
        Some(Commands::Open { appid }) => {
            cli::open::execute(*appid);
        }
        None => {
            log::info!("Launching GUI...");
            let native_options = NativeOptions::default();
            eframe::run_native(
                "Proton Prefix Manager",
                native_options,
                Box::new(|_cc| Ok(Box::new(ProtonPrefixManagerApp::new()))),
            ).expect("Failed to start GUI");
        }
    }
}