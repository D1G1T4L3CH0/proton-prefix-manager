use std::fs;
use std::path::{Path, PathBuf};

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
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Back up a Proton prefix by copying it to the given destination directory.
pub fn backup_prefix(prefix_path: &Path, backup_path: &Path) -> Result<PathBuf> {
    if !prefix_path.exists() {
        return Err(Error::FileSystemError(format!(
            "Prefix not found: {}",
            prefix_path.display()
        )));
    }
    if backup_path.exists() {
        return Err(Error::FileSystemError(format!(
            "Backup destination already exists: {}",
            backup_path.display()
        )));
    }
    copy_dir_recursive(prefix_path, backup_path)?;
    Ok(backup_path.to_path_buf())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[test]
    fn test_backup_and_restore() {
        let dir = tempdir().unwrap();
        let prefix = dir.path().join("prefix");
        let backup = dir.path().join("backup");
        fs::create_dir_all(prefix.join("sub")).unwrap();
        let mut f = fs::File::create(prefix.join("sub/file.txt")).unwrap();
        writeln!(f, "test").unwrap();

        backup_prefix(&prefix, &backup).unwrap();
        assert!(backup.join("sub/file.txt").exists());

        fs::remove_dir_all(&prefix).unwrap();
        restore_prefix(&backup, &prefix).unwrap();
        assert!(prefix.join("sub/file.txt").exists());
    }
}
