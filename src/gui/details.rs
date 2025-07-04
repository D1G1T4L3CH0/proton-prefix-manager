use crate::cli::{protontricks, winecfg};
use crate::core::models::GameInfo;
use crate::core::steam;
use crate::utils::backup as backup_utils;
use crate::utils::steam_paths;
use crate::utils::terminal;
use crate::utils::user_config;
use crate::utils::{library, manifest as manifest_utils};
use eframe::egui;
use eframe::egui::Modal;
use egui::menu;
use egui_phosphor::regular;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tinyfiledialogs as tfd;

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

#[derive(Clone, Default)]
pub struct PrefixInfo {
    pub version: Option<String>,
    pub has_dxvk: bool,
    pub has_vkd3d: bool,
}

#[derive(Debug)]
pub enum Action {
    Backup { app_id: u32, prefix: PathBuf },
    Restore { backup: PathBuf, prefix: PathBuf },
    DeleteBackup { backup: PathBuf },
    Reset { prefix: PathBuf },
}

impl<'a> GameDetails<'a> {
    pub fn new(game: Option<&'a GameInfo>) -> Self {
        Self {
            game,
            id: egui::Id::new("game_details"),
        }
    }

    fn show_path(&mut self, ui: &mut egui::Ui, label: &str, path: &Path) {
        let path_str = path.display().to_string();
        let copy_id = self.id.with("copy").with(&path_str);
        let current_time = ui.input(|i| i.time);
        let copy_time = ui.data_mut(|d| d.get_temp::<f64>(copy_id).unwrap_or(0.0));
        let is_copied = copy_time > current_time;

        ui.horizontal(|ui| {
            ui.strong(label);

            // Copy button
            let copy_text = if is_copied { format!("{} Copied", regular::CHECK) } else { format!("{} Copy", regular::COPY) };
            let copy_button = ui.button(copy_text);
            if copy_button.clicked() {
                ui.ctx().copy_text(path_str.clone());
                ui.data_mut(|d| d.insert_temp(copy_id, current_time + 2.0));
                ui.ctx().request_repaint();
            }
            copy_button.on_hover_text(format!("Copy path: {}", path_str));

            // Open folder button
            let open_button = ui.button(format!("{} Open", regular::FOLDER_OPEN));
            if open_button.clicked() {
                let _ = open::that(path);
            }
            open_button.on_hover_text(format!("Open: {}", path_str));

            // Terminal button
            let term_button = ui.add_enabled(
                terminal::terminal_available(),
                egui::Button::new(format!("{} Terminal", regular::TERMINAL)),
            );
            if term_button.clicked() {
                if let Err(e) = terminal::open_terminal(path) {
                    eprintln!("Failed to open terminal: {}", e);
                }
            }
            term_button.on_hover_text(format!("Open terminal at: {}", path_str));
        });

        ui.add(egui::Label::new(
            egui::RichText::new(path_str).small().monospace(),
        ));
        ui.add_space(4.0);
    }

    fn game_title_bar(&self, ui: &mut egui::Ui, game: &GameInfo) {
        ui.horizontal(|ui| {
            ui.heading(game.name());
            ui.separator();
            ui.label(format!("App ID: {}", game.app_id()));
        });
        ui.add_space(8.0);
    }

    pub fn prefix_tools_menu(
        &self,
        ui: &mut egui::Ui,
        game: &GameInfo,
        restore_dialog_open: &mut bool,
        delete_dialog_open: &mut bool,
        tools: &BTreeMap<String, bool>,
        status_message: &mut Option<String>,
        status_time: &mut f64,
    ) -> Option<Action> {
        let mut action = None;
        menu::menu_button(ui, &format!("{} Prefix Tools ▾", regular::WRENCH), |ui| {
            ui.menu_button("Prefix ▾", |ui| {
                if ui.button("Backup").clicked() {
                    action = Some(Action::Backup {
                        app_id: game.app_id(),
                        prefix: game.prefix_path().to_path_buf(),
                    });
                    ui.close_menu();
                }
                if ui.button("Restore").clicked() {
                    *restore_dialog_open = true;
                    ui.close_menu();
                }
                if ui.button("Delete Backup").clicked() {
                    *delete_dialog_open = true;
                    ui.close_menu();
                }
                if ui.button("Reset").clicked() {
                    if tfd::message_box_yes_no(
                        "Confirm Reset",
                        "Resetting will delete the prefix. It's prudent to create a backup of your important data or configuration files before performing any critical actions. This ensures you can restore your system to a known good state if something unexpected happens. Continue?",
                        tfd::MessageBoxIcon::Warning,
                        tfd::YesNo::No,
                    ) == tfd::YesNo::Yes
                    {
                        action = Some(Action::Reset { prefix: game.prefix_path().to_path_buf() });
                    }
                    ui.close_menu();
                }
            });

            ui.menu_button("Troubleshooting ▾", |ui| {
                if ui
                    .add_enabled(
                        *tools.get("winecfg").unwrap_or(&false),
                        egui::Button::new("Launch winecfg"),
                    )
                    .clicked()
                {
                    let appid = game.app_id();
                    *status_message = Some("Launching winecfg...".to_string());
                    *status_time = ui.input(|i| i.time);
                    thread::spawn(move || {
                        winecfg::execute(appid);
                    });
                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        *tools.get("protontricks").unwrap_or(&false),
                        egui::Button::new("Launch protontricks"),
                    )
                    .clicked()
                {
                    let appid = game.app_id();
                    *status_message = Some("Launching protontricks...".to_string());
                    *status_time = ui.input(|i| i.time);
                    thread::spawn(move || {
                        protontricks::execute(appid, &[]);
                    });
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
            });
        })
        .response
        .on_hover_text("Tools for managing this game's Proton prefix");
        action
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

    fn load_game_config(app_id: u32) -> io::Result<GameConfig> {
        let libraries = steam::get_steam_libraries()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        for lib in libraries {
            let manifest = lib
                .steamapps_path()
                .join(format!("appmanifest_{}.acf", app_id));
            if manifest.exists() {
                let contents = library::read_manifest_cached(&manifest).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "failed to read manifest")
                })?;
                let proton = manifest_utils::get_value(&contents, "CompatToolOverride");
                let launch = user_config::get_launch_options(app_id)
                    .or_else(|| manifest_utils::get_value(&contents, "LaunchOptions"))
                    .unwrap_or_default();
                let cloud = manifest_utils::get_value(&contents, "AllowCloudSaves")
                    .unwrap_or_else(|| "1".to_string())
                    == "1";
                let auto = manifest_utils::get_value(&contents, "AutoUpdateBehavior")
                    .unwrap_or_else(|| "0".to_string())
                    == "0";
                return Ok(GameConfig {
                    proton,
                    launch_options: launch,
                    cloud_sync: cloud,
                    auto_update: auto,
                });
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "manifest not found",
        ))
    }

    fn save_game_config(app_id: u32, cfg: &GameConfig) -> io::Result<()> {
        let libraries = steam::get_steam_libraries()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        for lib in libraries {
            let manifest = lib
                .steamapps_path()
                .join(format!("appmanifest_{}.acf", app_id));
            if manifest.exists() {
                let mut contents = library::read_manifest_cached(&manifest).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "failed to read manifest")
                })?;
                contents = manifest_utils::update_or_insert(
                    &contents,
                    "LaunchOptions",
                    &cfg.launch_options,
                );
                user_config::set_launch_options(app_id, &cfg.launch_options)?;
                if let Some(p) = &cfg.proton {
                    contents = manifest_utils::update_or_insert(&contents, "CompatToolOverride", p);
                    user_config::set_compat_tool(app_id, p)?;
                } else {
                    let _ = user_config::clear_compat_tool(app_id);
                }
                let cloud_val = if cfg.cloud_sync { "1" } else { "0" };
                contents =
                    manifest_utils::update_or_insert(&contents, "AllowCloudSaves", cloud_val);
                let auto_val = if cfg.auto_update { "0" } else { "1" };
                contents =
                    manifest_utils::update_or_insert(&contents, "AutoUpdateBehavior", auto_val);
                fs::write(&manifest, contents.as_bytes())?;
                library::update_manifest_cache(&manifest, &contents);
                return Ok(());
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "manifest not found",
        ))
    }

    fn list_proton_versions() -> Vec<String> {
        use once_cell::sync::OnceCell;
        static CACHE: OnceCell<Vec<String>> = OnceCell::new();
        if let Some(v) = CACHE.get() {
            return v.clone();
        }

        let mut versions = Vec::new();
        if let Ok(libraries) = steam::get_steam_libraries() {
            for lib in libraries {
                let common = lib.join("steamapps/common");
                if let Ok(entries) = fs::read_dir(&common) {
                    for e in entries.flatten() {
                        if let Ok(name) = e.file_name().into_string() {
                            if name.to_lowercase().contains("proton") {
                                versions.push(name);
                            }
                        }
                    }
                }
            }
        }

        for dir in steam_paths::compatibilitytools_dirs() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for e in entries.flatten() {
                    if e.path().is_dir() {
                        if let Ok(name) = e.file_name().into_string() {
                            versions.push(name);
                        }
                    }
                }
            }
        }

        versions.sort();
        versions.dedup();
        let _ = CACHE.set(versions.clone());
        versions
    }

    fn restore_window(
        &mut self,
        ctx: &egui::Context,
        game: &GameInfo,
        open: &mut bool,
    ) -> Option<Action> {
        let mut action = None;
        if !*open {
            return action;
        }

        let mut should_close = false;
        let response = Modal::new(egui::Id::new("restore_modal"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Select Backup to Restore");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            should_close = true;
                        }
                    });
                });
                ui.separator();
                let backups = backup_utils::list_backups(game.app_id());
                if backups.is_empty() {
                    ui.label("No backups found");
                } else {
                    for backup in backups {
                        let label = backup_utils::format_backup_name(&backup);
                        if ui.button(label).clicked() {
                            action = Some(Action::Restore {
                                backup: backup.clone(),
                                prefix: game.prefix_path().to_path_buf(),
                            });
                            should_close = true;
                        }
                    }
                }
            });

        if response.should_close() || should_close {
            *open = false;
        }
        action
    }

    fn delete_window(
        &mut self,
        ctx: &egui::Context,
        game: &GameInfo,
        open: &mut bool,
    ) -> Option<Action> {
        let mut action = None;
        if !*open {
            return action;
        }

        let mut should_close = false;
        let response = Modal::new(egui::Id::new("delete_modal"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Select Backup to Delete");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            should_close = true;
                        }
                    });
                });
                ui.separator();
                let backups = backup_utils::list_backups(game.app_id());
                if backups.is_empty() {
                    ui.label("No backups found");
                } else {
                    for backup in backups {
                        let label = backup_utils::format_backup_name(&backup);
                        if ui.button(label).clicked() {
                            action = Some(Action::DeleteBackup {
                                backup: backup.clone(),
                            });
                            should_close = true;
                        }
                    }
                }
            });

        if response.should_close() || should_close {
            *open = false;
        }
        action
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        restore_dialog_open: &mut bool,
        delete_dialog_open: &mut bool,
        configs: &mut HashMap<u32, GameConfig>,
        info_cache: &mut HashMap<u32, PrefixInfo>,
    ) -> Option<Action> {
        let mut repair_request = None;
        if let Some(game) = self.game {
            self.game_title_bar(ui, game);

            // Prefix Information
            egui::CollapsingHeader::new("Prefix Information")
                .default_open(true)
                .show(ui, |ui| {
                    if self.prefix_available() {
                        self.show_path(ui, "Prefix Path:", game.prefix_path());

                        let modified = game.modified();
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

                        let drive_c = game.prefix_path().join("pfx/drive_c");
                        if drive_c.exists() {
                            self.show_path(ui, "Drive C:", &drive_c);
                        }
                    } else {
                        ui.label("No prefix currently exists for this game.");
                    }

                    // Tools moved to the top toolbar
                });

            // Proton Information
            egui::CollapsingHeader::new(format!("{} Proton Information", regular::ROCKET))
                .default_open(true)
                .show(ui, |ui| {
                    let info = info_cache
                        .entry(game.app_id())
                        .or_insert_with(|| collect_prefix_info(game.prefix_path()));
                    if let Some(version) = &info.version {
                        ui.horizontal(|ui| {
                            ui.label("Version:");
                            ui.monospace(version);
                        });
                    } else {
                        ui.label("Proton version could not be detected");
                    }

                    if info.has_dxvk {
                        ui.label(format!("{} DXVK is enabled", regular::CHECK));
                    }
                    if info.has_vkd3d {
                        ui.label(format!("{} VKD3D is enabled", regular::CHECK));
                    }
                });

            // Game Details
            egui::CollapsingHeader::new(format!("{} Game Details", regular::GAME_CONTROLLER))
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(if game.has_manifest() {
                        format!("{} Game has a manifest file", regular::CHECK)
                    } else {
                        format!("{} No manifest file found", regular::X)
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

                    if let Some(user_dir) = steam::find_userdata_dir(game.app_id()) {
                        self.show_path(ui, "Userdata Directory:", &user_dir);
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
            let header_label = if has_custom {
                format!("{} Game Settings *", regular::GEAR)
            } else {
                format!("{} Game Settings", regular::GEAR)
            };
            egui::CollapsingHeader::new(header_label)
                .id_salt("game_settings_header")
                .default_open(has_custom)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Proton Version:");
                        let versions = Self::list_proton_versions();
                        egui::ComboBox::from_id_salt("proton_version")
                            .selected_text(
                                cfg.proton.clone().unwrap_or_else(|| "Default".to_string()),
                            )
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
                                .id_salt("launch_options")
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
                if ui.button(format!("{} SteamDB", regular::LINK)).clicked() {
                    let _ = open::that(format!("https://steamdb.info/app/{}/", game.app_id()));
                }

                if ui.button(format!("{} ProtonDB", regular::GAME_CONTROLLER)).clicked() {
                    let _ = open::that(format!("https://www.protondb.com/app/{}", game.app_id()));
                }

                if ui.button(format!("{} PCGamingWiki", regular::BOOKS)).clicked() {
                    let _ = open::that(format!(
                        "https://www.pcgamingwiki.com/api/appid.php?appid={}",
                        game.app_id()
                    ));
                }
            });

            if *restore_dialog_open {
                if let Some(act) = self.restore_window(ui.ctx(), game, restore_dialog_open) {
                    repair_request = Some(act);
                }
            }

            if *delete_dialog_open {
                if let Some(act) = self.delete_window(ui.ctx(), game, delete_dialog_open) {
                    repair_request = Some(act);
                }
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a game to view details");
            });
        }
        repair_request
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

pub fn collect_prefix_info(prefix_path: &Path) -> PrefixInfo {
    PrefixInfo {
        version: detect_proton_version(prefix_path),
        has_dxvk: has_dxvk(prefix_path),
        has_vkd3d: has_vkd3d(prefix_path),
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
