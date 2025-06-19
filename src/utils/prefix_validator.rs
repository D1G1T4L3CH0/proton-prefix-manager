use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::core::steam;
use crate::utils::steam_paths;

/// Status of a validation check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    Warning(String),
}

/// Result of a single validation check.
#[derive(Clone, Debug)]
pub struct CheckResult {
    pub label: String,
    pub status: CheckStatus,
}

fn detect_proton_version(prefix_path: &Path) -> Option<String> {
    let version_file = prefix_path.join("version");
    if version_file.exists() {
        if let Ok(contents) = fs::read_to_string(&version_file) {
            let version = contents.trim().to_string();
            if !version.is_empty() {
                return Some(version);
            }
        }
    }

    if let Some(parent) = prefix_path.parent() {
        let version_file = parent.join("version");
        if version_file.exists() {
            if let Ok(contents) = fs::read_to_string(&version_file) {
                let version = contents.trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
    }

    None
}

fn proton_runtime_exists(version: &str) -> bool {
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
                if lib.join("steamapps/common").join(cand).exists() {
                    return true;
                }
            }
        }
    }
    for dir in steam_paths::compatibilitytools_dirs() {
        for cand in &candidates {
            if dir.join(cand).exists() {
                return true;
            }
        }
    }
    false
}

impl CheckResult {
    fn pass(label: &str) -> Self {
        Self {
            label: label.to_string(),
            status: CheckStatus::Pass,
        }
    }

    fn fail(label: &str, msg: impl Into<String>) -> Self {
        Self {
            label: label.to_string(),
            status: CheckStatus::Fail(msg.into()),
        }
    }

    fn warn(label: &str, msg: impl Into<String>) -> Self {
        Self {
            label: label.to_string(),
            status: CheckStatus::Warning(msg.into()),
        }
    }
}

/// Validate a Proton prefix directory. `prefix` should be the compatdata/<appid>
/// directory, which contains a `pfx` subdirectory.
pub fn validate_prefix(prefix: &Path) -> Vec<CheckResult> {
    let mut results = Vec::new();

    // 1. Directory exists and is directory
    if !prefix.exists() {
        results.push(CheckResult::fail("Prefix directory", "Directory not found"));
        return results;
    }
    if !prefix.is_dir() {
        results.push(CheckResult::fail("Prefix directory", "Not a directory"));
        return results;
    }
    results.push(CheckResult::pass("Prefix directory"));

    let pfx = prefix.join("pfx");
    if !pfx.exists() {
        results.push(CheckResult::fail("pfx folder", "Missing pfx directory"));
        return results;
    }
    results.push(CheckResult::pass("pfx folder"));

    if fs::read_dir(&pfx)
        .map(|mut it| it.next().is_none())
        .unwrap_or(true)
    {
        results.push(CheckResult::fail("pfx folder", "Prefix appears empty"));
    }

    // Required subdirectories
    let drive_c = pfx.join("drive_c");
    if drive_c.is_dir() {
        results.push(CheckResult::pass("drive_c"));
    } else {
        results.push(CheckResult::fail("drive_c", "Missing drive_c directory"));
    }

    let dosdevices = pfx.join("dosdevices");
    if dosdevices.is_dir() {
        results.push(CheckResult::pass("dosdevices"));
    } else {
        results.push(CheckResult::fail(
            "dosdevices",
            "Missing dosdevices directory",
        ));
    }

    // Registry files
    for name in ["system.reg", "user.reg", "userdef.reg"] {
        let path = pfx.join(name);
        if path.is_file() {
            match fs::metadata(&path) {
                Ok(meta) => {
                    if meta.len() > 0 {
                        results.push(CheckResult::pass(name));
                    } else {
                        results.push(CheckResult::warn(name, "File is empty"));
                    }
                }
                Err(_) => {
                    results.push(CheckResult::fail(name, "Unreadable"));
                }
            }
        } else {
            results.push(CheckResult::fail(name, "Missing"));
        }
    }

    let winetricks_log = pfx.join("winetricks.log");
    if winetricks_log.is_file() {
        results.push(CheckResult::pass("winetricks.log"));
    } else {
        results.push(CheckResult::warn("winetricks.log", "Missing"));
    }

    // windows directory under drive_c
    let windows_dir = drive_c.join("windows");
    if windows_dir.is_dir() {
        results.push(CheckResult::pass("drive_c/windows"));
    } else {
        results.push(CheckResult::fail(
            "drive_c/windows",
            "Missing windows directory",
        ));
    }

    // Optional heuristics
    let program_files = drive_c.join("Program Files");
    if program_files.exists() {
        results.push(CheckResult::pass("Program Files"));
    } else {
        results.push(CheckResult::warn("Program Files", "Not found"));
    }

    let user_dir = drive_c.join("users/steamuser");
    if user_dir.exists() {
        results.push(CheckResult::pass("users/steamuser"));
    } else {
        results.push(CheckResult::warn("users/steamuser", "Not found"));
    }

    let mut broken_symlinks = 0;
    for entry in WalkDir::new(&pfx) {
        if let Ok(e) = entry {
            if e.file_type().is_symlink() {
                if let Ok(target) = fs::read_link(e.path()) {
                    if !target.is_absolute() {
                        let abs = e.path().parent().unwrap_or(Path::new("")).join(&target);
                        if !abs.exists() {
                            broken_symlinks += 1;
                        }
                    } else if !target.exists() {
                        broken_symlinks += 1;
                    }
                } else {
                    broken_symlinks += 1;
                }
            }
        }
    }
    if broken_symlinks > 0 {
        results.push(CheckResult::warn(
            "Symlinks",
            format!("{} broken links", broken_symlinks),
        ));
    }

    if let Some(ver) = detect_proton_version(prefix) {
        if proton_runtime_exists(&ver) {
            results.push(CheckResult::pass("Proton runtime"));
        } else {
            results.push(CheckResult::fail(
                "Proton runtime",
                format!("Version '{}' missing", ver),
            ));
        }
    } else {
        results.push(CheckResult::warn("Proton runtime", "Unknown version"));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_validate_prefix_missing() {
        let tmp = tempdir().unwrap();
        let missing = tmp.path().join("nope");
        let results = validate_prefix(&missing);
        assert!(matches!(results[0].status, CheckStatus::Fail(_)));
    }

    #[test]
    fn test_validate_prefix_ok() {
        let dir = tempdir().unwrap();
        let pfx = dir.path().join("pfx");
        let drive_c = pfx.join("drive_c/windows");
        let dosdevices = pfx.join("dosdevices");
        fs::create_dir_all(&drive_c).unwrap();
        fs::create_dir_all(&dosdevices).unwrap();
        for name in ["system.reg", "user.reg", "userdef.reg"] {
            let mut f = fs::File::create(pfx.join(name)).unwrap();
            writeln!(f, "test").unwrap();
        }

        let results = validate_prefix(dir.path());
        assert!(results
            .iter()
            .all(|r| !matches!(r.status, CheckStatus::Fail(_))));
    }
}
