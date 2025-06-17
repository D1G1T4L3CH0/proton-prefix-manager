use crate::utils::steam_paths;
use keyvalues_parser::{Value, Vdf};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Search Steam userdata directories for localconfig.vdf files.

fn most_recent_user_id() -> Option<String> {
    for dir in steam_paths::config_dirs() {
        let p = dir.join("loginusers.vdf");
        if !p.exists() {
            continue;
        }
        if let Ok(contents) = fs::read_to_string(&p) {
            if let Ok(vdf) = Vdf::parse(&contents) {
                let users_obj_opt = if vdf.key == "users" {
                    vdf.value.get_obj()
                } else {
                    vdf.value
                        .get_obj()
                        .and_then(|o| o.get("users"))
                        .and_then(|v| v.first())
                        .and_then(Value::get_obj)
                };
                if let Some(users_obj) = users_obj_opt {
                    for (uid, vals) in users_obj.iter() {
                        if let Some(user_obj) = vals.first().and_then(Value::get_obj) {
                            if let Some(most) = user_obj
                                .get("MostRecent")
                                .and_then(|v| v.first())
                                .and_then(Value::get_str)
                            {
                                if most == "1" {
                                    return Some(uid.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn find_localconfig_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let recent = most_recent_user_id();
    for dir in steam_paths::userdata_dirs() {
        if let Some(uid) = &recent {
            let cfg = dir.join(uid).join("config/localconfig.vdf");
            log::debug!("checking candidate path: {:?}", cfg);
            if cfg.exists() {
                files.push(cfg);
            }
        } else if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let cfg = entry.path().join("config/localconfig.vdf");
                log::debug!("checking candidate path: {:?}", cfg);
                if cfg.exists() {
                    files.push(cfg);
                }
            }
        }
    }
    files
}

/// Path to `localconfig.vdf` for the active Steam user, even if the file doesn't exist.
fn default_localconfig_path() -> Option<PathBuf> {
    let uid = most_recent_user_id()?;
    for dir in steam_paths::userdata_dirs() {
        let user_dir = dir.join(&uid);
        if user_dir.exists() {
            return Some(user_dir.join("config/localconfig.vdf"));
        }
    }
    None
}

/// Return all discovered `localconfig.vdf` files for the current user.
pub fn get_localconfig_paths() -> Vec<PathBuf> {
    find_localconfig_files()
}

/// Get the expected location of `localconfig.vdf` for the active user.
pub fn expected_localconfig_path() -> Option<PathBuf> {
    default_localconfig_path()
}

fn parse_launch_options(contents: &str, app_id: u32) -> Option<String> {
    let vdf = Vdf::parse(contents).ok()?;
    let root = vdf.value.get_obj()?;
    let apps = root
        .get("UserLocalConfigStore")?
        .first()?
        .get_obj()?
        .get("Software")?
        .first()?
        .get_obj()?
        .get("Valve")?
        .first()?
        .get_obj()?
        .get("Steam")?
        .first()?
        .get_obj()?
        .get("apps")?
        .first()?
        .get_obj()?;
    apps.get(app_id.to_string().as_str())?
        .first()?
        .get_obj()?
        .get("LaunchOptions")?
        .first()?
        .get_str()
        .map(|s| s.to_string())
}

pub fn get_launch_options(app_id: u32) -> Option<String> {
    for cfg in find_localconfig_files() {
        match fs::read_to_string(&cfg) {
            Ok(contents) => {
                log::debug!("read localconfig {:?} successfully", cfg);
                if let Some(val) = parse_launch_options(&contents, app_id) {
                    return Some(val);
                }
            }
            Err(e) => {
                log::debug!("failed to read {:?}: {}", cfg, e);
            }
        }
    }
    None
}

fn update_launch_options(contents: &str, app_id: u32, value: &str) -> Option<String> {
    // Parse the existing VDF or create a new one if parsing fails
    let mut vdf = Vdf::parse(contents).unwrap_or_else(|_| {
        Vdf::new(
            "UserLocalConfigStore".into(),
            Value::Obj(Default::default()),
        )
    });

    // Ensure we have a root object to work with
    if vdf.value.get_mut_obj().is_none() {
        vdf.value = Value::Obj(Default::default());
    }
    let root = vdf.value.get_mut_obj().unwrap();

    // Walk or create the nested hierarchy down to the apps object
    let ulcs = root
        .entry("UserLocalConfigStore".into())
        .or_insert_with(|| vec![Value::Obj(Default::default())]);
    let mut obj = ulcs.first_mut().and_then(Value::get_mut_obj).unwrap();

    for key in ["Software", "Valve", "Steam", "apps"] {
        obj = obj
            .entry(key.into())
            .or_insert_with(|| vec![Value::Obj(Default::default())])
            .first_mut()
            .and_then(Value::get_mut_obj)
            .unwrap();
    }

    let entry = obj
        .entry(app_id.to_string().into())
        .or_insert_with(|| vec![Value::Obj(Default::default())]);
    let app_obj = entry.first_mut().and_then(Value::get_mut_obj).unwrap();

    match app_obj.get_mut("LaunchOptions") {
        Some(vals) if !vals.is_empty() => {
            if let Some(s) = vals.first_mut().and_then(Value::get_mut_str) {
                *s.to_mut() = value.to_string();
            }
        }
        _ => {
            app_obj.insert("LaunchOptions".into(), vec![Value::Str(value.into())]);
        }
    }

    Some(format!("{}", vdf))
}

pub fn set_launch_options(app_id: u32, value: &str) -> io::Result<()> {
    let mut found = false;
    for cfg in find_localconfig_files() {
        found = true;
        match fs::read_to_string(&cfg) {
            Ok(contents) => {
                log::debug!("read localconfig {:?} successfully", cfg);
                if let Some(updated) = update_launch_options(&contents, app_id, value) {
                    match fs::write(&cfg, updated) {
                        Ok(_) => {
                            log::debug!("wrote launch options to {:?}", cfg);
                            return Ok(());
                        }
                        Err(e) => {
                            log::debug!("failed to write {:?}: {}", cfg, e);
                            return Err(e);
                        }
                    }
                }
            }
            Err(e) => {
                log::debug!("failed to read {:?}: {}", cfg, e);
            }
        }
    }
    if let Some(cfg) = default_localconfig_path() {
        fs::create_dir_all(cfg.parent().unwrap())?;
        if let Some(updated) = update_launch_options("", app_id, value) {
            fs::write(&cfg, updated)?;
            log::debug!("created {:?} with launch options", cfg);
            return Ok(());
        }
    }
    if found {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to update localconfig",
        ))
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "localconfig not found",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::TEST_MUTEX;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs as unix_fs;
    #[cfg(windows)]
    use std::os::windows::fs as windows_fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_localconfig_files_respects_loginusers() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let dir = tempdir().unwrap();
        let home = dir.path();

        let userdata = home.join(".steam/steam/userdata");
        fs::create_dir_all(&userdata).unwrap();
        let u1 = userdata.join("111111111");
        let u2 = userdata.join("222222222");
        fs::create_dir_all(u1.join("config")).unwrap();
        fs::create_dir_all(u2.join("config")).unwrap();
        let cfg1 = u1.join("config/localconfig.vdf");
        let cfg2 = u2.join("config/localconfig.vdf");
        fs::write(&cfg1, "").unwrap();
        fs::write(&cfg2, "").unwrap();

        let config_dir = home.join(".steam/config");
        fs::create_dir_all(&config_dir).unwrap();
        let login = config_dir.join("loginusers.vdf");
        let contents = r#""users" {
            "111111111" { "MostRecent" "0" }
            "222222222" { "MostRecent" "1" }
        }"#;
        fs::write(&login, contents).unwrap();

        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home);

        let files = find_localconfig_files();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], cfg2);

        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        }
    }

    #[test]
    fn test_userdata_dirs_deduplicates_symlink() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let dir = tempdir().unwrap();
        let home = dir.path();

        let p1 = home.join(".steam/steam/userdata");
        fs::create_dir_all(&p1).unwrap();

        let p2 = home.join(".local/share/Steam/userdata");
        fs::create_dir_all(p2.parent().unwrap()).unwrap();

        #[cfg(unix)]
        unix_fs::symlink(&p1, &p2).unwrap();
        #[cfg(windows)]
        windows_fs::symlink_dir(&p1, &p2).unwrap();

        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home);

        let dirs = steam_paths::userdata_dirs();
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0], fs::canonicalize(&p1).unwrap());

        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        }
    }

    #[test]
    fn test_userdata_dirs_checks_steam_root() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let dir = tempdir().unwrap();
        let home = dir.path();

        let p = home.join(".steam/root/userdata");
        fs::create_dir_all(&p).unwrap();

        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home);

        let dirs = steam_paths::userdata_dirs();
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0], fs::canonicalize(&p).unwrap());

        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        }
    }

    #[test]
    fn test_update_launch_options_creates_section() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let contents = "";
        let updated = update_launch_options(contents, 123, "-novid").unwrap();
        assert_eq!(
            parse_launch_options(&updated, 123),
            Some("-novid".to_string())
        );
    }

    #[test]
    fn test_set_launch_options_missing_file() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (home, _prefix, _login) = crate::test_helpers::setup_steam_env(123456, true);
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());
        fs::create_dir_all(home.path().join(".steam/steam/userdata/111111111/config")).unwrap();

        let result = set_launch_options(123456, "-novid");
        assert!(result.is_ok());
        let cfg_path = home
            .path()
            .join(".steam/steam/userdata/111111111/config/localconfig.vdf");
        assert!(cfg_path.exists());

        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        }
    }
}
