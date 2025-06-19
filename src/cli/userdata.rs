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
pub static OPENED_PATHS: Lazy<Mutex<Vec<std::path::PathBuf>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
#[cfg(test)]
fn open_path(path: &std::path::Path) -> std::io::Result<()> {
    OPENED_PATHS.lock().unwrap().push(path.to_path_buf());
    Ok(())
}

pub fn execute(appid: u32) {
    log::debug!("userdata command: appid={}", appid);
    println!("📂 Opening userdata for AppID: {}", appid);

    match steam::find_userdata_dir(appid) {
        Some(path) => {
            println!("🗂  Opening folder: {}", path.display());
            if let Err(e) = open_path(&path) {
                eprintln!("❌ Failed to open folder: {}", e);
            }
        }
        None => {
            println!("❌ Userdata folder not found for AppID: {}", appid);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{setup_steam_env, TEST_MUTEX};
    use std::fs;

    #[test]
    fn test_execute_opens_userdata() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 777777;
        let (home, _prefix, _login_opt) = setup_steam_env(appid, true);
        let userdata_base = home.path().join(".steam/steam/userdata/111111111");
        fs::create_dir_all(userdata_base.join(appid.to_string())).unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        OPENED_PATHS.lock().unwrap().clear();
        execute(appid);

        let opened = OPENED_PATHS.lock().unwrap();
        assert_eq!(opened.len(), 1);
        assert_eq!(opened[0], userdata_base.join(appid.to_string()));

        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        }
    }

    #[test]
    fn test_execute_no_userdata() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 888888;
        let (home, _prefix, _login_opt) = setup_steam_env(appid, true);
        let userdata_base = home.path().join(".steam/steam/userdata/111111111");
        fs::create_dir_all(&userdata_base).unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        OPENED_PATHS.lock().unwrap().clear();
        execute(appid);

        let opened = OPENED_PATHS.lock().unwrap();
        assert!(opened.is_empty());

        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        }
    }
}
