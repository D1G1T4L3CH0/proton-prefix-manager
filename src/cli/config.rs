use crate::core::steam;
use crate::utils::manifest as manifest_utils;
use crate::utils::user_config;
use std::fs;

pub fn execute(
    appid: u32,
    launch: Option<String>,
    proton: Option<String>,
    cloud: Option<bool>,
    auto_update: Option<String>,
) {
    log::debug!(
        "config command: appid={} launch={:?} proton={:?} cloud={:?} auto_update={:?}",
        appid,
        launch,
        proton,
        cloud,
        auto_update
    );
    if launch.is_none() && proton.is_none() && cloud.is_none() && auto_update.is_none() {
        println!("No configuration changes specified.");
        return;
    }

    match steam::get_steam_libraries() {
        Ok(libraries) => {
            for lib in libraries {
                let manifest = lib
                    .steamapps_path()
                    .join(format!("appmanifest_{}.acf", appid));
                if manifest.exists() {
                    match fs::read_to_string(&manifest) {
                        Ok(mut contents) => {
                            if let Some(v) = launch {
                                contents = manifest_utils::update_or_insert(&contents, "LaunchOptions", &v);
                                if let Err(e) = user_config::set_launch_options(appid, &v) {
                                    eprintln!("Failed to update launch options: {}", e);
                                }
                            }
                            if let Some(v) = proton {
                                contents = manifest_utils::update_or_insert(&contents, "CompatToolOverride", &v);
                                if let Err(e) = user_config::set_compat_tool(appid, &v) {
                                    eprintln!("Failed to update compatibility tool: {}", e);
                                }
                            }
                            if let Some(v) = cloud {
                                let val = if v { "1" } else { "0" };
                                contents = manifest_utils::update_or_insert(&contents, "AllowCloudSaves", val);
                            }
                            if let Some(v) = auto_update {
                                contents = manifest_utils::update_or_insert(&contents, "AutoUpdateBehavior", &v);
                            }
                            if let Err(e) = fs::write(&manifest, contents) {
                                eprintln!("Failed to write manifest: {}", e);
                            } else {
                                println!("Updated {}", manifest.display());
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read manifest {}: {}", manifest.display(), e);
                        }
                    }
                    return;
                }
            }
            println!("Manifest not found for {}", appid);
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }
}
