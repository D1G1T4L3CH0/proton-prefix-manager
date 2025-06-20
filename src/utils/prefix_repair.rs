use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::steam;
use crate::error::{Error, Result};
use crate::utils::steam_paths;

fn detect_proton_version(prefix_path: &Path) -> Option<String> {
    let version_file = prefix_path.join("version");
    log::debug!("looking for version in {:?}", version_file);
    if version_file.exists() {
        if let Ok(contents) = fs::read_to_string(&version_file) {
            let version = contents.trim().to_string();
            if !version.is_empty() {
                log::debug!("found version '{}' in {:?}", version, version_file);
                return Some(version);
            }
        }
    }

    if let Some(parent) = prefix_path.parent() {
        let version_file = parent.join("version");
        log::debug!("looking for version in parent {:?}", version_file);
        if version_file.exists() {
            if let Ok(contents) = fs::read_to_string(&version_file) {
                let version = contents.trim().to_string();
                if !version.is_empty() {
                    log::debug!("found version '{}' in {:?}", version, version_file);
                    return Some(version);
                }
            }
        }
    }
    None
}

fn find_proton_runtime(version: &str) -> Option<PathBuf> {
    let mut candidates = vec![version.to_string()];
    let normalized = version.trim();
    if !normalized.to_lowercase().starts_with("proton") {
        let base = normalized
            .split(|c| c == '-' || c == ' ')
            .next()
            .unwrap_or(normalized);
        candidates.push(format!("Proton {}", base));
    } else {
        let rest = normalized
            .trim_start_matches("Proton")
            .trim()
            .split(|c| c == '-' || c == ' ')
            .next()
            .unwrap_or("");
        if !rest.is_empty() {
            candidates.push(format!("Proton {}", rest));
        }
    }

    if let Ok(libs) = steam::get_steam_libraries() {
        for cand in &candidates {
            for lib in &libs {
                let path = lib.join("steamapps/common").join(cand);
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    for dir in steam_paths::compatibilitytools_dirs() {
        for cand in &candidates {
            let path = dir.join(cand);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

fn find_wineboot(runtime: &Path) -> Option<PathBuf> {
    let candidates = [
        runtime.join("dist/bin/wineboot"),
        runtime.join("files/bin/wineboot"),
        runtime.join("bin/wineboot"),
    ];
    for c in candidates.iter() {
        if c.exists() {
            return Some(c.clone());
        }
    }
    None
}

/// Attempt to repair a Proton prefix.
///
/// This will recreate critical folders and run `wineboot` to
/// regenerate missing registry files.
pub fn repair_prefix(prefix: &Path) -> Result<()> {
    log::debug!("repairing prefix at {:?}", prefix);
    let pfx = prefix.join("pfx");
    if !pfx.exists() {
        log::debug!("creating {:?}", pfx);
        fs::create_dir_all(&pfx)?;
    }
    // Ensure core directories exist
    let drive_c = pfx.join("drive_c");
    let dosdevices = pfx.join("dosdevices");
    log::debug!("ensuring {:?} exists", drive_c);
    fs::create_dir_all(&drive_c)?;
    log::debug!("ensuring {:?} exists", dosdevices);
    fs::create_dir_all(&dosdevices)?;
    let _ = fs::File::create(pfx.join(".update-timestamp"));

    // Run wineboot to regenerate registry files
    if let Some(version) = detect_proton_version(prefix) {
        log::debug!("detected proton version: {}", version);
        if let Some(runtime) = find_proton_runtime(&version) {
            log::debug!("found proton runtime at {:?}", runtime);
            if let Some(wb) = find_wineboot(&runtime) {
                log::debug!("using wineboot at {:?}", wb);
                let status = Command::new(wb)
                    .env("WINEPREFIX", &pfx)
                    .status()
                    .map_err(Error::from)?;
                if !status.success() {
                    return Err(Error::FileSystemError("wineboot failed".into()));
                }
                return Ok(());
            } else {
                log::debug!("wineboot not found in runtime {:?}", runtime);
            }
        } else {
            log::debug!("runtime for version {} not found", version);
        }
    } else {
        log::debug!("proton version could not be detected");
    }

    log::debug!("falling back to system wineboot");
    let status = Command::new("wineboot")
        .env("WINEPREFIX", &pfx)
        .status()
        .map_err(Error::from)?;
    if !status.success() {
        return Err(Error::FileSystemError("wineboot failed".into()));
    }
    Ok(())
}
