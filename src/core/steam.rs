//! Steam-related operations.
//!
//! This module contains functions for interacting with Steam,
//! including finding libraries, games, and Proton prefixes.

use crate::core::models::{GameInfo, SteamLibrary};
use crate::error::{Error, Result};
use crate::utils::library;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

// Cache for Steam libraries with timestamp
struct LibraryCache {
    libraries: Vec<SteamLibrary>,
    timestamp: SystemTime,
}

// Cache for game manifests with timestamp
struct ManifestCache {
    games: Vec<GameInfo>,
    timestamp: SystemTime,
}

// Global caches with mutex protection
static LIBRARY_CACHE: Lazy<Mutex<Option<LibraryCache>>> = Lazy::new(|| Mutex::new(None));
static MANIFEST_CACHE: Lazy<Mutex<Option<ManifestCache>>> = Lazy::new(|| Mutex::new(None));

#[cfg(test)]
pub fn clear_caches() {
    *LIBRARY_CACHE.lock().unwrap() = None;
    *MANIFEST_CACHE.lock().unwrap() = None;
}

// Cache duration (5 seconds)
const CACHE_DURATION: std::time::Duration = std::time::Duration::from_secs(5);

/// Gets a list of Steam library folders with caching.
///
/// # Returns
///
/// A `Result` containing a vector of paths to Steam library folders,
/// or an error if Steam is not found or the library folders cannot be parsed.
///
/// # Errors
///
/// Returns an error if:
/// - The home directory cannot be found
/// - The Steam installation cannot be found
/// - The libraryfolders.vdf file cannot be parsed
pub fn get_steam_libraries() -> Result<Vec<SteamLibrary>> {
    let mut cache = LIBRARY_CACHE.lock().unwrap();

    // Check if cache is valid
    if let Some(cached) = &*cache {
        if SystemTime::now().duration_since(cached.timestamp).unwrap() < CACHE_DURATION {
            return Ok(cached.libraries.clone());
        }
    }

    // Cache invalid or empty, fetch fresh data
    let mut vdf_path = None;
    for dir in crate::utils::steam_paths::config_dirs() {
        let candidate = dir.join("libraryfolders.vdf");
        if candidate.exists() {
            vdf_path = Some(candidate);
            break;
        }
    }

    let vdf_path =
        vdf_path.ok_or_else(|| Error::SteamConfigNotFound(PathBuf::from("libraryfolders.vdf")))?;

    let vdf_path_str = vdf_path
        .to_str()
        .ok_or(Error::Parse("Invalid path".to_string()))?;
    let library_paths = library::parse_libraryfolders_vdf(vdf_path_str).ok_or(Error::Parse(
        "Failed to parse libraryfolders.vdf".to_string(),
    ))?;

    let mut libraries = Vec::new();
    for path in library_paths {
        if let Ok(library) = SteamLibrary::new(path) {
            libraries.push(library);
        }
    }

    if libraries.is_empty() {
        return Err(Error::SteamNotFound);
    }

    // Update cache
    *cache = Some(LibraryCache {
        libraries: libraries.clone(),
        timestamp: SystemTime::now(),
    });

    Ok(libraries)
}

/// Finds the Proton prefix for a specific AppID.
///
/// # Arguments
///
/// * `appid` - The Steam AppID of the game
/// * `libraries` - A slice of Steam library folders
///
/// # Returns
///
/// An `Option` containing the path to the Proton prefix if found,
/// or `None` if no prefix is found.
pub fn find_proton_prefix(appid: u32, libraries: &[SteamLibrary]) -> Option<PathBuf> {
    for library in libraries {
        let candidate = library.compatdata_path().join(appid.to_string());
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Finds the Steam userdata directory for a specific AppID.
///
/// This uses the active Steam user's `localconfig.vdf` location to
/// determine the account ID and checks all detected `userdata` bases.
/// Returns `Some(PathBuf)` if the directory exists.
pub fn find_userdata_dir(appid: u32) -> Option<PathBuf> {
    if let Some(cfg) = crate::utils::user_config::expected_localconfig_path() {
        if let Some(user_dir) = cfg.parent().and_then(|p| p.parent()) {
            let candidate = user_dir.join(appid.to_string());
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Searches for games by name.
///
/// # Arguments
///
/// * `name` - The name to search for
///
/// # Returns
///
/// A `Result` containing a vector of `GameInfo` structs,
/// or an error if the search fails.
///
/// # Errors
///
/// Returns an error if:
/// - The Steam libraries cannot be found
pub fn search_games(name: &str) -> Result<Vec<GameInfo>> {
    let libraries = get_steam_libraries()?;
    let mut results = Vec::new();

    // First collect all matching games
    let mut matching_games = Vec::new();

    for library in &libraries {
        let steamapps_path = library.steamapps_path();
        if let Ok(entries) = fs::read_dir(&steamapps_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("acf") {
                    if let Some((appid, game_name, last_played)) = library::parse_appmanifest(&path)
                    {
                        if game_name.to_lowercase().contains(&name.to_lowercase()) {
                            matching_games.push((appid, game_name, last_played));
                        }
                    }
                }
            }
        }
    }

    // Then find prefixes for all matching games
    for (appid, game_name, last_played) in matching_games {
        if let Some(prefix_path) = find_proton_prefix(appid, &libraries) {
            if let Ok(game_info) = GameInfo::new(appid, game_name, prefix_path, true, last_played) {
                results.push(game_info);
            }
        }
    }

    Ok(results)
}

/// Loads games from a single library
fn load_games_from_library(library: &SteamLibrary) -> Result<Vec<GameInfo>> {
    let mut games = Vec::new();
    let steamapps_path = library.steamapps_path();

    // Parse all appmanifest files
    if let Ok(entries) = fs::read_dir(&steamapps_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                if fname.starts_with("appmanifest_") && fname.ends_with(".acf") {
                    if let Some((appid, game_name, last_played)) = library::parse_appmanifest(&path)
                    {
                        let prefix_path = library.compatdata_path().join(appid.to_string());
                        if let Ok(game_info) =
                            GameInfo::new(appid, game_name, prefix_path, true, last_played)
                        {
                            games.push(game_info);
                        }
                    }
                }
            }
        }
    }

    // Check any prefix that lacks a manifest
    let compatdata = library.compatdata_path();
    if let Ok(compat_entries) = fs::read_dir(compatdata) {
        for c in compat_entries.flatten() {
            if let Ok(appid) = c.file_name().to_string_lossy().parse::<u32>() {
                // Check if the game is already in the list
                if !games.iter().any(|g| g.app_id() == appid) {
                    let prefix_path = c.path();
                    if let Ok(game_info) = GameInfo::new(
                        appid,
                        format!("App {}", appid),
                        prefix_path,
                        false,
                        0, // No manifest means no last played time
                    ) {
                        games.push(game_info);
                    }
                }
            }
        }
    }

    Ok(games)
}

/// Loads all games from the given Steam libraries with caching and parallel processing.
pub fn load_games_from_libraries(libraries: &[SteamLibrary]) -> Result<Vec<GameInfo>> {
    let mut cache = MANIFEST_CACHE.lock().unwrap();

    // Check if cache is valid
    if let Some(cached) = &*cache {
        if SystemTime::now().duration_since(cached.timestamp).unwrap() < CACHE_DURATION {
            return Ok(cached.games.clone());
        }
    }

    // Cache invalid or empty, fetch fresh data
    let mut games = Vec::new();

    // Process libraries in parallel
    let results: Vec<Result<Vec<GameInfo>>> = libraries
        .par_iter()
        .map(|library| load_games_from_library(library))
        .collect();

    // Combine results
    for result in results {
        match result {
            Ok(mut library_games) => games.append(&mut library_games),
            Err(e) => log::error!("Failed to load games from library: {}", e),
        }
    }

    // Update cache
    *cache = Some(ManifestCache {
        games: games.clone(),
        timestamp: SystemTime::now(),
    });

    Ok(games)
}

/// Refresh information for a single game by reading its latest manifest and prefix data.
pub fn refresh_game_info(app_id: u32) -> Result<GameInfo> {
    let libraries = get_steam_libraries()?;

    let mut prefix_path = None;
    let mut name = None;
    let mut last_played = 0;
    let mut has_manifest = false;

    for lib in &libraries {
        let manifest = lib
            .steamapps_path()
            .join(format!("appmanifest_{}.acf", app_id));
        if manifest.exists() {
            if let Some((_, game_name, lp)) = library::parse_appmanifest(&manifest) {
                name = Some(game_name);
                last_played = lp;
                has_manifest = true;
            }
            prefix_path = Some(lib.compatdata_path().join(app_id.to_string()));
            break;
        }
    }

    if prefix_path.is_none() {
        prefix_path = find_proton_prefix(app_id, &libraries);
    }

    let prefix = prefix_path.ok_or(Error::InvalidAppId(app_id.to_string()))?;
    let game_name = name.unwrap_or_else(|| format!("App {}", app_id));

    GameInfo::new(app_id, game_name, prefix, has_manifest, last_played)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_proton_prefix() {
        let dir = tempdir().unwrap();
        let library = SteamLibrary::new(dir.path().to_path_buf()).unwrap();

        // Create a mock Steam library structure
        let compatdata = library.compatdata_path();
        std::fs::create_dir_all(&compatdata).unwrap();

        // Create a mock prefix
        let prefix = compatdata.join("123456");
        std::fs::create_dir_all(&prefix).unwrap();

        // Test finding the prefix
        let libraries = vec![library];
        let result = find_proton_prefix(123456, &libraries);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), prefix);

        // Test with non-existent prefix
        let result = find_proton_prefix(999999, &libraries);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_userdata_dir() {
        let _guard = crate::test_helpers::TEST_MUTEX.lock().unwrap();
        let dir = tempdir().unwrap();
        let home = dir.path();
        let userdata = home.join(".steam/steam/userdata/111111111");
        std::fs::create_dir_all(&userdata).unwrap();
        std::fs::create_dir_all(userdata.join("123456")).unwrap();
        let config_dir = home.join(".steam/steam/config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let login = config_dir.join("loginusers.vdf");
        let contents = "\"users\" { \"111111111\" { \"MostRecent\" \"1\" } }";
        std::fs::write(&login, contents).unwrap();

        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home);

        let result = find_userdata_dir(123456);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), userdata.join("123456"));

        let result = find_userdata_dir(999999);
        assert!(result.is_none());

        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        }
    }
}
