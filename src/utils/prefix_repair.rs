use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

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
    log::debug!("running wineboot for {:?}", pfx);
    let status = Command::new("wineboot")
        .env("WINEPREFIX", &pfx)
        .status()
        .map_err(Error::from)?;
    if !status.success() {
        return Err(Error::FileSystemError("wineboot failed".into()));
    }
    Ok(())
}
