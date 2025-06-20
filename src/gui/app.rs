use super::advanced_search::{advanced_search_dialog, AdvancedSearchState};
use super::backup_manager::BackupManagerWindow;
use super::details::{GameConfig, GameDetails, PrefixInfo};
use super::game_list::{compare_games, GameList};
use super::runtime_cleaner::RuntimeCleanerWindow;
use super::SortOption;
use crate::core::models::GameInfo;
use crate::core::steam;
use crate::utils::dependencies::scan_tools;
use crate::utils::prefix_validator::CheckResult;
use crate::utils::terminal;
use eframe::egui;
use eframe::egui::Modal;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use tinyfiledialogs as tfd;

pub struct ProtonPrefixManagerApp {
    loading: bool,
    search_query: String,
    installed_games: Arc<Mutex<Vec<GameInfo>>>,
    filtered_games: Vec<GameInfo>,
    selected_game: Option<GameInfo>,
    last_selected_app_id: Option<u32>,
    search_changed: bool,
    error_message: Option<String>,
    status_message: Option<String>,
    last_status_update: f64,
    dark_mode: bool,
    restore_dialog_open: bool,
    delete_dialog_open: bool,
    validation_dialog_open: bool,
    validation_results: Vec<CheckResult>,
    tool_status: BTreeMap<String, bool>,
    last_tool_scan: f64,
    config_cache: HashMap<u32, GameConfig>,
    prefix_cache: HashMap<u32, PrefixInfo>,
    show_backup_manager: bool,
    backup_manager: BackupManagerWindow,
    show_runtime_cleaner: bool,
    runtime_cleaner: RuntimeCleanerWindow,
    show_advanced_search: bool,
    adv_state: AdvancedSearchState,
    sort_option: SortOption,
    show_repair_dialog: bool,
    repair_rx: Option<Receiver<crate::error::Result<()>>>,
    repair_prefix: Option<PathBuf>,
}

impl Default for ProtonPrefixManagerApp {
    fn default() -> Self {
        Self {
            loading: true,
            search_query: String::new(),
            installed_games: Arc::new(Mutex::new(Vec::new())),
            filtered_games: Vec::new(),
            selected_game: None,
            last_selected_app_id: None,
            search_changed: false,
            error_message: None,
            status_message: Some("Loading...".to_string()),
            last_status_update: 0.0,
            dark_mode: true,
            restore_dialog_open: false,
            delete_dialog_open: false,
            validation_dialog_open: false,
            validation_results: Vec::new(),
            tool_status: {
                let mut map = scan_tools(&["protontricks", "winecfg"]);
                map.insert("terminal".to_string(), terminal::terminal_available());
                map
            },
            last_tool_scan: 0.0,
            config_cache: HashMap::new(),
            prefix_cache: HashMap::new(),
            show_backup_manager: false,
            backup_manager: BackupManagerWindow::new(),
            show_runtime_cleaner: false,
            runtime_cleaner: RuntimeCleanerWindow::new(),
            show_advanced_search: false,
            adv_state: AdvancedSearchState::default(),
            sort_option: SortOption::ModifiedDesc,
            show_repair_dialog: false,
            repair_rx: None,
            repair_prefix: None,
        }
    }
}

impl ProtonPrefixManagerApp {
    pub fn new() -> Self {
        let app = Self::default();
        let games = Arc::clone(&app.installed_games);

        thread::spawn(move || match steam::get_steam_libraries() {
            Ok(libraries) => match steam::load_games_from_libraries(&libraries) {
                Ok(local_list) => {
                    let mut locked = games.lock().unwrap();
                    *locked = local_list;
                }
                Err(e) => {
                    log::error!("Failed to load games: {}", e);
                }
            },
            Err(e) => {
                log::error!("Failed to get Steam libraries: {}", e);
            }
        });

        app
    }

    fn clear_selection_data(&mut self, app_id: Option<u32>) {
        if let Some(id) = app_id {
            self.config_cache.remove(&id);
            self.prefix_cache.remove(&id);
        }
        if app_id.is_none() {
            self.config_cache.clear();
            self.prefix_cache.clear();
        }
        crate::utils::library::clear_manifest_cache();
        crate::utils::user_config::clear_localconfig_cache();
    }

    fn sort_filtered_games(&mut self) {
        let opt = self.sort_option;
        self.filtered_games.sort_by(|a, b| compare_games(a, b, opt));
    }

    fn search_games(&mut self) {
        let query = self.search_query.to_lowercase();
        if let Ok(locked) = self.installed_games.lock() {
            self.filtered_games = locked
                .iter()
                .filter(|game| {
                    game.name().to_lowercase().contains(&query)
                        || game.app_id().to_string().contains(&query)
                })
                .cloned()
                .collect();
        }

        self.sort_filtered_games();

        // Update status message
        if self.filtered_games.is_empty() && !query.is_empty() {
            self.status_message = Some(format!("No games found matching '{}'", query));
        } else if !self.filtered_games.is_empty() {
            self.status_message = Some(format!("Found {} games", self.filtered_games.len()));
        } else {
            self.status_message = None;
        }
        self.search_changed = false;
    }

    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.dark_mode = !self.dark_mode;
        self.apply_theme(ctx);
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            // Create a custom light theme that's much less bright
            let mut visuals = egui::Visuals::light();

            // Use a significantly darker background - more like a "medium" theme than light
            visuals.panel_fill = egui::Color32::from_rgb(210, 210, 210); // Medium gray
            visuals.window_fill = egui::Color32::from_rgb(210, 210, 210);
            visuals.extreme_bg_color = egui::Color32::from_rgb(190, 190, 190); // Darker for contrast

            // Make sure widgets have clear borders and backgrounds
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(200, 200, 200);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(220, 220, 220); // Lighter to stand out
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(230, 230, 230);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(240, 240, 240);

            // Add clear borders to widgets
            visuals.widgets.noninteractive.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 160, 160));
            visuals.widgets.inactive.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 160, 160));
            visuals.widgets.hovered.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100));
            visuals.widgets.active.bg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 70));

            // Darker text for better contrast
            visuals.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 20, 20));
            visuals.widgets.inactive.fg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 20, 20));
            visuals.widgets.hovered.fg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0));
            visuals.widgets.active.fg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0));

            // Set the custom visuals
            ctx.set_visuals(visuals);
        }
    }

    fn start_repair(&mut self, prefix: PathBuf) {
        self.show_repair_dialog = true;
        let p = prefix.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let res = crate::utils::prefix_repair::repair_prefix(&p);
            let _ = tx.send(res);
        });
        self.repair_rx = Some(rx);
        self.repair_prefix = Some(prefix);
    }
}

impl eframe::App for ProtonPrefixManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        self.apply_theme(ctx);

        // Clear status message after a short delay
        let current_time = ctx.input(|i| i.time);
        if self.status_message.is_some() && current_time - self.last_status_update > 5.0 {
            self.status_message = None;
        }

        // Check if loading is complete
        if self.loading {
            if let Ok(games) = self.installed_games.lock() {
                if !games.is_empty() {
                    self.loading = false;
                    self.filtered_games = games.clone();
                    self.status_message =
                        Some(format!("Loaded {} games", self.filtered_games.len()));
                } else if games.is_empty() && self.loading && ctx.input(|i| i.time) > 3.0 {
                    // If after 3 seconds we still have no games, assume there was an error
                    self.loading = false;
                    self.error_message = Some(
                        "Failed to load Steam games. Make sure Steam is installed.".to_string(),
                    );
                }
            }
            if !self.loading {
                self.sort_filtered_games();
            }
        }

        // Show error popup if there's an error
        if let Some(error) = &self.error_message {
            let error_msg = error.clone();
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&error_msg);
                    if ui.button("Close").clicked() {
                        self.error_message = None;
                    }
                });
        }

        if self.search_changed {
            self.search_games();
            self.last_status_update = ctx.input(|i| i.time);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Proton Prefix Manager");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(if self.dark_mode { "☀" } else { "🌙" }).clicked() {
                        self.toggle_theme(ctx);
                    }
                    if ui
                        .button("🔎 Advanced Search")
                        .on_hover_text("Advanced Search")
                        .clicked()
                    {
                        if let Ok(g) = self.installed_games.lock() {
                            self.adv_state.perform_search(&g);
                        }
                        self.show_advanced_search = true;
                    }
                    if ui
                        .button("💾 Manage Backups")
                        .on_hover_text("View and manage backups for all games.")
                        .clicked()
                    {
                        self.show_backup_manager = true;
                    }
                    if ui
                        .button("🧹 Steam Runtime Cleaner")
                        .on_hover_text("Find leftover data to delete.")
                        .clicked()
                    {
                        self.show_runtime_cleaner = true;
                    }
                    if let Some(game) = self.selected_game.as_ref() {
                        let details = GameDetails::new(Some(game));
                        details.prefix_tools_menu(
                            ui,
                            game,
                            &mut self.restore_dialog_open,
                            &mut self.delete_dialog_open,
                            &mut self.validation_dialog_open,
                            &mut self.validation_results,
                            &self.tool_status,
                            &mut self.status_message,
                            &mut self.last_status_update,
                        );
                    }
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                let search_icon = if self.dark_mode { "🔍 " } else { "🔎 " };
                ui.label(format!("{}Search:", search_icon));

                // Create a frame around the search box to make it more visible
                egui::Frame::new()
                    .stroke(egui::Stroke::new(
                        1.0,
                        if self.dark_mode {
                            egui::Color32::from_gray(100)
                        } else {
                            egui::Color32::from_gray(100)
                        },
                    ))
                    .inner_margin(egui::vec2(4.0, 2.0))
                    .show(ui, |ui| {
                        let response = ui.text_edit_singleline(&mut self.search_query);
                        if response.changed() {
                            self.search_changed = true;
                        }
                    });

                if !self.search_query.is_empty() {
                    if ui.button("❌").clicked() {
                        self.search_query.clear();
                        self.search_changed = true;
                    }
                }
            });
        });

        // Status bar at the bottom
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(msg) = &self.status_message {
                    ui.label(msg);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(
                        "GitHub",
                        "https://github.com/D1G1T4L3CH0/proton-prefix-manager",
                    );
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.loading {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("Loading Steam games...");
                });
                return;
            }

            if let Some(error) = &self.error_message {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(error).color(egui::Color32::RED));
                });
                return;
            }

            egui::SidePanel::left("game_list_panel")
                .resizable(true)
                .show(ctx, |ui| {
                    let changed = GameList::new(&self.filtered_games).show(
                        ui,
                        &mut self.selected_game,
                        &mut self.sort_option,
                    );
                    if changed {
                        self.sort_filtered_games();
                    }
                });

            let current_id = self.selected_game.as_ref().map(|g| g.app_id());
            if current_id != self.last_selected_app_id {
                self.clear_selection_data(self.last_selected_app_id);
                self.last_selected_app_id = current_id;
                if let Some(id) = current_id {
                    if let Ok(updated) = steam::refresh_game_info(id) {
                        self.selected_game = Some(updated);
                    }
                    self.config_cache.remove(&id);
                    self.prefix_cache.insert(
                        id,
                        super::details::collect_prefix_info(
                            self.selected_game.as_ref().unwrap().prefix_path(),
                        ),
                    );
                } else {
                    self.clear_selection_data(None);
                }
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .id_salt("details_panel")
                    .show(ui, |ui| {
                        let repair = GameDetails::new(self.selected_game.as_ref()).show(
                            ui,
                            &mut self.restore_dialog_open,
                            &mut self.delete_dialog_open,
                            &mut self.validation_dialog_open,
                            &mut self.validation_results,
                            &mut self.config_cache,
                            &mut self.prefix_cache,
                        );
                        if let Some(p) = repair {
                            self.start_repair(p);
                        }
                    });
            });
        });

        if let Ok(games) = self.installed_games.lock() {
            self.backup_manager
                .show(ctx, &mut self.show_backup_manager, Some(&games));
        } else {
            self.backup_manager
                .show(ctx, &mut self.show_backup_manager, None);
        }

        self.runtime_cleaner
            .show(ctx, &mut self.show_runtime_cleaner);

        if let Ok(games) = self.installed_games.lock() {
            if self.show_advanced_search {
                advanced_search_dialog(
                    ctx,
                    &mut self.adv_state,
                    &mut self.show_advanced_search,
                    &games,
                    &mut self.selected_game,
                );
            }
        }

        if self.show_repair_dialog {
            if let Some(rx) = &self.repair_rx {
                if let Ok(res) = rx.try_recv() {
                    self.show_repair_dialog = false;
                    self.repair_rx = None;
                    if let Some(prefix) = self.repair_prefix.take() {
                        match res {
                            Ok(_) => {
                                self.validation_results =
                                    crate::utils::prefix_validator::validate_prefix(&prefix);
                                tfd::message_box_ok(
                                    "Repair",
                                    "Repair completed",
                                    tfd::MessageBoxIcon::Info,
                                );
                            }
                            Err(e) => {
                                tfd::message_box_ok(
                                    "Repair failed",
                                    &format!("{}", e),
                                    tfd::MessageBoxIcon::Error,
                                );
                            }
                        }
                    }
                }
            }

            let area = Modal::default_area(egui::Id::new("repair_modal"))
                .default_size(egui::vec2(240.0, 80.0));
            Modal::new(egui::Id::new("repair_modal"))
                .area(area)
                .frame(egui::Frame::window(&ctx.style()))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.spinner();
                        ui.label("Repairing prefix...");
                    });
                });
        }

        // Periodically rescan for external tools so disabled buttons can update
        let now = ctx.input(|i| i.time);
        if now - self.last_tool_scan > 5.0 {
            self.tool_status = scan_tools(&["protontricks", "winecfg"]);
            self.tool_status
                .insert("terminal".to_string(), terminal::terminal_available());
            self.last_tool_scan = now;
        }
    }
}
