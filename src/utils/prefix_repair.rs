use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

/// Attempt to repair a Proton prefix.
///
/// This will recreate critical folders and run `wineboot` to
/// regenerate missing registry files.
pub fn repair_prefix(prefix: &Path) -> Result<()> {
    let pfx = prefix.join("pfx");
    if !pfx.exists() {
        fs::create_dir_all(&pfx)?;
    }
    // Ensure core directories exist
    fs::create_dir_all(pfx.join("drive_c"))?;
    fs::create_dir_all(pfx.join("dosdevices"))?;
    let _ = fs::File::create(pfx.join(".update-timestamp"));

    // Run wineboot to regenerate registry files
    let status = Command::new("wineboot")
        .env("WINEPREFIX", &pfx)
        .status()
        .map_err(Error::from)?;
    if !status.success() {
        return Err(Error::FileSystemError("wineboot failed".into()));
    }
    Ok(())
}
