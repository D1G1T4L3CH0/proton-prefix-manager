use std::fs;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::os::unix::fs as unix_fs;

use chrono::Local;
use dirs_next;

use std::collections::BTreeMap;

use crate::core::models::SteamLibrary;
use crate::error::{Error, Result};

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else if file_type.is_symlink() {
            let target = fs::read_link(entry.path())?;
            #[cfg(unix)]
            unix_fs::symlink(&target, &dest_path)?;
            #[cfg(not(unix))]
            fs::copy(target, dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Back up a Proton prefix by copying it to the given destination directory.
pub fn backup_root() -> PathBuf {
    dirs_next::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("proton-prefix-manager")
        .join("backups")
}

pub fn create_backup(prefix_path: &Path, appid: u32) -> Result<PathBuf> {
    if !prefix_path.exists() {
        return Err(Error::FileSystemError(format!(
            "Prefix not found: {}",
            prefix_path.display()
        )));
    }

    let root = backup_root().join(appid.to_string());
    fs::create_dir_all(&root)?;
    let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
    let dest = root.join(timestamp);
    copy_dir_recursive(prefix_path, &dest)?;
    Ok(dest)
}

/// Restore a Proton prefix from a backup directory.
pub fn restore_prefix(backup_path: &Path, prefix_path: &Path) -> Result<PathBuf> {
    if !backup_path.exists() {
        return Err(Error::FileSystemError(format!(
            "Backup not found: {}",
            backup_path.display()
        )));
    }

    if prefix_path.exists() {
        fs::remove_dir_all(prefix_path)?;
    }
    copy_dir_recursive(backup_path, prefix_path)?;
    Ok(prefix_path.to_path_buf())
}

pub fn list_backups(appid: u32) -> Vec<PathBuf> {
    let root = backup_root().join(appid.to_string());
    if let Ok(entries) = fs::read_dir(root) {
        let mut list: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        list.sort();
        list
    } else {
        Vec::new()
    }
}

/// List backups for all applications.
pub fn list_all_backups() -> BTreeMap<u32, Vec<PathBuf>> {
    let mut map = BTreeMap::new();
    let root = backup_root();
    if let Ok(app_dirs) = fs::read_dir(root) {
        for app_dir in app_dirs.flatten() {
            let path = app_dir.path();
            if path.is_dir() {
                if let Some(appid_str) = app_dir.file_name().to_str() {
                    if let Ok(appid) = appid_str.parse::<u32>() {
                        let backups = list_backups(appid);
                        if !backups.is_empty() {
                            map.insert(appid, backups);
                        }
                    }
                }
            }
        }
    }
    map
}

/// Format a backup directory name (usually a timestamp) into a human readable string.
pub fn format_backup_name(path: &Path) -> String {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(name, "%Y%m%d%H%M%S") {
            return dt.format("%Y-%m-%d %H:%M:%S").to_string();
        }
        name.to_string()
    } else {
        path.display().to_string()
    }
}

pub fn delete_backup(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

pub fn reset_prefix(prefix_path: &Path) -> Result<()> {
    if prefix_path.exists() {
        fs::remove_dir_all(prefix_path)?;
    }
    Ok(())
}

pub fn clear_shader_cache(appid: u32, libraries: &[SteamLibrary]) -> Result<()> {
    for lib in libraries {
        let cache = lib
            .steamapps_path()
            .join("shadercache")
            .join(appid.to_string());
        if cache.exists() {
            fs::remove_dir_all(cache)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_backup_and_restore() {
        let dir = tempdir().unwrap();
        let prefix = dir.path().join("prefix");
        fs::create_dir_all(prefix.join("sub")).unwrap();
        let mut f = fs::File::create(prefix.join("sub/file.txt")).unwrap();
        writeln!(f, "test").unwrap();

        let backup = create_backup(&prefix, 42).unwrap();
        assert!(backup.join("sub/file.txt").exists());

        fs::remove_dir_all(&prefix).unwrap();
        restore_prefix(&backup, &prefix).unwrap();
        assert!(prefix.join("sub/file.txt").exists());
    }
}
