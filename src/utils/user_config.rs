use regex::Regex;
use std::{fs, io, path::{PathBuf}};
use dirs_next;

/// Search Steam userdata directories for localconfig.vdf files.
fn userdata_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs_next::home_dir() {
        let p1 = home.join(".steam/steam/userdata");
        if p1.exists() { dirs.push(p1); }
        let p2 = home.join(".local/share/Steam/userdata");
        if p2.exists() { dirs.push(p2); }
    }
    dirs
}

fn find_localconfig_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in userdata_dirs() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let cfg = entry.path().join("config/localconfig.vdf");
                if cfg.exists() { files.push(cfg); }
            }
        }
    }
    files
}

fn parse_launch_options(contents: &str, app_id: u32) -> Option<String> {
    let pattern = format!(r#"(?s)\"{}\"\s*\{{[^}}]*\"LaunchOptions\"\s+\"([^\"]*)\""#, app_id);
    let re = Regex::new(&pattern).ok()?;
    re.captures(contents)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
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
    let pattern = format!(r#"(?s)(\"{}\"\s*\{{)([^}}]*)(\}})"#, app_id);
    let re = Regex::new(&pattern).ok()?;
    if let Some(cap) = re.captures(contents) {
        let start = cap.get(1)?.as_str();
        let body = cap.get(2)?.as_str();
        let end = cap.get(3)?.as_str();
        let updated_body = crate::utils::manifest::update_or_insert(body, "LaunchOptions", value);
        let mut new_section = String::new();
        new_section.push_str(start);
        new_section.push_str(&updated_body);
        new_section.push_str(end);
        Some(re.replace(contents, new_section).into_owned())
    } else {
        None
    }
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
