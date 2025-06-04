use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::core::models::GameInfo;
use crate::core::steam;
use super::game_list::GameList;
use super::details::GameDetails;

pub struct ProtonPrefixFinderApp {
    loading: bool,
    search_query: String,
    installed_games: Arc<Mutex<Vec<GameInfo>>>,
    filtered_games: Vec<GameInfo>,
    selected_game: Option<GameInfo>,
    search_changed: bool,
    error_message: Option<String>,
    status_message: Option<String>,
    dark_mode: bool,
    load_error: Option<String>,
}

impl Default for ProtonPrefixFinderApp {
    fn default() -> Self {
        Self {
            loading: true,
            search_query: String::new(),
            installed_games: Arc::new(Mutex::new(Vec::new())),
            filtered_games: Vec::new(),
            selected_game: None,
            search_changed: false,
            error_message: None,
            status_message: Some("Loading...".to_string()),
            dark_mode: true,
            load_error: None,
        }
    }
}

impl ProtonPrefixFinderApp {
    pub fn new() -> Self {
        let app = Self::default();
        let games = Arc::clone(&app.installed_games);
        
        thread::spawn(move || {
            match steam::get_steam_libraries() {
                Ok(libraries) => {
                    match steam::load_games_from_libraries(&libraries) {
                        Ok(local_list) => {
                            let mut locked = games.lock().unwrap();
                            *locked = local_list;
                        },
                        Err(e) => {
                            log::error!("Failed to load games: {}", e);
                        }
                    }
                },
                Err(e) => {
                    log::error!("Failed to get Steam libraries: {}", e);
                }
            }
        });
        
        app
    }

    fn search_games(&mut self) {
        let query = self.search_query.to_lowercase();
        if let Ok(locked) = self.installed_games.lock() {
            self.filtered_games = locked
                .iter()
                .filter(|game| game.name().to_lowercase().contains(&query)
                    || game.app_id().to_string().contains(&query))
                .cloned()
                .collect();
            
            self.filtered_games.sort_by(|a, b| a.name().cmp(b.name()));
            
            // Update status message
            if self.filtered_games.is_empty() && !query.is_empty() {
                self.status_message = Some(format!("No games found matching '{}'", query));
            } else if !self.filtered_games.is_empty() {
                self.status_message = Some(format!("Found {} games", self.filtered_games.len()));
            } else {
                self.status_message = None;
            }
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
            visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 160, 160));
            visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 160, 160));
            visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100));
            visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 70));
            
            // Darker text for better contrast
            visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 20, 20));
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 20, 20));
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0));
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0));
            
            // Set the custom visuals
            ctx.set_visuals(visuals);
        }
    }
}

impl eframe::App for ProtonPrefixFinderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        self.apply_theme(ctx);
        
        // Check if loading is complete
        if self.loading {
            if let Ok(games) = self.installed_games.lock() {
                if !games.is_empty() {
                    self.loading = false;
                    self.filtered_games = games.clone();
                    self.status_message = Some(format!("Loaded {} games", self.filtered_games.len()));
                } else if games.is_empty() && self.loading && ctx.input(|i| i.time) > 3.0 {
                    // If after 3 seconds we still have no games, assume there was an error
                    self.loading = false;
                    self.error_message = Some("Failed to load Steam games. Make sure Steam is installed.".to_string());
                }
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
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Proton Prefix Finder");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(if self.dark_mode { "☀" } else { "🌙" }).clicked() {
                        self.toggle_theme(ctx);
                    }
                });
            });
            
            ui.separator();
            
            ui.horizontal(|ui| {
                let search_icon = if self.dark_mode { "🔍 " } else { "🔎 " };
                ui.label(format!("{}Search:", search_icon));
                
                // Create a frame around the search box to make it more visible
                egui::Frame::new()
                    .stroke(egui::Stroke::new(1.0, if self.dark_mode {
                        egui::Color32::from_gray(100)
                    } else {
                        egui::Color32::from_gray(100)
                    }))
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
                    ui.hyperlink_to("GitHub", "https://github.com/yourusername/proton-prefix-finder");
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

            ui.columns(2, |cols| {
                let (left_col, right_col) = cols.split_at_mut(1);
                let left_ui = &mut left_col[0];
                let right_ui = &mut right_col[0];

                // Game list component
                GameList::new(&self.filtered_games)
                    .show(left_ui, &mut self.selected_game);

                // Game details component in a ScrollArea
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .id_source("details_panel")
                    .show(right_ui, |ui| {
                        GameDetails::new(self.selected_game.as_ref())
                            .show(ui);
                    });
            });
        });
    }
} 