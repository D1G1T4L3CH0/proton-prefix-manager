use crate::core::steam;
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
pub static PREFIX_RESULTS: Lazy<Mutex<Vec<(u32, Option<std::path::PathBuf>)>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(test)]
fn emit_prefix_result(appid: u32, prefix: Option<std::path::PathBuf>, _format: &OutputFormat) {
    PREFIX_RESULTS.lock().unwrap().push((appid, prefix));
}

pub fn execute(appid: u32, format: &OutputFormat) {
    if matches!(format, OutputFormat::Normal) {
        println!("🔍 Locating Proton prefix for AppID: {}", appid);
    }
    
    match steam::get_steam_libraries() {
        Ok(libraries) => {
            let prefix = steam::find_proton_prefix(appid, &libraries);
            emit_prefix_result(appid, prefix, format);
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
    use tempfile::tempdir;
    use crate::test_helpers::TEST_MUTEX;

    fn setup_mock_steam(appid: u32) -> (tempfile::TempDir, std::path::PathBuf) {
        let home = tempdir().unwrap();
        let config_dir = home.path().join(".steam/steam/config");
        fs::create_dir_all(&config_dir).unwrap();

        let library_dir = home.path().join("library");
        let compat_path = library_dir.join("steamapps/compatdata").join(appid.to_string());
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
    fn test_execute_prefix_found() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 4242;
        let (home, prefix) = setup_mock_steam(appid);
        let old_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", home.path()); }

        PREFIX_RESULTS.lock().unwrap().clear();
        execute(appid, &OutputFormat::Plain);

        let results = PREFIX_RESULTS.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, appid);
        assert_eq!(results[0].1.as_ref().unwrap(), &prefix);

        if let Some(h) = old_home { unsafe { std::env::set_var("HOME", h); } }
    }

    #[test]
    fn test_execute_prefix_missing() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 1337;
        let (home, prefix) = setup_mock_steam(appid);
        fs::remove_dir_all(&prefix).unwrap();
        let old_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", home.path()); }

        PREFIX_RESULTS.lock().unwrap().clear();
        execute(appid, &OutputFormat::Plain);

        let results = PREFIX_RESULTS.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_none());

        if let Some(h) = old_home { unsafe { std::env::set_var("HOME", h); } }
    }
}
