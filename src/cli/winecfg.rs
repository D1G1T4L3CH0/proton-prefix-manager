use crate::core::steam;

#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(not(test))]
fn run_winecfg(prefix_path: &std::path::Path) -> std::io::Result<()> {
    std::process::Command::new("winecfg")
        .env("WINEPREFIX", prefix_path)
        .spawn()
        .map(|_| ())
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
    println!("🍷 Launching winecfg for AppID: {}", appid);

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
    use crate::test_helpers::TEST_MUTEX;
    use std::fs;
    use tempfile::tempdir;

    fn setup_mock_steam(appid: u32) -> (tempfile::TempDir, std::path::PathBuf) {
        let home = tempdir().unwrap();
        let config_dir = home.path().join(".steam/steam/config");
        fs::create_dir_all(&config_dir).unwrap();

        let library_dir = home.path().join("library");
        let compat_path = library_dir
            .join("steamapps/compatdata")
            .join(appid.to_string());
        fs::create_dir_all(&compat_path).unwrap();

        let vdf_path = config_dir.join("libraryfolders.vdf");
        let content = format!(
            "\"libraryfolders\" {{\n    \"0\" {{\n        \"path\" \"{}\"\n    }}\n}}",
            library_dir.display()
        );
        fs::write(&vdf_path, content).unwrap();

        (home, compat_path)
    }

    #[test]
    fn test_execute_runs_winecfg() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 4321;
        let (home, prefix) = setup_mock_steam(appid);
        let old_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", home.path()); }

        WINECFG_CALLS.lock().unwrap().clear();
        execute(appid);

        let calls = WINECFG_CALLS.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], prefix);

        if let Some(h) = old_home { unsafe { std::env::set_var("HOME", h); } }
    }

    #[test]
    fn test_execute_no_prefix() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 8765;
        let (home, prefix) = setup_mock_steam(appid);
        fs::remove_dir_all(&prefix).unwrap();
        let old_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", home.path()); }

        WINECFG_CALLS.lock().unwrap().clear();
        execute(appid);

        let calls = WINECFG_CALLS.lock().unwrap();
        assert!(calls.is_empty());

        if let Some(h) = old_home { unsafe { std::env::set_var("HOME", h); } }
    }
}
