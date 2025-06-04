use std::path::PathBuf;

use crate::core::steam;
use crate::utils::backup as backup_utils;

pub fn execute(appid: u32, backup_path: PathBuf) {
    println!("♻️ Restoring Proton prefix for AppID: {}", appid);

    match steam::get_steam_libraries() {
        Ok(libraries) => {
            if let Some(prefix_path) = steam::find_proton_prefix(appid, &libraries) {
                match backup_utils::restore_prefix(&backup_path, &prefix_path) {
                    Ok(path) => println!("✅ Prefix restored to {}", path.display()),
                    Err(e) => eprintln!("❌ Failed to restore prefix: {}", e),
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
