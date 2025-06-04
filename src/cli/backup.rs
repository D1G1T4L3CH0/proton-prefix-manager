use std::path::PathBuf;

use crate::core::steam;
use crate::utils::backup as backup_utils;

pub fn execute(appid: u32, destination: PathBuf) {
    println!("📦 Backing up Proton prefix for AppID: {}", appid);

    match steam::get_steam_libraries() {
        Ok(libraries) => {
            if let Some(prefix_path) = steam::find_proton_prefix(appid, &libraries) {
                match backup_utils::backup_prefix(&prefix_path, &destination) {
                    Ok(path) => println!("✅ Backup created at {}", path.display()),
                    Err(e) => eprintln!("❌ Failed to back up prefix: {}", e),
                }
            } else {
                println!("❌ Proton prefix not found for AppID: {}", appid);
            }
        }
        Err(err) => {
            eprintln!("❌ Error: {}", err);
        }
    }
}
