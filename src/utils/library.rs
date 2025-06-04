use std::{fs, path::{Path, PathBuf}};
use regex::Regex;
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref PATH_REGEX: Regex = Regex::new(r#""path"\s+"([^"]+)""#).unwrap();
}

pub fn parse_libraryfolders_vdf(vdf_path: &str) -> Option<Vec<PathBuf>> {
    let content = fs::read_to_string(vdf_path).ok()?;
    
    let mut library_paths = Vec::new();
    for cap in PATH_REGEX.captures_iter(&content) {
        let path = PathBuf::from(&cap[1]);
        if path.exists() {
            library_paths.push(path);
        }
    }

    Some(library_paths)
}

pub fn parse_appmanifest(path: &Path) -> Option<(u32, String, u64)> {
    let contents = fs::read_to_string(path).ok()?;
    
    // Use a more efficient approach with fewer allocations
    let mut appid: Option<u32> = None;
    let mut name: Option<String> = None;
    let mut last_played: Option<u64> = None;
    
    // Only parse until we have all values
    for line in contents.lines() {
        let trimmed = line.trim();
        if appid.is_none() && trimmed.starts_with("\"appid\"") {
            if let Some(val) = trimmed.split('"').nth(3) {
                appid = val.parse().ok();
            }
        } else if name.is_none() && trimmed.starts_with("\"name\"") {
            if let Some(val) = trimmed.split('"').nth(3) {
                name = Some(val.to_string());
            }
        } else if last_played.is_none() && trimmed.starts_with("\"LastPlayed\"") {
            if let Some(val) = trimmed.split('"').nth(3) {
                last_played = val.parse().ok();
            }
        }
        
        // Early return if we have all values
        if appid.is_some() && name.is_some() && last_played.is_some() {
            break;
        }
    }
    
    match (appid, name, last_played) {
        (Some(a), Some(n), Some(l)) => Some((a, n, l)),
        (Some(a), Some(n), None) => Some((a, n, 0)), // Default to 0 if LastPlayed is not found
        _ => None,
    }
}

// Cache for game names to avoid repeated file reads
lazy_static! {
    static ref GAME_NAME_CACHE: std::sync::Mutex<HashMap<u32, String>> = std::sync::Mutex::new(HashMap::new());
}

pub fn find_game_name_by_appid(appid: u32) -> Option<String> {
    // Check cache first
    {
        let cache = GAME_NAME_CACHE.lock().unwrap();
        if let Some(name) = cache.get(&appid) {
            return Some(name.clone());
        }
    }
    
    // If not in cache, look it up
    if let Some(home) = dirs::home_dir() {
        let steam_path = home.join(".steam/steam");
        let libraries = parse_libraryfolders_vdf(
            steam_path.join("config/libraryfolders.vdf").to_str()?
        )?;
        
        for library in &libraries {
            let steamapps_path = library.join("steamapps");
            let appmanifest_path = steamapps_path.join(format!("appmanifest_{}.acf", appid));
            
            if appmanifest_path.exists() {
                if let Some((_, name, _)) = parse_appmanifest(&appmanifest_path) {
                    // Add to cache
                    let mut cache = GAME_NAME_CACHE.lock().unwrap();
                    cache.insert(appid, name.clone());
                    return Some(name);
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_appmanifest() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("appmanifest_123456.acf");
        
        let content = r#"
        "AppState"
        {
            "appid"     "123456"
            "name"      "Test Game"
            "other"     "value"
        }
        "#;
        
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        let result = parse_appmanifest(&file_path);
        assert!(result.is_some());
        
        let (appid, name, _) = result.unwrap();
        assert_eq!(appid, 123456);
        assert_eq!(name, "Test Game");
    }

    #[test]
    fn test_parse_appmanifest_missing_fields() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("appmanifest_incomplete.acf");
        
        let content = r#"
        "AppState"
        {
            "appid"     "123456"
            // No name field
        }
        "#;
        
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        
        let result = parse_appmanifest(&file_path);
        assert!(result.is_none());
    }

    #[test]
    fn test_game_name_cache() {
        // This test verifies that the cache works
        // First call should add to cache, second should use cache
        
        // We can't easily test this directly, but we can verify the cache exists
        let cache = GAME_NAME_CACHE.lock().unwrap();
        assert!(cache.is_empty());
        // Drop the lock
        drop(cache);
        
        // Insert a test value
        {
            let mut cache = GAME_NAME_CACHE.lock().unwrap();
            cache.insert(12345, "Test Cache Game".to_string());
        }
        
        // Verify it's there
        {
            let cache = GAME_NAME_CACHE.lock().unwrap();
            assert_eq!(cache.get(&12345), Some(&"Test Cache Game".to_string()));
        }
    }

    #[test]
    fn test_library_parsing() {
        // Example test, replace with real logic later
        assert!(true);
    }
}
