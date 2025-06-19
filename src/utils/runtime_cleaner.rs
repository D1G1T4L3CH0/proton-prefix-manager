use crate::core::steam;
use crate::utils::library::parse_appmanifest_installdir;
use crate::utils::steam_paths;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct RuntimeItem {
    pub path: PathBuf,
    pub app_id: Option<u32>,
    pub reason: String,
    pub selected: bool,
    pub verified: bool,
}

#[derive(Default)]
pub struct ScanResults {
    pub install_folders: Vec<RuntimeItem>,
    pub prefixes: Vec<RuntimeItem>,
    pub shader_caches: Vec<RuntimeItem>,
    pub tools: Vec<RuntimeItem>,
}

fn is_valid_tool(dir: &Path) -> bool {
    dir.join("proton").exists() || dir.join("proton.sh").exists()
}

pub fn scan() -> ScanResults {
    let mut results = ScanResults::default();
    if let Ok(libraries) = steam::get_steam_libraries() {
        let mut appids = HashSet::new();
        let mut installdirs = HashSet::new();
        for lib in &libraries {
            let steamapps = lib.steamapps_path();
            if let Ok(entries) = fs::read_dir(&steamapps) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.extension().and_then(|s| s.to_str()) == Some("acf") {
                        if let Some((appid, dir)) = parse_appmanifest_installdir(&p) {
                            appids.insert(appid);
                            installdirs.insert(dir);
                        }
                    }
                }
            }
        }
        // Orphaned install folders
        for lib in &libraries {
            let common = lib.steamapps_path().join("common");
            if let Ok(entries) = fs::read_dir(&common) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                            if !installdirs.contains(name) {
                                results.install_folders.push(RuntimeItem {
                                    path: p,
                                    app_id: None,
                                    reason: "No matching appmanifest".to_string(),
                                    selected: true,
                                    verified: true,
                                });
                            }
                        }
                    }
                }
            }
        }
        // Orphaned prefixes
        for lib in &libraries {
            let compat = lib.compatdata_path();
            if let Ok(entries) = fs::read_dir(&compat) {
                for e in entries.flatten() {
                    if let Ok(app) = e.file_name().to_string_lossy().parse::<u32>() {
                        if !appids.contains(&app) {
                            results.prefixes.push(RuntimeItem {
                                path: e.path(),
                                app_id: Some(app),
                                reason: format!("No appmanifest found for AppID {}", app),
                                selected: true,
                                verified: true,
                            });
                        }
                    }
                }
            }
        }
        // Unused shader cache
        for lib in &libraries {
            let shader = lib.steamapps_path().join("shadercache");
            if let Ok(entries) = fs::read_dir(&shader) {
                for e in entries.flatten() {
                    if let Ok(app) = e.file_name().to_string_lossy().parse::<u32>() {
                        if !appids.contains(&app) {
                            results.shader_caches.push(RuntimeItem {
                                path: e.path(),
                                app_id: Some(app),
                                reason: format!("No appmanifest found for AppID {}", app),
                                selected: true,
                                verified: true,
                            });
                        }
                    }
                }
            }
        }
    }

    // custom Proton tools
    for dir in steam_paths::compatibilitytools_dirs() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for e in entries.flatten() {
                if e.path().is_dir() && !is_valid_tool(&e.path()) {
                    results.tools.push(RuntimeItem {
                        path: e.path(),
                        app_id: None,
                        reason: "Missing proton executable".to_string(),
                        selected: false,
                        verified: false,
                    });
                }
            }
        }
    }

    results
}

pub fn delete_item(item: &RuntimeItem) -> std::io::Result<()> {
    fs::remove_dir_all(&item.path)
}
