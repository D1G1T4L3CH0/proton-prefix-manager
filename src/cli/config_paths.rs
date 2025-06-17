use crate::utils::user_config;

#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(not(test))]
fn emit_paths(paths: Vec<std::path::PathBuf>) {
    if paths.is_empty() {
        println!("No localconfig.vdf files found");
    } else {
        for p in paths {
            println!("{}", p.display());
        }
    }
}

#[cfg(test)]
pub static EMITTED_PATHS: Lazy<Mutex<Vec<Vec<std::path::PathBuf>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
#[cfg(test)]
fn emit_paths(paths: Vec<std::path::PathBuf>) {
    EMITTED_PATHS.lock().unwrap().push(paths);
}

pub fn execute() {
    let paths = user_config::get_localconfig_paths();
    emit_paths(paths);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{setup_steam_env, TEST_MUTEX};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_execute_emits_found_paths() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let (home, _prefix, login_opt) = setup_steam_env(0, true);
        let login = login_opt.unwrap();
        let userdata = home.path().join(".steam/steam/userdata");
        fs::create_dir_all(&userdata).unwrap();
        let u1 = userdata.join("111111111");
        let u2 = userdata.join("222222222");
        fs::create_dir_all(u1.join("config")).unwrap();
        fs::create_dir_all(u2.join("config")).unwrap();
        let cfg1 = u1.join("config/localconfig.vdf");
        let cfg2 = u2.join("config/localconfig.vdf");
        fs::write(&cfg1, "").unwrap();
        fs::write(&cfg2, "").unwrap();
        let contents = r#""users" {
            "111111111" { "MostRecent" "0" }
            "222222222" { "MostRecent" "1" }
        }"#;
        fs::write(&login, contents).unwrap();
        let expected = cfg2.clone();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());

        EMITTED_PATHS.lock().unwrap().clear();
        execute();

        let emitted = EMITTED_PATHS.lock().unwrap();
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0], vec![expected]);

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }

    #[test]
    fn test_execute_no_files_found() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::core::steam::clear_caches();
        let dir = tempdir().unwrap();
        let home = dir.path();
        fs::create_dir_all(home.join(".steam/steam/userdata")).unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home);

        EMITTED_PATHS.lock().unwrap().clear();
        execute();

        let emitted = EMITTED_PATHS.lock().unwrap();
        assert_eq!(emitted.len(), 1);
        assert!(emitted[0].is_empty());

        if let Some(h) = old_home { std::env::set_var("HOME", h); }
    }
}
