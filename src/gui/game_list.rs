use super::sort::GameSortKey;
use crate::core::models::GameInfo;
use eframe::egui;
use egui_phosphor::regular;

/// Wrapper to display a game list with sorting controls

pub struct GameList<'a> {
    games: &'a [GameInfo],
}

impl<'a> GameList<'a> {
    pub fn new(games: &'a [GameInfo]) -> Self {
        Self { games }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        selected_game: &mut Option<GameInfo>,
        sort_key: &mut GameSortKey,
        descending: &mut bool,
    ) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            ui.heading("Installed Games");

            ui.horizontal(|ui| {
                ui.label("Sort by:");
                let prev = *sort_key;
                egui::ComboBox::from_id_source("sort_combo")
                    .selected_text(sort_key.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(sort_key, GameSortKey::LastPlayed, "Last Played");
                        ui.selectable_value(sort_key, GameSortKey::LastUpdated, "Last Updated");
                        ui.selectable_value(sort_key, GameSortKey::Name, "Name");
                        ui.selectable_value(sort_key, GameSortKey::AppId, "AppID");
                        ui.selectable_value(sort_key, GameSortKey::ProtonVersion, "Proton Version");
                    });
                if *sort_key != prev {
                    changed = true;
                }

                let arrow = if *descending { regular::ARROW_DOWN } else { regular::ARROW_UP };
                if ui.button(arrow).on_hover_text("Toggle order").clicked() {
                    *descending = !*descending;
                    changed = true;
                }
            });

            if self.games.is_empty() {
                ui.label("No games found");
                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for game in self.games {
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
        changed
    }
}
