use crate::core::steam;
#[cfg(not(test))]
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
    use crate::test_helpers::{setup_steam_env, TEST_MUTEX};
    use std::fs;

    #[test]
    fn test_search_finds_game() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let appid = 7777;
        let name = "Test Game";
        let (home, _prefix, _) = setup_steam_env(appid, false);
        let steamapps = home.path().join("library/steamapps");
        fs::create_dir_all(&steamapps).unwrap();
        let manifest = steamapps.join(format!("appmanifest_{}.acf", appid));
        let manifest_content = format!(
            "\"AppState\" {{\n    \"appid\" \"{}\"\n    \"name\" \"{}\"\n}}",
            appid, name
        );
        fs::write(&manifest, manifest_content).unwrap();
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
        let (home, _prefix, _) = setup_steam_env(8888, false);
        let steamapps = home.path().join("library/steamapps");
        fs::create_dir_all(&steamapps).unwrap();
        let manifest = steamapps.join("appmanifest_8888.acf");
        fs::write(&manifest, "").unwrap();
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
