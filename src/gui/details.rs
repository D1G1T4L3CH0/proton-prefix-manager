use crate::core::models::GameInfo;
use crate::core::steam;
use crate::utils::backup as backup_utils;
use eframe::egui;
use std::thread;
use crate::cli::{protontricks, winecfg};
use crate::utils::terminal;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io;
use crate::utils::manifest as manifest_utils;
use tinyfiledialogs as tfd;
use chrono::NaiveDateTime;
use egui::menu;

pub struct GameDetails<'a> {
    game: Option<&'a GameInfo>,
    id: egui::Id, // Add a unique ID for this instance
}

#[derive(Clone, Default)]
pub struct GameConfig {
    proton: Option<String>,
    launch_options: String,
    auto_update: bool,
    cloud_sync: bool,
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

    fn game_title_bar(&self, ui: &mut egui::Ui, game: &GameInfo) {
        ui.horizontal(|ui| {
            ui.heading(game.name());
            ui.separator();
            ui.label(format!("App ID: {}", game.app_id()));
        });
        ui.add_space(8.0);
    }

    fn prefix_tools_menu(
        &self,
        ui: &mut egui::Ui,
        game: &GameInfo,
        restore_dialog_open: &mut bool,
        delete_dialog_open: &mut bool,
        tools: &BTreeMap<String, bool>,
    ) {
        menu::menu_button(ui, "Prefix Tools ▾", |ui| {
            if ui.button("Backup Prefix").clicked() {
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
                ui.close_menu();
            }
            if ui.button("Restore Backup").clicked() {
                *restore_dialog_open = true;
                ui.close_menu();
            }
            if ui.button("Delete Backup").clicked() {
                *delete_dialog_open = true;
                ui.close_menu();
            }
            if ui.button("Reset Prefix").clicked() {
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
                ui.close_menu();
            }
            if ui.button("Clear Shader Cache").clicked() {
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
                ui.close_menu();
            }
            if ui.add_enabled(*tools.get("terminal").unwrap_or(&false), egui::Button::new("Open Terminal")).clicked() {
                let path = game.prefix_path().to_path_buf();
                thread::spawn(move || {
                    if let Err(e) = terminal::open_terminal(&path) {
                        eprintln!("Failed to open terminal: {}", e);
                    }
                });
                ui.close_menu();
            }
            if ui.button("Open Prefix Folder").clicked() {
                let _ = open::that(game.prefix_path());
                ui.close_menu();
            }
            if ui.add_enabled(*tools.get("winecfg").unwrap_or(&false), egui::Button::new("Launch winecfg")).clicked() {
                let appid = game.app_id();
                thread::spawn(move || {
                    winecfg::execute(appid);
                });
                ui.close_menu();
            }
            if ui.add_enabled(*tools.get("protontricks").unwrap_or(&false), egui::Button::new("Launch protontricks")).clicked() {
                let appid = game.app_id();
                thread::spawn(move || {
                    protontricks::execute(appid, &[]);
                });
                ui.close_menu();
            }
        })
        .response
        .on_hover_text("Tools for managing this game's Proton prefix");
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

    fn load_game_config(app_id: u32) -> io::Result<GameConfig> {
        let libraries = steam::get_steam_libraries()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        for lib in libraries {
            let manifest = lib
                .steamapps_path()
                .join(format!("appmanifest_{}.acf", app_id));
            if manifest.exists() {
                let contents = fs::read_to_string(&manifest)?;
                let proton = manifest_utils::get_value(&contents, "CompatToolOverride");
                let launch = manifest_utils::get_value(&contents, "LaunchOptions").unwrap_or_default();
                let cloud = manifest_utils::get_value(&contents, "AllowCloudSaves").unwrap_or_else(|| "1".to_string()) == "1";
                let auto = manifest_utils::get_value(&contents, "AutoUpdateBehavior").unwrap_or_else(|| "0".to_string()) == "0";
                return Ok(GameConfig {
                    proton,
                    launch_options: launch,
                    cloud_sync: cloud,
                    auto_update: auto,
                });
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "manifest not found"))
    }

    fn save_game_config(app_id: u32, cfg: &GameConfig) -> io::Result<()> {
        let libraries = steam::get_steam_libraries()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        for lib in libraries {
            let manifest = lib
                .steamapps_path()
                .join(format!("appmanifest_{}.acf", app_id));
            if manifest.exists() {
                let mut contents = fs::read_to_string(&manifest)?;
                contents = manifest_utils::update_or_insert(&contents, "LaunchOptions", &cfg.launch_options);
                if let Some(p) = &cfg.proton {
                    contents = manifest_utils::update_or_insert(&contents, "CompatToolOverride", p);
                }
                let cloud_val = if cfg.cloud_sync { "1" } else { "0" };
                contents = manifest_utils::update_or_insert(&contents, "AllowCloudSaves", cloud_val);
                let auto_val = if cfg.auto_update { "0" } else { "1" };
                contents = manifest_utils::update_or_insert(&contents, "AutoUpdateBehavior", auto_val);
                fs::write(&manifest, contents)?;
                return Ok(());
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "manifest not found"))
    }

    fn list_proton_versions() -> Vec<String> {
        let mut versions = Vec::new();
        if let Ok(libraries) = steam::get_steam_libraries() {
            for lib in libraries {
                let common = lib.join("steamapps/common");
                if let Ok(entries) = fs::read_dir(&common) {
                    for e in entries.flatten() {
                        if let Ok(name) = e.file_name().into_string() {
                            if name.to_lowercase().starts_with("proton") {
                                versions.push(name);
                            }
                        }
                    }
                }
            }
        }
        versions.sort();
        versions.dedup();
        versions
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
        configs: &mut HashMap<u32, GameConfig>,
    ) {
        if let Some(game) = self.game {
            self.game_title_bar(ui, game);

            // Prefix Information
            egui::CollapsingHeader::new("Prefix Information")
                .default_open(true)
                .show(ui, |ui| {
                    if self.prefix_available() {
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
                                            ui.monospace(
                                                datetime.format("%Y-%m-%d %H:%M").to_string(),
                                            );
                                            ui.end_row();
                                        });
                                }
                            }
                        }

                        let drive_c = game.prefix_path().join("pfx/drive_c");
                        if drive_c.exists() {
                            self.show_path(ui, "Drive C:", &drive_c);
                        }
                    } else {
                        ui.label("No prefix currently exists for this game.");
                    }

                    ui.horizontal(|ui| {
                        self.prefix_tools_menu(ui, game, restore_dialog_open, delete_dialog_open, tools);
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
                    if has_vkd3d(game.prefix_path()) {
                        ui.label("✓ VKD3D is enabled");
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

            // Game Settings section
            let cfg = configs
                .entry(game.app_id())
                .or_insert_with(|| Self::load_game_config(game.app_id()).unwrap_or_default());
            let has_custom = !cfg.launch_options.is_empty()
                || cfg.proton.is_some()
                || !cfg.auto_update
                || !cfg.cloud_sync;
            let header_label = if has_custom { "⚙ Game Settings *" } else { "⚙ Game Settings" };
            egui::CollapsingHeader::new(header_label)
                .default_open(has_custom)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Proton Version:");
                        let versions = Self::list_proton_versions();
                        egui::ComboBox::from_id_source("proton_version")
                            .selected_text(cfg.proton.clone().unwrap_or_else(|| "Default".to_string()))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut cfg.proton, None, "Default");
                                for v in versions {
                                    ui.selectable_value(&mut cfg.proton, Some(v.clone()), v);
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Launch Options:");
                        ui.add(
                            egui::TextEdit::singleline(&mut cfg.launch_options)
                                .id_source("launch_options")
                                .hint_text("e.g. PROTON_LOG=1"),
                        );
                    });
                    ui.horizontal(|ui| {
                        let lbl = ui.checkbox(&mut cfg.auto_update, "Enable auto-update");
                        lbl.on_hover_text("Toggle automatic updates for this game");
                    });
                    ui.horizontal(|ui| {
                        let lbl = ui.checkbox(&mut cfg.cloud_sync, "Enable Steam Cloud");
                        lbl.on_hover_text("Sync save data via Steam Cloud");
                    });
                    if ui.button("Save").clicked() {
                        match Self::save_game_config(game.app_id(), cfg) {
                            Ok(_) => tfd::message_box_ok(
                                "Config",
                                "Settings saved",
                                tfd::MessageBoxIcon::Info,
                            ),
                            Err(e) => tfd::message_box_ok(
                                "Save failed",
                                &format!("{}", e),
                                tfd::MessageBoxIcon::Error,
                            ),
                        };
                    }
                })
                .header_response
                .on_hover_text("Manage game specific options stored in appmanifest");

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

fn has_vkd3d(prefix_path: &Path) -> bool {
    let dll_path = prefix_path.join("pfx/drive_c/windows/system32");
    dll_path.join("d3d12.dll").exists()
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
