//! Data models used throughout the application.

use std::path::PathBuf;
use crate::error::{Error, Result};

/// Represents a Steam game with its Proton prefix information.
#[derive(Clone, Debug)]
pub struct GameInfo {
    /// The Steam AppID of the game
    app_id: u32,
    
    /// The name of the game
    name: String,
    
    /// The path to the Proton prefix for this game
    prefix_path: PathBuf,
    
    /// Whether the game has a manifest file (appmanifest_*.acf)
    has_manifest: bool,

    /// Last time the game was played (Unix timestamp)
    last_played: u64,
}

impl GameInfo {
    /// Creates a new GameInfo instance with validation.
    pub fn new(app_id: u32, name: String, prefix_path: PathBuf, has_manifest: bool, last_played: u64) -> Result<Self> {
        if app_id == 0 {
            return Err(Error::InvalidAppId("AppID cannot be zero".to_string()));
        }

        if name.is_empty() {
            return Err(Error::InvalidManifest("Game name cannot be empty".to_string()));
        }

        Ok(Self {
            app_id,
            name,
            prefix_path,
            has_manifest,
            last_played,
        })
    }

    /// Gets the AppID of the game.
    pub fn app_id(&self) -> u32 {
        self.app_id
    }

    /// Gets the name of the game.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the path to the Proton prefix.
    pub fn prefix_path(&self) -> &PathBuf {
        &self.prefix_path
    }

    /// Checks if the game has a manifest file.
    pub fn has_manifest(&self) -> bool {
        self.has_manifest
    }

    /// Gets the last played time as a Unix timestamp.
    pub fn last_played(&self) -> u64 {
        self.last_played
    }

    /// Checks if the Proton prefix exists.
    pub fn prefix_exists(&self) -> bool {
        self.prefix_path.exists()
    }
}

/// Represents a Steam library folder with validation and functionality.
#[derive(Clone, Debug)]
pub struct SteamLibrary {
    /// The path to the library folder
    path: PathBuf,
}

impl SteamLibrary {
    /// Creates a new SteamLibrary instance with validation.
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(Error::LibraryNotFound(path));
        }

        if !path.is_dir() {
            return Err(Error::FileSystemError(format!("{} is not a directory", path.display())));
        }

        Ok(Self { path })
    }

    /// Gets the path to the library folder.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Gets the path to the steamapps directory.
    pub fn steamapps_path(&self) -> PathBuf {
        self.path.join("steamapps")
    }

    /// Gets the path to the compatdata directory.
    pub fn compatdata_path(&self) -> PathBuf {
        self.steamapps_path().join("compatdata")
    }

    /// Checks if the library is valid and accessible.
    pub fn is_valid(&self) -> bool {
        self.path.exists() && self.steamapps_path().exists()
    }

    /// Joins a path to the library path.
    pub fn join<P: AsRef<std::path::Path>>(&self, path: P) -> PathBuf {
        self.path.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_game_info_creation() {
        let temp_dir = tempdir().unwrap();
        let prefix_path = temp_dir.path().join("compatdata").join("123456");
        
        // Test valid creation
        let game = GameInfo::new(123456, "Test Game".to_string(), prefix_path.clone(), true, 0).unwrap();
        assert_eq!(game.app_id(), 123456);
        assert_eq!(game.name(), "Test Game");
        assert_eq!(game.prefix_path(), &prefix_path);
        assert!(game.has_manifest());

        // Test invalid AppID
        assert!(GameInfo::new(0, "Test Game".to_string(), prefix_path.clone(), true, 0).is_err());

        // Test empty name
        assert!(GameInfo::new(123456, String::new(), prefix_path.clone(), true, 0).is_err());
    }

    #[test]
    fn test_steam_library_creation() {
        let temp_dir = tempdir().unwrap();
        // Create the steamapps directory to satisfy `is_valid` checks
        std::fs::create_dir(temp_dir.path().join("steamapps")).unwrap();

        // Test valid library
        let library = SteamLibrary::new(temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(library.path(), temp_dir.path());
        assert!(library.is_valid());

        // Test non-existent path
        let non_existent = temp_dir.path().join("nonexistent");
        assert!(SteamLibrary::new(non_existent).is_err());

        // Test join method
        let joined = library.join("test/path");
        assert_eq!(joined, temp_dir.path().join("test/path"));
    }
} 