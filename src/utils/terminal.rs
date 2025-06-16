use std::path::Path;
use std::process::Command;

use super::dependencies::command_available;

/// Find a usable terminal emulator command.
///
/// Checks the `TERMINAL` environment variable first, then some common
/// terminal emulators.
pub fn find_terminal() -> Option<String> {
    if let Ok(term) = std::env::var("TERMINAL") {
        if command_available(&term) {
            return Some(term);
        }
    }

    for cmd in ["x-terminal-emulator", "gnome-terminal", "konsole", "xterm"] {
        if command_available(cmd) {
            return Some(cmd.to_string());
        }
    }

    None
}

/// Check if any terminal emulator is available.
pub fn terminal_available() -> bool {
    find_terminal().is_some()
}

/// Launch a terminal with `WINEPREFIX` and working directory set to `path`.
pub fn open_terminal(path: &Path) -> std::io::Result<()> {
    let term = find_terminal()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No terminal emulator found"))?;

    Command::new(term)
        .env("WINEPREFIX", path)
        .current_dir(path)
        .spawn()
        .map(|_| ())
}
