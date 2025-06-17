use crate::core::steam;
use crate::utils::dependencies::command_available;

#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(not(test))]
fn run_protontricks(appid: u32, args: &[String]) -> std::io::Result<()> {
    let status = std::process::Command::new("protontricks")
        .arg(appid.to_string())
        .args(args)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("protontricks exited with status {}", status),
        ))
    }
}

#[cfg(test)]
pub static PROTONTRICKS_CALLS: Lazy<Mutex<Vec<(u32, Vec<String>)>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(test)]
fn run_protontricks(appid: u32, args: &[String]) -> std::io::Result<()> {
    PROTONTRICKS_CALLS
        .lock()
        .unwrap()
        .push((appid, args.to_vec()));
    Ok(())
}

pub fn execute(appid: u32, args: &[String]) {
    log::debug!("protontricks command: appid={} args={:?}", appid, args);
    println!("🔧 Running protontricks for AppID: {}", appid);

    if !command_available("protontricks") {
        eprintln!("❌ 'protontricks' is not installed or not found in PATH. Please install it to use this feature.");
        return;
    }

    match steam::get_steam_libraries() {
        Ok(libraries) => {
            if steam::find_proton_prefix(appid, &libraries).is_some() {
                if let Err(e) = run_protontricks(appid, args) {
                    eprintln!("❌ Failed to run protontricks: {}", e);
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
    fn test_execute_runs_protontricks() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 1234;
        let (home, _prefix, _) = setup_steam_env(appid, false);
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        PROTONTRICKS_CALLS.lock().unwrap().clear();
        execute(appid, &["-v".to_string()]);

        let calls = PROTONTRICKS_CALLS.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, appid);
        assert_eq!(calls[0].1, vec!["-v".to_string()]);

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }

    #[test]
    fn test_execute_no_prefix() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 5678;
        let (home, prefix, _) = setup_steam_env(appid, false);
        fs::remove_dir_all(&prefix).unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        PROTONTRICKS_CALLS.lock().unwrap().clear();
        execute(appid, &[]);

        let calls = PROTONTRICKS_CALLS.lock().unwrap();
        assert!(calls.is_empty());

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }
}
