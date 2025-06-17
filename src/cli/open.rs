use crate::core::steam;

#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(not(test))]
fn open_path(path: &std::path::Path) -> std::io::Result<()> {
    open::that(path)
}

#[cfg(test)]
pub static OPENED_PATHS: Lazy<Mutex<Vec<std::path::PathBuf>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(test)]
fn open_path(path: &std::path::Path) -> std::io::Result<()> {
    OPENED_PATHS.lock().unwrap().push(path.to_path_buf());
    Ok(())
}

pub fn execute(appid: u32) {
    log::debug!("open command: appid={}", appid);
    println!("📂 Opening Proton prefix for AppID: {}", appid);
    
    match steam::get_steam_libraries() {
        Ok(libraries) => {
            if let Some(prefix_path) = steam::find_proton_prefix(appid, &libraries) {
                println!("🗂  Opening folder: {}", prefix_path.display());
                if let Err(e) = open_path(&prefix_path) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use crate::test_helpers::{TEST_MUTEX, setup_steam_env};

    #[test]
    fn test_execute_opens_prefix() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 5555;
        let (home, prefix, _) = setup_steam_env(appid, false);
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        OPENED_PATHS.lock().unwrap().clear();
        execute(appid);

        let opened = OPENED_PATHS.lock().unwrap();
        assert_eq!(opened.len(), 1);
        assert_eq!(opened[0], prefix);

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }

    #[test]
    fn test_execute_no_prefix() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 6666;
        let (home, prefix, _) = setup_steam_env(appid, false);
        fs::remove_dir_all(&prefix).unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        OPENED_PATHS.lock().unwrap().clear();
        execute(appid);

        let opened = OPENED_PATHS.lock().unwrap();
        assert!(opened.is_empty());

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }
}
