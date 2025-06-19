use crate::core::models::GameInfo;
use eframe::egui;

#[derive(Clone, PartialEq, Eq)]
pub enum SortKey {
    LastPlayed,
    Name,
    AppId,
}

impl Default for SortKey {
    fn default() -> Self {
        SortKey::LastPlayed
    }
}

#[derive(Default, Clone)]
pub struct AdvancedSearchState {
    pub query: String,
    pub has_manifest: bool,
    pub has_prefix: bool,
    pub sort_key: SortKey,
    pub descending: bool,
    #[allow(dead_code)]
    last_update: f64,
    pub results: Vec<GameInfo>,
}

impl AdvancedSearchState {
    pub fn perform_search(&mut self, games: &[GameInfo]) {
        let q = self.query.to_lowercase();
        self.results = games
            .iter()
            .filter(|g| {
                (q.is_empty()
                    || g.name().to_lowercase().contains(&q)
                    || g.app_id().to_string().contains(&q)
                    || g.prefix_path()
                        .display()
                        .to_string()
                        .to_lowercase()
                        .contains(&q))
                    && (!self.has_manifest || g.has_manifest())
                    && (!self.has_prefix || g.prefix_path().exists())
            })
            .cloned()
            .collect();

        self.results.sort_by(|a, b| match self.sort_key {
            SortKey::LastPlayed => a.last_played().cmp(&b.last_played()),
            SortKey::Name => a.name().cmp(b.name()),
            SortKey::AppId => a.app_id().cmp(&b.app_id()),
        });
        if self.descending {
            self.results.reverse();
        }
    }
}

pub fn advanced_search_dialog(
    ctx: &egui::Context,
    state: &mut AdvancedSearchState,
    open: &mut bool,
    games: &[GameInfo],
    selected: &mut Option<GameInfo>,
) {
    let mut window_open = *open;
    let mut close_window = false;
    egui::Window::new("Advanced Search")
        .open(&mut window_open)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");
                let resp = ui.text_edit_singleline(&mut state.query);
                if resp.changed() {
                    state.perform_search(games);
                }
            });
            ui.separator();
            let mut changed = false;
            changed |= ui
                .checkbox(&mut state.has_manifest, "Has manifest")
                .changed();
            changed |= ui.checkbox(&mut state.has_prefix, "Has prefix").changed();
            ui.separator();
            egui::ComboBox::from_label("Sort By")
                .selected_text(match state.sort_key {
                    SortKey::LastPlayed => "Last Modified",
                    SortKey::Name => "Name",
                    SortKey::AppId => "AppID",
                })
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(&mut state.sort_key, SortKey::LastPlayed, "Last Modified")
                        .changed();
                    changed |= ui
                        .selectable_value(&mut state.sort_key, SortKey::Name, "Name")
                        .changed();
                    changed |= ui
                        .selectable_value(&mut state.sort_key, SortKey::AppId, "AppID")
                        .changed();
                });
            changed |= ui.checkbox(&mut state.descending, "Descending").changed();
            if ui.button("Clear Previous Search").clicked() {
                *state = AdvancedSearchState::default();
                state.perform_search(games);
            }
            ui.separator();
            if changed {
                state.perform_search(games);
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                for game in &state.results {
                    if ui
                        .button(format!("{} ({})", game.name(), game.app_id()))
                        .clicked()
                    {
                        *selected = Some(game.clone());
                        close_window = true;
                    }
                }
            });
        });
    if close_window {
        window_open = false;
    }
    *open = window_open;
}
