use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod backup;
pub mod clear_cache;
pub mod delete_backup;
pub mod list_backups;
pub mod open;
pub mod prefix;
pub mod reset;
pub mod restore;
pub mod search;

/// Proton Prefix Manager CLI
///
/// A tool to find and manage Proton prefixes for Steam games.
/// Run without arguments to launch the GUI.
/// Each command has its own options - use --help with a command to see them.
#[derive(Parser)]
#[command(name = "proton-prefix-manager")]
#[command(about = "Find and manage Proton prefixes easily", long_about = None)]
pub struct Cli {
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
        /// The Steam App ID of the game
        appid: u32,

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
}
