use crate::core::steam;
use crate::utils::dependencies::command_available;

#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(not(test))]
fn run_winecfg(prefix_path: &std::path::Path) -> std::io::Result<()> {
    let status = std::process::Command::new("winecfg")
        .env("WINEPREFIX", prefix_path)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("winecfg exited with status {}", status),
        ))
    }
}

#[cfg(test)]
pub static WINECFG_CALLS: Lazy<Mutex<Vec<std::path::PathBuf>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(test)]
fn run_winecfg(prefix_path: &std::path::Path) -> std::io::Result<()> {
    WINECFG_CALLS
        .lock()
        .unwrap()
        .push(prefix_path.to_path_buf());
    Ok(())
}

pub fn execute(appid: u32) {
    log::debug!("winecfg command: appid={}", appid);
    println!("🍷 Launching winecfg for AppID: {}", appid);

    if !command_available("winecfg") {
        eprintln!("❌ 'winecfg' is not installed or not found in PATH. Please install it to use this feature.");
        return;
    }

    match steam::get_steam_libraries() {
        Ok(libraries) => {
            if let Some(prefix_path) = steam::find_proton_prefix(appid, &libraries) {
                if let Err(e) = run_winecfg(&prefix_path) {
                    eprintln!("❌ Failed to launch winecfg: {}", e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{setup_steam_env, TEST_MUTEX};
    use std::fs;

    #[test]
    fn test_execute_runs_winecfg() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 4321;
        let (home, prefix, _) = setup_steam_env(appid, false);
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        WINECFG_CALLS.lock().unwrap().clear();
        execute(appid);

        let calls = WINECFG_CALLS.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], prefix);

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }

    #[test]
    fn test_execute_no_prefix() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 8765;
        let (home, prefix, _) = setup_steam_env(appid, false);
        fs::remove_dir_all(&prefix).unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        WINECFG_CALLS.lock().unwrap().clear();
        execute(appid);

        let calls = WINECFG_CALLS.lock().unwrap();
        assert!(calls.is_empty());

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }
}
