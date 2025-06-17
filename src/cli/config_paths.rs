use crate::utils::user_config;

#[cfg(test)]
use once_cell::sync::Lazy;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(not(test))]
fn emit_paths(paths: Vec<std::path::PathBuf>) {
    if paths.is_empty() {
        println!("No localconfig.vdf files found");
    } else {
        for p in paths {
            println!("{}", p.display());
        }
    }
}

#[cfg(test)]
pub static EMITTED_PATHS: Lazy<Mutex<Vec<Vec<std::path::PathBuf>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
#[cfg(test)]
fn emit_paths(paths: Vec<std::path::PathBuf>) {
    EMITTED_PATHS.lock().unwrap().push(paths);
}

pub fn execute() {
    let paths = user_config::get_localconfig_paths();
    emit_paths(paths);
}
