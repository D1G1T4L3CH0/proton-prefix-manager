use eframe::egui;
use crate::core::models::GameInfo;

const ITEM_HEIGHT: f32 = 24.0; // Height of each game item in pixels
const VISIBLE_ITEMS: usize = 20; // Number of items to render at once

pub struct GameList<'a> {
    games: &'a [GameInfo],
}

impl<'a> GameList<'a> {
    pub fn new(games: &'a [GameInfo]) -> Self {
        Self { games }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, selected_game: &mut Option<GameInfo>) {
        ui.vertical(|ui| {
            ui.heading("Installed Games");
            
            if self.games.is_empty() {
                ui.label("No games found");
                return;
            }
            
            // Sort games alphabetically and case-insensitively
            let mut sorted_games: Vec<&GameInfo> = self.games.iter().collect();
            sorted_games.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));
            
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Render all items
                    for game in sorted_games {
                        let is_selected = selected_game
                            .as_ref()
                            .map_or(false, |g| g.app_id() == game.app_id());
                        
                        let response = ui.selectable_label(is_selected, game.name());
                        
                        if response.clicked() {
                            *selected_game = Some(game.clone());
                        }
                        
                        response.on_hover_text(format!("AppID: {}", game.app_id()));
                    }
                });
        });
    }
} 