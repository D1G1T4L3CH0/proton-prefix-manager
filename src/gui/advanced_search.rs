use crate::core::models::GameInfo;
use crate::core::steam;
use crate::utils::{manifest as manifest_utils, user_config};
use eframe::egui;
use eframe::egui::Modal;
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    LastPlayed,
    Name,
    AppId,
    ProtonVersion,
}

impl Default for SortKey {
    fn default() -> Self {
        SortKey::LastPlayed
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    Any,
    Has,
    Missing,
}

impl Default for TriState {
    fn default() -> Self {
        TriState::Any
    }
}

impl TriState {
    fn matches(&self, value: bool) -> bool {
        match self {
            TriState::Any => true,
            TriState::Has => value,
            TriState::Missing => !value,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            TriState::Any => "Any",
            TriState::Has => "Has",
            TriState::Missing => "Missing",
        }
    }
}

#[derive(Clone)]
struct ConfigFlags {
    auto_update: bool,
    cloud_sync: bool,
    custom_launch: bool,
    custom_proton: bool,
    proton: Option<String>,
}

fn tri_state_combo(ui: &mut egui::Ui, label: &str, state: &mut TriState) -> bool {
    let mut changed = false;
    egui::ComboBox::from_label(label)
        .selected_text(state.label())
        .show_ui(ui, |ui| {
            changed |= ui.selectable_value(state, TriState::Any, "Any").changed();
            changed |= ui.selectable_value(state, TriState::Has, "Has").changed();
            changed |= ui
                .selectable_value(state, TriState::Missing, "Missing")
                .changed();
        });
    changed
}
#[derive(Clone)]
pub struct AdvancedSearchState {
    pub query: String,
    pub has_manifest: TriState,
    pub has_prefix: TriState,
    pub auto_update: TriState,
    pub cloud_sync: TriState,
    pub custom_launch: TriState,
    pub custom_proton: TriState,
    pub sort_key: SortKey,
    pub descending: bool,
    #[allow(dead_code)]
    last_update: f64,
    pub results: Vec<GameInfo>,
    config_cache: HashMap<u32, ConfigFlags>,
}

impl Default for AdvancedSearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            has_manifest: TriState::Any,
            has_prefix: TriState::Any,
            auto_update: TriState::Any,
            cloud_sync: TriState::Any,
            custom_launch: TriState::Any,
            custom_proton: TriState::Any,
            sort_key: SortKey::default(),
            descending: false,
            last_update: 0.0,
            results: Vec::new(),
            config_cache: HashMap::new(),
        }
    }
}

impl AdvancedSearchState {
    fn load_flags(&mut self, app_id: u32) -> Option<ConfigFlags> {
        if let Some(c) = self.config_cache.get(&app_id) {
            return Some(c.clone());
        }
        let libs = steam::get_steam_libraries().ok()?;
        for lib in libs {
            let manifest = lib
                .steamapps_path()
                .join(format!("appmanifest_{}.acf", app_id));
            if manifest.exists() {
                if let Ok(contents) = fs::read_to_string(&manifest) {
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
                    let flags = ConfigFlags {
                        auto_update: auto,
                        cloud_sync: cloud,
                        custom_launch: !launch.is_empty(),
                        custom_proton: proton.is_some(),
                        proton,
                    };
                    self.config_cache.insert(app_id, flags.clone());
                    return Some(flags);
                }
            }
        }
        None
    }

    pub fn perform_search(&mut self, games: &[GameInfo]) {
        let q = self.query.to_lowercase();
        let require_flags = self.sort_key == SortKey::ProtonVersion
            || self.auto_update != TriState::Any
            || self.cloud_sync != TriState::Any
            || self.custom_launch != TriState::Any
            || self.custom_proton != TriState::Any;
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
                    && self.has_manifest.matches(g.has_manifest())
                    && self.has_prefix.matches(g.prefix_path().exists())
                    && {
                        if !require_flags {
                            true
                        } else if let Some(f) = self.load_flags(g.app_id()) {
                            self.auto_update.matches(f.auto_update)
                                && self.cloud_sync.matches(f.cloud_sync)
                                && self.custom_launch.matches(f.custom_launch)
                                && self.custom_proton.matches(f.custom_proton)
                        } else {
                            self.auto_update == TriState::Any
                                && self.cloud_sync == TriState::Any
                                && self.custom_launch == TriState::Any
                                && self.custom_proton == TriState::Any
                        }
                    }
            })
            .cloned()
            .collect();

        if self.sort_key == SortKey::ProtonVersion && require_flags {
            let ids: Vec<u32> = self.results.iter().map(|g| g.app_id()).collect();
            for id in ids {
                let _ = self.load_flags(id);
            }
        }

        let sort_key = self.sort_key;
        self.results.sort_by(|a, b| match sort_key {
            SortKey::LastPlayed => a.last_played().cmp(&b.last_played()),
            SortKey::Name => a.name().cmp(b.name()),
            SortKey::AppId => a.app_id().cmp(&b.app_id()),
            SortKey::ProtonVersion => {
                let pa = self
                    .config_cache
                    .get(&a.app_id())
                    .and_then(|f| f.proton.clone())
                    .unwrap_or_default();
                let pb = self
                    .config_cache
                    .get(&b.app_id())
                    .and_then(|f| f.proton.clone())
                    .unwrap_or_default();
                pa.cmp(&pb)
            }
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
    if !*open {
        return;
    }

    let mut should_close = false;
    let mut close_window = false;

    let response = Modal::new(egui::Id::new("advanced_search"))
        .frame(egui::Frame::window(&ctx.style()))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Advanced Search");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            });
            ui.separator();
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
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

                columns[1].vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        let resp = ui.text_edit_singleline(&mut state.query);
                        if resp.changed() {
                            state.perform_search(games);
                        }
                    });
                    ui.separator();
                    let mut changed = false;
                    changed |= tri_state_combo(ui, "Has manifest", &mut state.has_manifest);
                    changed |= tri_state_combo(ui, "Has prefix", &mut state.has_prefix);
                    changed |= tri_state_combo(ui, "Auto-update enabled", &mut state.auto_update);
                    changed |= tri_state_combo(ui, "Steam Cloud enabled", &mut state.cloud_sync);
                    changed |=
                        tri_state_combo(ui, "Custom launch options", &mut state.custom_launch);
                    changed |=
                        tri_state_combo(ui, "Custom Proton version", &mut state.custom_proton);
                    ui.separator();
                    egui::ComboBox::from_label("Sort By")
                        .selected_text(match state.sort_key {
                            SortKey::LastPlayed => "Last Modified",
                            SortKey::Name => "Name",
                            SortKey::AppId => "AppID",
                            SortKey::ProtonVersion => "Proton Version",
                        })
                        .show_ui(ui, |ui| {
                            changed |= ui
                                .selectable_value(
                                    &mut state.sort_key,
                                    SortKey::LastPlayed,
                                    "Last Modified",
                                )
                                .changed();
                            changed |= ui
                                .selectable_value(&mut state.sort_key, SortKey::Name, "Name")
                                .changed();
                            changed |= ui
                                .selectable_value(&mut state.sort_key, SortKey::AppId, "AppID")
                                .changed();
                            changed |= ui
                                .selectable_value(
                                    &mut state.sort_key,
                                    SortKey::ProtonVersion,
                                    "Proton Version",
                                )
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
                });
            });
        });

    if close_window {
        should_close = true;
    }

    if response.should_close() || should_close {
        *open = false;
    }
}
