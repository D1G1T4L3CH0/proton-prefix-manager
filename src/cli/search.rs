use crate::core::steam;
use crate::utils::output;
use crate::utils::output::OutputFormat;
use crate::core::models::GameInfo;

#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(not(test))]
fn emit_search_results(results: Vec<GameInfo>, format: &OutputFormat) {
    output::print_search_results(results, format);
}

#[cfg(test)]
pub static SEARCH_RESULTS: Lazy<Mutex<Vec<Vec<GameInfo>>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(test)]
fn emit_search_results(results: Vec<GameInfo>, _format: &OutputFormat) {
    SEARCH_RESULTS.lock().unwrap().push(results);
}


pub fn execute(name: &str, format: &OutputFormat) {
    if matches!(format, OutputFormat::Normal) {
        println!("🔎 Searching for '{}'", name);
    }

    match steam::search_games(name) {
        Ok(results) => {
            emit_search_results(results, format);
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

    fn setup_mock_steam(appid: u32, name: &str) -> tempfile::TempDir {
        let home = tempdir().unwrap();
        let config_dir = home.path().join(".steam/steam/config");
        fs::create_dir_all(&config_dir).unwrap();

        let library_dir = home.path().join("library");
        let steamapps = library_dir.join("steamapps");
        fs::create_dir_all(&steamapps).unwrap();
        let compat_path = library_dir.join("steamapps/compatdata").join(appid.to_string());
        fs::create_dir_all(&compat_path).unwrap();

        let manifest = steamapps.join(format!("appmanifest_{}.acf", appid));
        let manifest_content = format!(
            "\"AppState\" {{\n    \"appid\" \"{}\"\n    \"name\" \"{}\"\n}}",
            appid, name
        );
        fs::write(&manifest, manifest_content).unwrap();

        let vdf_path = config_dir.join("libraryfolders.vdf");
        let content = format!(
            "\"libraryfolders\" {{\n    \"0\" {{\n        \"path\" \"{}\"\n    }}\n}}",
            library_dir.display()
        );
        fs::write(&vdf_path, content).unwrap();

        home
    }

    #[test]
    fn test_search_finds_game() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 7777;
        let name = "Test Game";
        let home = setup_mock_steam(appid, name);
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        SEARCH_RESULTS.lock().unwrap().clear();
        execute("test", &OutputFormat::Plain);

        let results = SEARCH_RESULTS.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].len(), 1);
        assert_eq!(results[0][0].app_id(), appid);
        assert_eq!(results[0][0].name(), name);

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }

    #[test]
    fn test_search_no_results() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let home = setup_mock_steam(8888, "Another Game");
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        SEARCH_RESULTS.lock().unwrap().clear();
        execute("nomatch", &OutputFormat::Plain);

        let results = SEARCH_RESULTS.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_empty());

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }
}
