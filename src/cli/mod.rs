use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod backup;
pub mod clear_cache;
pub mod config;
pub mod config_paths;
pub mod delete_backup;
pub mod list_backups;
pub mod open;
pub mod prefix;
pub mod protontricks;
pub mod reset;
pub mod restore;
pub mod search;
pub mod userdata;
pub mod winecfg;

/// Proton Prefix Manager CLI
///
/// A tool to find and manage Proton prefixes for Steam games.
/// Run without arguments to launch the GUI.
/// Each command has its own options - use --help with a command to see them.
#[derive(Parser)]
#[command(name = "proton-prefix-manager")]
#[command(about = "Find and manage Proton prefixes easily", long_about = None)]
pub struct Cli {
    /// Enable debug logging
    #[arg(long, short, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search for a game by name (supports --json, --plain, --delimiter output options)
    Search {
        /// The name of the game to search for
        name: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Output in plain format (no formatting or emojis)
        #[arg(long)]
        plain: bool,

        /// Specify custom delimiter for output
        #[arg(long)]
        delimiter: Option<String>,
    },

    /// Find the Proton prefix for an installed game (supports --json, --plain, --delimiter output options)
    Prefix {
        /// The Steam App ID of the game
        appid: u32,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Output in plain format (no formatting or emojis)
        #[arg(long)]
        plain: bool,

        /// Specify custom delimiter for output
        #[arg(long)]
        delimiter: Option<String>,
    },

    /// Open the Proton prefix in the file manager
    Open {
        /// The Steam App ID of the game
        appid: u32,
    },

    /// Open the Steam userdata directory for the given App ID
    Userdata {
        /// The Steam App ID of the game
        appid: u32,
    },

    /// Back up the Proton prefix to the default backup location
    Backup {
        /// The Steam App ID of the game
        appid: u32,
    },

    /// Restore the Proton prefix from a backup directory
    Restore {
        /// The Steam App ID of the game
        appid: u32,

        /// Path to the backup directory
        path: PathBuf,
    },

    /// List backups for the given App ID
    ListBackups {
        /// The Steam App ID of the game
        appid: u32,
    },

    /// Delete a specific backup
    DeleteBackup {
        /// Path to the backup directory
        backup: PathBuf,
    },

    /// Delete the existing prefix
    Reset {
        /// The Steam App ID of the game
        appid: u32,
    },

    /// Clear the shader cache for the given App ID
    ClearCache {
        /// The Steam App ID of the game
        appid: u32,
    },

    /// Run protontricks for the given App ID
    Protontricks {
        /// The Steam App ID of the game
        appid: u32,

        /// Additional arguments for protontricks
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Launch winecfg for the given App ID
    Winecfg {
        /// The Steam App ID of the game
        appid: u32,
    },

    /// Edit game configuration in the manifest
    Config {
        /// The Steam App ID of the game
        appid: u32,

        /// Set custom launch options
        #[arg(long)]
        launch: Option<String>,

        /// Force a specific Proton version
        #[arg(long)]
        proton: Option<String>,

        /// Enable or disable Steam Cloud
        #[arg(long)]
        cloud: Option<bool>,

        /// Auto update behavior
        #[arg(long)]
        auto_update: Option<String>,
    },

    /// Show paths to discovered localconfig.vdf files
    ConfigPaths,
}
