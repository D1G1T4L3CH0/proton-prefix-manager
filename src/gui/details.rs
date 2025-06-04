use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::core::models::GameInfo;

pub struct GameDetails<'a> {
    game: Option<&'a GameInfo>,
    id: egui::Id,  // Add a unique ID for this instance
}

impl<'a> GameDetails<'a> {
    pub fn new(game: Option<&'a GameInfo>) -> Self {
        Self {
            game,
            id: egui::Id::new("game_details"),
        }
    }

    fn show_path(&mut self, ui: &mut egui::Ui, label: &str, path: &Path) {
        // Create a grid for label and content
        egui::Grid::new(format!("grid_{}", label))
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                // Label column
                ui.strong(label);

                // Action buttons group
                ui.horizontal(|ui| {
                    let path_str = path.display().to_string();
                    let copy_id = self.id.with("copy").with(&path_str);
                    let current_time = ui.input(|i| i.time);
                    let copy_time = ui.data_mut(|d| d.get_temp::<f64>(copy_id).unwrap_or(0.0));
                    let is_copied = copy_time > current_time;

                    let button_size = egui::vec2(24.0, 24.0);
                    
                    // Copy button with feedback
                    let copy_button = ui.add_sized(
                        button_size,
                        egui::Button::new(
                            if is_copied {
                                egui::RichText::new("✔").color(egui::Color32::from_rgb(50, 255, 50))
                            } else {
                                egui::RichText::new("📋")
                            }
                        )
                    );

                    if copy_button.clicked() {
                        ui.ctx().copy_text(path_str.clone());
                        ui.data_mut(|d| d.insert_temp(copy_id, current_time + 2.0));
                        ui.ctx().request_repaint();
                    }
                    copy_button.on_hover_text(format!("Copy path: {}", path_str));

                    // Open folder button
                    let open_button = ui.add_sized(button_size, egui::Button::new("📂"));
                    if open_button.clicked() {
                        let _ = open::that(path);
                    }
                    open_button.on_hover_text(format!("Open: {}", path_str));
                });

                ui.end_row();
            });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if let Some(game) = self.game {
            // Game title and AppID in a header section
            ui.heading(game.name());
            ui.horizontal(|ui| {
                ui.label("App ID:");
                ui.monospace(game.app_id().to_string());
            });
            ui.add_space(12.0);

            // Prefix Information
            egui::CollapsingHeader::new("Prefix Information")
                .default_open(true)
                .show(ui, |ui| {
                    self.show_path(ui, "Prefix Path:", game.prefix_path());
                    
                    if let Ok(metadata) = fs::metadata(game.prefix_path()) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(time) = modified.duration_since(UNIX_EPOCH) {
                                let datetime = chrono::DateTime::<chrono::Local>::from(
                                    SystemTime::UNIX_EPOCH + time
                                );
                                egui::Grid::new("modified_time")
                                    .num_columns(2)
                                    .spacing([8.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Last Modified:");
                                        ui.monospace(datetime.format("%Y-%m-%d %H:%M").to_string());
                                        ui.end_row();
                                    });
                            }
                        }
                    }
                    
                    let drive_c = game.prefix_path().join("pfx/drive_c");
                    if drive_c.exists() {
                        self.show_path(ui, "Drive C:", &drive_c);
                    }
                });

            // Proton Information
            egui::CollapsingHeader::new("🚀 Proton Information")
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(version) = detect_proton_version(game.prefix_path()) {
                        ui.horizontal(|ui| {
                            ui.label("Version:");
                            ui.monospace(&version);
                        });
                        log::debug!("Displaying Proton version: {}", version);
                    } else {
                        ui.label("Proton version could not be detected");
                        log::debug!("No Proton version to display");
                    }
                    
                    if has_dxvk(game.prefix_path()) {
                        ui.label("✓ DXVK is enabled");
                    }
                });

            // Game Details
            egui::CollapsingHeader::new("🎮 Game Details")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(if game.has_manifest() {
                        "✓ Game has a manifest file"
                    } else {
                        "❌ No manifest file found"
                    });
                    
                    // Last played time
                    let last_played = game.last_played();
                    if last_played > 0 {
                        let datetime = chrono::DateTime::<chrono::Local>::from(
                            std::time::UNIX_EPOCH + std::time::Duration::from_secs(last_played)
                        );
                        ui.horizontal(|ui| {
                            ui.label("Last played:");
                            ui.monospace(datetime.format("%Y-%m-%d %H:%M").to_string());
                        });
                    }
                    
                    if let Some(install_dir) = find_install_dir(game.app_id()) {
                        self.show_path(ui, "Install Directory:", &install_dir);
                    }
                });

            ui.add_space(8.0);

            // External Links
            ui.horizontal(|ui| {
                if ui.button("🔗 SteamDB").clicked() {
                    let _ = open::that(format!("https://steamdb.info/app/{}/", game.app_id()));
                }
                
                if ui.button("🎮 ProtonDB").clicked() {
                    let _ = open::that(format!("https://www.protondb.com/app/{}", game.app_id()));
                }
                
                if ui.button("📚 PCGamingWiki").clicked() {
                    let _ = open::that(format!("https://www.pcgamingwiki.com/api/appid.php?appid={}", game.app_id()));
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a game to view details");
            });
        }
    }
}

// Helper functions
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} bytes", size)
    }
}

fn detect_proton_version(prefix_path: &Path) -> Option<String> {
    log::debug!("Detecting Proton version for prefix: {:?}", prefix_path);

    // First check the 'version' file in the prefix
    let version_file = prefix_path.join("version");
    log::debug!("Checking version file: {:?}", version_file);
    if version_file.exists() {
        if let Ok(contents) = fs::read_to_string(&version_file) {
            let version = contents.trim().to_string();
            log::debug!("Found version in prefix: {}", version);
            return Some(version);
        }
    }
    
    // Check for 'version' in the parent directory (compatdata)
    if let Some(parent) = prefix_path.parent() {
        let version_file = parent.join("version");
        log::debug!("Checking parent version file: {:?}", version_file);
        if version_file.exists() {
            if let Ok(contents) = fs::read_to_string(&version_file) {
                let version = contents.trim().to_string();
                log::debug!("Found version in parent: {}", version);
                return Some(version);
            }
        }
    }

    // Check for version in the prefix's parent directory name (e.g., Proton 8.0)
    if let Some(parent) = prefix_path.parent() {
        if let Some(parent_name) = parent.file_name() {
            if let Some(parent_str) = parent_name.to_str() {
                if parent_str.to_lowercase().contains("proton") {
                    log::debug!("Found version in parent directory name: {}", parent_str);
                    return Some(parent_str.to_string());
                }
            }
        }
    }

    // Check for toolmanifest.vdf in the prefix
    let toolmanifest = prefix_path.join("toolmanifest.vdf");
    log::debug!("Checking toolmanifest: {:?}", toolmanifest);
    if toolmanifest.exists() {
        if let Ok(contents) = fs::read_to_string(&toolmanifest) {
            for line in contents.lines() {
                let line = line.trim();
                if line.starts_with("\"name\"") {
                    if let Some(name) = line.split('"').nth(3) {
                        if name.contains("Proton") {
                            log::debug!("Found version in toolmanifest: {}", name);
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }
    }

    // Check for proton_version in the prefix
    let proton_version = prefix_path.join("proton_version");
    log::debug!("Checking proton_version file: {:?}", proton_version);
    if proton_version.exists() {
        if let Ok(contents) = fs::read_to_string(&proton_version) {
            let version = contents.trim().to_string();
            log::debug!("Found version in proton_version: {}", version);
            return Some(version);
        }
    }

    // Check for the dist.info file which some Proton versions use
    let dist_info = prefix_path.join("dist.info");
    log::debug!("Checking dist.info file: {:?}", dist_info);
    if dist_info.exists() {
        if let Ok(contents) = fs::read_to_string(&dist_info) {
            if let Some(version_line) = contents.lines().find(|l| l.contains("DIST_VERSION=")) {
                if let Some(version) = version_line.split('=').nth(1) {
                    let version = format!("Proton {}", version.trim());
                    log::debug!("Found version in dist.info: {}", version);
                    return Some(version);
                }
            }
        }
    }

    log::debug!("No Proton version found for prefix: {:?}", prefix_path);
    None
}

fn has_dxvk(prefix_path: &Path) -> bool {
    // Check for DXVK DLLs in the prefix
    let dll_path = prefix_path.join("pfx/drive_c/windows/system32");
    if dll_path.exists() {
        let dlls = ["d3d11.dll", "d3d10.dll", "d3d9.dll"];
        dlls.iter().any(|dll| dll_path.join(dll).exists())
    } else {
        false
    }
}

fn find_install_dir(app_id: u32) -> Option<std::path::PathBuf> {
    use crate::core::steam;
    
    if let Ok(libraries) = steam::get_steam_libraries() {
        for library in libraries {
            let app_manifest = library.join("steamapps").join(format!("appmanifest_{}.acf", app_id));
            if app_manifest.exists() {
                if let Ok(contents) = fs::read_to_string(&app_manifest) {
                    // Look for the "installdir" field in the manifest
                    if let Some(path) = contents.lines()
                        .find(|line| line.contains("installdir"))
                        .and_then(|line| line.split('"').nth(3))
                    {
                        return Some(library.join("steamapps/common").join(path));
                    }
                }
            }
        }
    }
    None
}

