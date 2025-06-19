use crate::core::models::GameInfo;
use eframe::egui;
use std::cmp::Ordering;

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

pub(super) fn compare_games(a: &GameInfo, b: &GameInfo, sort: SortOption) -> Ordering {
    match sort {
        SortOption::NameAsc => a
            .name()
            .to_lowercase()
            .cmp(&b.name().to_lowercase()),
        SortOption::NameDesc => b
            .name()
            .to_lowercase()
            .cmp(&a.name().to_lowercase()),
        SortOption::ModifiedAsc | SortOption::ModifiedDesc => {
            let ta = a.modified();
            let tb = b.modified();
            if matches!(sort, SortOption::ModifiedAsc) {
                ta.cmp(&tb)
            } else {
                tb.cmp(&ta)
            }
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
