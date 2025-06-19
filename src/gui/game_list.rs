use crate::core::models::GameInfo;
use eframe::egui;
use std::fs;
use std::time::SystemTime;

/// Available sort options for the game list
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortOption {
    NameAsc,
    NameDesc,
    ModifiedAsc,
    ModifiedDesc,
}

impl SortOption {
    pub fn label(&self) -> &'static str {
        match self {
            SortOption::NameAsc => "Name \u{2191}",
            SortOption::NameDesc => "Name \u{2193}",
            SortOption::ModifiedAsc => "Last Modified \u{2191}",
            SortOption::ModifiedDesc => "Last Modified \u{2193}",
        }
    }
}

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
        sort_option: &mut SortOption,
    ) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            ui.heading("Installed Games");

            ui.horizontal(|ui| {
                ui.label("Sort by:");
                let prev = *sort_option;
                egui::ComboBox::from_id_source("sort_combo")
                    .selected_text(sort_option.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(sort_option, SortOption::ModifiedDesc, SortOption::ModifiedDesc.label());
                        ui.selectable_value(sort_option, SortOption::ModifiedAsc, SortOption::ModifiedAsc.label());
                        ui.selectable_value(sort_option, SortOption::NameAsc, SortOption::NameAsc.label());
                        ui.selectable_value(sort_option, SortOption::NameDesc, SortOption::NameDesc.label());
                    });
                if *sort_option != prev {
                    changed = true;
                }
            });

            if self.games.is_empty() {
                ui.label("No games found");
                return;
            }

            // Prepare sorted games based on the selected option
            let mut sorted_games: Vec<&GameInfo> = self.games.iter().collect();
            match sort_option {
                SortOption::NameAsc => {
                    sorted_games.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));
                }
                SortOption::NameDesc => {
                    sorted_games.sort_by(|a, b| b.name().to_lowercase().cmp(&a.name().to_lowercase()));
                }
                SortOption::ModifiedAsc | SortOption::ModifiedDesc => {
                    let mut with_time: Vec<(&GameInfo, SystemTime)> = sorted_games
                        .iter()
                        .map(|g| {
                            let time = fs::metadata(g.prefix_path())
                                .and_then(|m| m.modified())
                                .unwrap_or(SystemTime::UNIX_EPOCH);
                            (*g, time)
                        })
                        .collect();
                    with_time.sort_by_key(|(_, t)| *t);
                    if matches!(sort_option, SortOption::ModifiedDesc) {
                        with_time.reverse();
                    }
                    sorted_games = with_time.into_iter().map(|(g, _)| g).collect();
                }
            }

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
        changed
    }
}
