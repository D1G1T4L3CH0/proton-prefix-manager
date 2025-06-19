use crate::core::steam;
use crate::utils::prefix_repair;

pub fn execute(appid: u32) {
    log::debug!("repair command: appid={}", appid);
    println!("Attempting to repair prefix for {}...", appid);
    match steam::get_steam_libraries() {
        Ok(libs) => {
            if let Some(prefix) = steam::find_proton_prefix(appid, &libs) {
                match prefix_repair::repair_prefix(&prefix) {
                    Ok(_) => println!("Prefix repaired"),
                    Err(e) => eprintln!("Failed to repair prefix: {}", e),
                }
            } else {
                println!("Prefix not found for {}", appid);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
