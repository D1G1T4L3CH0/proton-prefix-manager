use crate::core::models::GameInfo;
use crate::core::steam;
use crate::utils::backup as backup_utils;
use eframe::egui;
use std::thread;
use crate::cli::{protontricks, winecfg};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tinyfiledialogs as tfd;
use chrono::NaiveDateTime;

pub struct GameDetails<'a> {
    game: Option<&'a GameInfo>,
    id: egui::Id, // Add a unique ID for this instance
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
                        egui::Button::new(if is_copied {
                            egui::RichText::new("✔").color(egui::Color32::from_rgb(50, 255, 50))
                        } else {
                            egui::RichText::new("📋")
                        }),
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

    fn prefix_available(&self) -> bool {
        if let Some(game) = self.game {
            let path = game.prefix_path();
            if path.exists() {
                if let Ok(mut entries) = fs::read_dir(path) {
                    return entries.next().is_some();
                }
            }
        }
        false
    }

    fn format_backup_name(path: &Path) -> String {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Ok(dt) = NaiveDateTime::parse_from_str(name, "%Y%m%d%H%M%S") {
                return dt.format("%Y-%m-%d %H:%M:%S").to_string();
            }
            name.to_string()
        } else {
            path.display().to_string()
        }
    }

    fn restore_window(
        &mut self,
        ctx: &egui::Context,
        game: &GameInfo,
        open: &mut bool,
    ) {
        egui::Window::new("Select Backup to Restore")
            .collapsible(false)
            .show(ctx, |ui| {
                let backups = backup_utils::list_backups(game.app_id());
                if backups.is_empty() {
                    ui.label("No backups found");
                } else {
                    for backup in backups {
                        let label = Self::format_backup_name(&backup);
                        if ui.button(label).clicked() {
                            match backup_utils::restore_prefix(&backup, game.prefix_path()) {
                                Ok(_) => tfd::message_box_ok(
                                    "Restore",
                                    "Prefix restored",
                                    tfd::MessageBoxIcon::Info,
                                ),
                                Err(e) => tfd::message_box_ok(
                                    "Restore failed",
                                    &format!("{}", e),
                                    tfd::MessageBoxIcon::Error,
                                ),
                            };
                            *open = false;
                        }
                    }
                }
                if ui.button("Close").clicked() {
                    *open = false;
                }
            });
    }

    fn delete_window(
        &mut self,
        ctx: &egui::Context,
        game: &GameInfo,
        open: &mut bool,
    ) {
        egui::Window::new("Select Backup to Delete")
            .collapsible(false)
            .show(ctx, |ui| {
                let backups = backup_utils::list_backups(game.app_id());
                if backups.is_empty() {
                    ui.label("No backups found");
                } else {
                    for backup in backups {
                        let label = Self::format_backup_name(&backup);
                        if ui.button(label).clicked() {
                            match backup_utils::delete_backup(&backup) {
                                Ok(_) => tfd::message_box_ok(
                                    "Delete",
                                    "Backup removed",
                                    tfd::MessageBoxIcon::Info,
                                ),
                                Err(e) => tfd::message_box_ok(
                                    "Delete failed",
                                    &format!("{}", e),
                                    tfd::MessageBoxIcon::Error,
                                ),
                            };
                            *open = false;
                        }
                    }
                }
                if ui.button("Close").clicked() {
                    *open = false;
                }
            });
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        restore_dialog_open: &mut bool,
        delete_dialog_open: &mut bool,
        tools: &BTreeMap<String, bool>,
    ) {
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
                                    SystemTime::UNIX_EPOCH + time,
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

                    ui.horizontal(|ui| {
                        if self.prefix_available() {
                            if ui.button("📦 Backup").clicked() {
                                match backup_utils::create_backup(game.prefix_path(), game.app_id()) {
                                    Ok(p) => tfd::message_box_ok(
                                        "Backup",
                                        &format!("Backup created at {}", p.display()),
                                        tfd::MessageBoxIcon::Info,
                                    ),
                                    Err(e) => tfd::message_box_ok(
                                        "Backup failed",
                                        &format!("{}", e),
                                        tfd::MessageBoxIcon::Error,
                                    ),
                                }
                            }

                            if ui.button("🗑 Reset Prefix").clicked() {
                                match backup_utils::reset_prefix(game.prefix_path()) {
                                    Ok(_) => tfd::message_box_ok(
                                        "Reset",
                                        "Prefix deleted",
                                        tfd::MessageBoxIcon::Info,
                                    ),
                                    Err(e) => tfd::message_box_ok(
                                        "Reset failed",
                                        &format!("{}", e),
                                        tfd::MessageBoxIcon::Error,
                                    ),
                                }
                            }

                            if ui.button("🧹 Clear Shader Cache").clicked() {
                                if let Ok(libs) = steam::get_steam_libraries() {
                                    match backup_utils::clear_shader_cache(game.app_id(), &libs) {
                                        Ok(_) => tfd::message_box_ok(
                                            "Shader Cache",
                                            "Shader cache cleared",
                                            tfd::MessageBoxIcon::Info,
                                        ),
                                        Err(e) => tfd::message_box_ok(
                                            "Shader Cache failed",
                                            &format!("{}", e),
                                            tfd::MessageBoxIcon::Error,
                                        ),
                                    }
                                }
                            }

                            let protontricks_btn = ui.add_enabled(
                                *tools.get("protontricks").unwrap_or(&false),
                                egui::Button::new("🔧 Protontricks"),
                            );
                            if protontricks_btn.clicked() {
                                let appid = game.app_id();
                                thread::spawn(move || {
                                    protontricks::execute(appid, &[]);
                                });
                            }
                            if !tools.get("protontricks").unwrap_or(&false) {
                                protontricks_btn.on_hover_text(
                                    "This feature requires `protontricks`. Please install it using your package manager.",
                                );
                            }

                            let winecfg_btn = ui.add_enabled(
                                *tools.get("winecfg").unwrap_or(&false),
                                egui::Button::new("⚙️ winecfg"),
                            );
                            if winecfg_btn.clicked() {
                                let appid = game.app_id();
                                thread::spawn(move || {
                                    winecfg::execute(appid);
                                });
                            }
                            if !tools.get("winecfg").unwrap_or(&false) {
                                winecfg_btn.on_hover_text(
                                    "`winecfg` is not installed or not found in PATH. Please install Wine.",
                                );
                            }
                        }

                        if ui.button("♻️ Restore").clicked() {
                            *restore_dialog_open = true;
                        }

                        if ui.button("🗑 Delete Backup").clicked() {
                            *delete_dialog_open = true;
                        }
                    });
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
                            std::time::UNIX_EPOCH + std::time::Duration::from_secs(last_played),
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
                    let _ = open::that(format!(
                        "https://www.pcgamingwiki.com/api/appid.php?appid={}",
                        game.app_id()
                    ));
                }
            });

            if *restore_dialog_open {
                self.restore_window(ui.ctx(), game, restore_dialog_open);
            }

            if *delete_dialog_open {
                self.delete_window(ui.ctx(), game, delete_dialog_open);
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a game to view details");
            });
        }
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
            let app_manifest = library
                .join("steamapps")
                .join(format!("appmanifest_{}.acf", app_id));
            if app_manifest.exists() {
                if let Ok(contents) = fs::read_to_string(&app_manifest) {
                    // Look for the "installdir" field in the manifest
                    if let Some(path) = contents
                        .lines()
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
