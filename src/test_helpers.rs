#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
pub static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[cfg(test)]
use tempfile::TempDir;
#[cfg(test)]
use std::fs;

/// Create a temporary Steam environment for tests.
///
/// Returns the temporary directory, the compatdata prefix path for the
/// provided `appid` and optionally the path to a created `loginusers.vdf`.
#[cfg(test)]
pub fn setup_steam_env(appid: u32, create_loginusers: bool) -> (TempDir, std::path::PathBuf, Option<std::path::PathBuf>) {
    let home = tempfile::tempdir().unwrap();
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

    let loginusers_path = if create_loginusers {
        let login_path = config_dir.join("loginusers.vdf");
        let contents = r#""users" { "111111111" { "MostRecent" "1" } }"#;
        fs::write(&login_path, contents).unwrap();
        Some(login_path)
    } else {
        None
    };

    (home, compat_path, loginusers_path)
}
