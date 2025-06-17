use crate::core::steam;
use crate::utils::backup as backup_utils;

pub fn execute(appid: u32) {
    log::debug!("reset command: appid={}", appid);
    match steam::get_steam_libraries() {
        Ok(libraries) => {
            if let Some(prefix) = steam::find_proton_prefix(appid, &libraries) {
                match backup_utils::reset_prefix(&prefix) {
                    Ok(_) => println!("Prefix deleted"),
                    Err(e) => eprintln!("Failed to delete prefix: {}", e),
                }
            } else {
                println!("Prefix not found for {}", appid);
            }
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }
}
