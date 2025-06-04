use crate::core::steam;

pub fn execute(appid: u32) {
    println!("📂 Opening Proton prefix for AppID: {}", appid);
    
    match steam::get_steam_libraries() {
        Ok(libraries) => {
            if let Some(prefix_path) = steam::find_proton_prefix(appid, &libraries) {
                println!("🗂  Opening folder: {}", prefix_path.display());
                if let Err(e) = open::that(&prefix_path) {
                    eprintln!("❌ Failed to open folder: {}", e);
                }
            } else {
                println!("❌ Proton prefix not found for AppID: {}", appid);
            }
        },
        Err(err) => {
            eprintln!("❌ Error: {}", err);
        }
    }
} 