use crate::core::steam;
use crate::utils::backup as backup_utils;

pub fn execute(appid: u32) {
    log::debug!("reset command: appid={}", appid);
    println!("\u{26a0}\u{fe0f} It's prudent to create a backup of your important data or configuration files before performing any critical actions. This ensures you can restore your system to a known good state if something unexpected happens.");
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
