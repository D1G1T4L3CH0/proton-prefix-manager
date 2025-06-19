use keyvalues_parser::{Value, Vdf};
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};

struct ManifestEntry {
    contents: String,
    modified: SystemTime,
}

static MANIFEST_FILE_CACHE: Lazy<Mutex<HashMap<PathBuf, ManifestEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static MANIFEST_FILE_ORDER: Lazy<Mutex<VecDeque<PathBuf>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));
const MANIFEST_CACHE_LIMIT: usize = 20;

pub fn read_manifest_cached(path: &Path) -> Option<String> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let mut cache = MANIFEST_FILE_CACHE.lock().unwrap();
    let mut order = MANIFEST_FILE_ORDER.lock().unwrap();
    if let Some(entry) = cache.get(path) {
        if entry.modified >= modified {
            return Some(entry.contents.clone());
        }
    }
    let contents = fs::read_to_string(path).ok()?;
    cache.insert(
        path.to_path_buf(),
        ManifestEntry {
            contents: contents.clone(),
            modified,
        },
    );
    order.retain(|p| p != path);
    order.push_back(path.to_path_buf());
    if order.len() > MANIFEST_CACHE_LIMIT {
        if let Some(old) = order.pop_front() {
            cache.remove(&old);
        }
    }
    Some(contents)
}

pub fn update_manifest_cache(path: &Path, contents: &str) {
    if let Ok(modified) = fs::metadata(path).and_then(|m| m.modified()) {
        let mut cache = MANIFEST_FILE_CACHE.lock().unwrap();
        let mut order = MANIFEST_FILE_ORDER.lock().unwrap();
        cache.insert(
            path.to_path_buf(),
            ManifestEntry {
                contents: contents.to_string(),
                modified,
            },
        );
        order.retain(|p| p != path);
        order.push_back(path.to_path_buf());
        if order.len() > MANIFEST_CACHE_LIMIT {
            if let Some(old) = order.pop_front() {
                cache.remove(&old);
            }
        }
    }
}
pub fn clear_manifest_cache() {
    MANIFEST_FILE_CACHE.lock().unwrap().clear();
    MANIFEST_FILE_ORDER.lock().unwrap().clear();
}

pub fn parse_libraryfolders_vdf(vdf_path: &str) -> Option<Vec<PathBuf>> {
    let content = fs::read_to_string(vdf_path).ok()?;
    let vdf = Vdf::parse(&content).ok()?;
    let mut library_paths = Vec::new();
    let folders_obj_opt = if vdf.key == "libraryfolders" {
        vdf.value.get_obj()
    } else {
        vdf.value
            .get_obj()
            .and_then(|o| o.get("libraryfolders"))
            .and_then(|v| v.first())
            .and_then(Value::get_obj)
    };
    if let Some(folders) = folders_obj_opt {
        for (_k, vals) in folders.iter() {
            if let Some(val) = vals.first() {
                if let Some(folder_obj) = val.get_obj() {
                    if let Some(path_val) = folder_obj.get("path").and_then(|v| v.first()) {
                        if let Some(path_str) = path_val.get_str() {
                            let pb = PathBuf::from(path_str);
                            if pb.exists() {
                                library_paths.push(pb);
                            }
                        }
                    }
                }
            }
        }
    }
    Some(library_paths)
}

pub fn parse_appmanifest(path: &Path) -> Option<(u32, String, u64)> {
    let contents = read_manifest_cached(path)?;
    let vdf = Vdf::parse(&contents).ok()?;
    let app_state = vdf.value.get_obj()?;
    let appid = app_state.get("appid")?.first()?.get_str()?.parse().ok()?;
    let name = app_state.get("name")?.first()?.get_str()?.to_string();
    let last_played = app_state
        .get("LastPlayed")
        .and_then(|v| v.first())
        .and_then(|v| v.get_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    Some((appid, name, last_played))
}

// Cache for game names to avoid repeated file reads

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
    fn test_library_parsing() {
        let dir = tempdir().unwrap();
        let lib1 = dir.path().join("lib1");
        let lib2 = dir.path().join("lib2");
        std::fs::create_dir_all(&lib1).unwrap();
        std::fs::create_dir_all(&lib2).unwrap();
        let vdf_path = dir.path().join("libraryfolders.vdf");
        let content = format!(
            "\"libraryfolders\" {{\n    \"0\" {{\n        \"path\" \"{}\"\n    }}\n    \"1\" {{\n        \"path\" \"{}\"\n    }}\n}}",
            lib1.display(),
            lib2.display()
        );
        std::fs::write(&vdf_path, content).unwrap();
        let libs = parse_libraryfolders_vdf(vdf_path.to_str().unwrap()).unwrap();
        assert_eq!(libs.len(), 2);
        assert!(libs.contains(&lib1));
        assert!(libs.contains(&lib2));
    }
}
