use crate::core::steam;
#[cfg(not(test))]
use crate::utils::output;
use crate::utils::output::OutputFormat;

#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(not(test))]
fn emit_prefix_result(appid: u32, prefix: Option<std::path::PathBuf>, format: &OutputFormat) {
    output::print_prefix_result(appid, prefix, format);
}

#[cfg(test)]
pub static PREFIX_RESULTS: Lazy<Mutex<Vec<(u32, Option<std::path::PathBuf>)>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(test)]
fn emit_prefix_result(appid: u32, prefix: Option<std::path::PathBuf>, _format: &OutputFormat) {
    PREFIX_RESULTS.lock().unwrap().push((appid, prefix));
}

pub fn execute(appid: u32, format: &OutputFormat) {
    log::debug!("prefix command: appid={} format={:?}", appid, format);
    if matches!(format, OutputFormat::Normal) {
        println!("🔍 Locating Proton prefix for AppID: {}", appid);
    }

    match steam::get_steam_libraries() {
        Ok(libraries) => {
            let prefix = steam::find_proton_prefix(appid, &libraries);
            emit_prefix_result(appid, prefix, format);
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
    fn test_execute_prefix_found() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 4242;
        let (home, prefix, _) = setup_steam_env(appid, false);
        let old_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", home.path());
        }

        PREFIX_RESULTS.lock().unwrap().clear();
        execute(appid, &OutputFormat::Plain);

        let results = PREFIX_RESULTS.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, appid);
        assert_eq!(results[0].1.as_ref().unwrap(), &prefix);

        if let Some(h) = old_home {
            unsafe {
                std::env::set_var("HOME", h);
            }
        }
    }

    #[test]
    fn test_execute_prefix_missing() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 1337;
        let (home, prefix, _) = setup_steam_env(appid, false);
        fs::remove_dir_all(&prefix).unwrap();
        let old_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", home.path());
        }

        PREFIX_RESULTS.lock().unwrap().clear();
        execute(appid, &OutputFormat::Plain);

        let results = PREFIX_RESULTS.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_none());

        if let Some(h) = old_home {
            unsafe {
                std::env::set_var("HOME", h);
            }
        }
    }
}
