use dirs_next;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Return possible base directories for Steam installations.
///
/// Checks common locations under the user's home directory and
/// returns any that exist, deduplicated using canonical paths.
pub fn steam_base_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut seen = HashSet::new();

    if let Some(home) = dirs_next::home_dir() {
        let candidates = [
            home.join(".steam/steam"),
            home.join(".local/share/Steam"),
            home.join(".steam/root"),
            home.join(".steam/debian-installation"),
            home.join(".steam"),
        ];

        for cand in candidates.iter() {
            if cand.exists() {
                let canon = fs::canonicalize(cand).unwrap_or_else(|_| cand.clone());
                if seen.insert(canon.clone()) {
                    dirs.push(canon);
                }
            }
        }
    }

    dirs
}

/// Generate userdata directories for all detected Steam bases.
pub fn userdata_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut seen = HashSet::new();
    for base in steam_base_dirs() {
        let p = base.join("userdata");
        if p.exists() {
            let canon = fs::canonicalize(&p).unwrap_or(p.clone());
            if seen.insert(canon.clone()) {
                dirs.push(canon);
            }
        }
    }
    dirs
}

/// Generate config directory paths for all detected Steam bases.
pub fn config_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut seen = HashSet::new();

    if let Some(home) = dirs_next::home_dir() {
        let candidates = [
            home.join(".steam/steam/config"),
            home.join(".local/share/Steam/config"),
            home.join(".steam/config"),
            home.join(".steam/root/config"),
            home.join(".steam/debian-installation/config"),
        ];

        for cand in candidates.iter() {
            if cand.exists() {
                let canon = fs::canonicalize(cand).unwrap_or_else(|_| cand.clone());
                if seen.insert(canon.clone()) {
                    dirs.push(canon);
                }
            }
        }
    }

    dirs
}
