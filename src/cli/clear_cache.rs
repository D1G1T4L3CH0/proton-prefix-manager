use crate::core::steam;
use crate::utils::backup as backup_utils;

pub fn execute(appid: u32) {
    log::debug!("clear-cache command: appid={}", appid);
    match steam::get_steam_libraries() {
        Ok(libs) => match backup_utils::clear_shader_cache(appid, &libs) {
            Ok(_) => println!("Shader cache cleared"),
            Err(e) => eprintln!("Failed to clear shader cache: {}", e),
        },
        Err(e) => eprintln!("❌ Error: {}", e),
    }
}
