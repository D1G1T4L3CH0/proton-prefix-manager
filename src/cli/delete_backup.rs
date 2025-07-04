use std::path::PathBuf;

use crate::core::steam;
use crate::utils::backup as backup_utils;

pub fn execute(backup: PathBuf) {
    log::debug!("delete-backup command: path={}", backup.display());
    match steam::get_steam_libraries() {
        Ok(_libs) => match backup_utils::delete_backup(&backup) {
            Ok(_) => println!("Deleted backup {}", backup.display()),
            Err(e) => eprintln!("Failed to delete backup: {}", e),
        },
        Err(err) => eprintln!("❌ Error: {}", err),
    }
}
