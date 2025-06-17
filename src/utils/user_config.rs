use keyvalues_parser::{Vdf, Value};
use std::fs;
use std::io;
use std::path::PathBuf;
use dirs_next;

/// Search Steam userdata directories for localconfig.vdf files.
fn userdata_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs_next::home_dir() {
        let paths = [
            home.join(".steam/steam/userdata"),
            home.join(".local/share/Steam/userdata"),
        ];

        for p in paths.iter() {
            if p.exists() {
                if let Ok(canon) = fs::canonicalize(p) {
                    if !dirs.contains(&canon) {
                        dirs.push(canon);
                    }
                } else if !dirs.contains(p) {
                    dirs.push(p.clone());
                }
            }
        }
    }
    dirs
}

fn most_recent_user_id() -> Option<String> {
    if let Some(home) = dirs_next::home_dir() {
        let paths = [
            home.join(".steam/steam/config/loginusers.vdf"),
            home.join(".local/share/Steam/config/loginusers.vdf"),
            home.join(".steam/config/loginusers.vdf"),
            home.join(".steam/root/config/loginusers.vdf"),
        ];
        for p in paths.iter() {
            if p.exists() {
                if let Ok(contents) = fs::read_to_string(p) {
                    if let Ok(vdf) = Vdf::parse(&contents) {
                        let users_obj_opt = if vdf.key == "users" {
                            vdf.value.get_obj()
                        } else {
                            vdf
                                .value
                                .get_obj()
                                .and_then(|o| o.get("users"))
                                .and_then(|v| v.first())
                                .and_then(Value::get_obj)
                        };
                        if let Some(users_obj) = users_obj_opt {
                            for (uid, vals) in users_obj.iter() {
                                if let Some(user_obj) = vals.first().and_then(Value::get_obj) {
                                    if let Some(most) = user_obj.get("MostRecent").and_then(|v| v.first()).and_then(Value::get_str) {
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
        }
    }
    None
}

fn find_localconfig_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let recent = most_recent_user_id();
    for dir in userdata_dirs() {
        if let Some(uid) = &recent {
            let cfg = dir.join(uid).join("config/localconfig.vdf");
            if cfg.exists() {
                files.push(cfg);
            }
        } else if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let cfg = entry.path().join("config/localconfig.vdf");
                if cfg.exists() {
                    files.push(cfg);
                }
            }
        }
    }
    files
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
    apps
        .get(app_id.to_string().as_str())?
        .first()?
        .get_obj()?
        .get("LaunchOptions")?
        .first()?
        .get_str()
        .map(|s| s.to_string())
}

pub fn get_launch_options(app_id: u32) -> Option<String> {
    for cfg in find_localconfig_files() {
        if let Ok(contents) = fs::read_to_string(&cfg) {
            if let Some(val) = parse_launch_options(&contents, app_id) {
                return Some(val);
            }
        }
    }
    None
}

fn update_launch_options(contents: &str, app_id: u32, value: &str) -> Option<String> {
    let mut vdf = Vdf::parse(contents).ok()?;
    let root = vdf.value.get_mut_obj()?;
    let apps = root
        .get_mut("UserLocalConfigStore")?
        .first_mut()?
        .get_mut_obj()?
        .get_mut("Software")?
        .first_mut()?
        .get_mut_obj()?
        .get_mut("Valve")?
        .first_mut()?
        .get_mut_obj()?
        .get_mut("Steam")?
        .first_mut()?
        .get_mut_obj()?
        .get_mut("apps")?
        .first_mut()?
        .get_mut_obj()?;
    let entry = apps
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
    for cfg in find_localconfig_files() {
        if let Ok(contents) = fs::read_to_string(&cfg) {
            if let Some(updated) = update_launch_options(&contents, app_id, value) {
                fs::write(&cfg, updated)?;
                return Ok(());
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "localconfig not found"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use crate::test_helpers::TEST_MUTEX;
    #[cfg(unix)]
    use std::os::unix::fs as unix_fs;
    #[cfg(windows)]
    use std::os::windows::fs as windows_fs;

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
        unsafe { std::env::set_var("HOME", home); }

        let files = find_localconfig_files();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], cfg2);

        if let Some(h) = old_home { unsafe { std::env::set_var("HOME", h); } }
    }

    #[test]
    fn test_userdata_dirs_resolves_symlinks() {
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
        unsafe { std::env::set_var("HOME", home); }

        let dirs = userdata_dirs();
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0], fs::canonicalize(&p1).unwrap());

        if let Some(h) = old_home { unsafe { std::env::set_var("HOME", h); } }
    }
}
