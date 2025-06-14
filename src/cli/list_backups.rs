use crate::core::steam;
use crate::utils::backup as backup_utils;

pub fn execute(appid: u32) {
    match steam::get_steam_libraries() {
        Ok(_libs) => {
            let backups = backup_utils::list_backups(appid);
            if backups.is_empty() {
                println!("No backups found");
            } else {
                for b in backups {
                    println!("{}", b.display());
                }
            }
        }
        Err(err) => eprintln!("❌ Error: {}", err),
    }
}
