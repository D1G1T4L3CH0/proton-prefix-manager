use crate::core::{models::GameInfo, steam};
use crate::utils::backup as backup_utils;
use eframe::egui;
use eframe::egui::Modal;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::fs;
use std::path::{Path, PathBuf};
use tinyfiledialogs as tfd;

pub struct BackupEntry {
    pub app_id: u32,
    pub game_name: String,
    pub path: PathBuf,
    pub size: u64,
    pub created: String,
    pub selected: bool,
}

pub struct BackupManagerWindow {
    entries: Vec<BackupEntry>,
    confirm_delete_all: bool,
    needs_refresh: bool,
    loading: bool,
    rx: Option<Receiver<Vec<BackupEntry>>>,
}

impl BackupManagerWindow {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            confirm_delete_all: false,
            needs_refresh: true,
            loading: false,
            rx: None,
        }
    }

    fn dir_size(path: &Path) -> std::io::Result<u64> {
        let mut size = 0;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let md = entry.metadata()?;
            if md.is_dir() {
                size += Self::dir_size(&entry.path())?;
            } else {
                size += md.len();
            }
        }
        Ok(size)
    }

    fn format_size(size: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        let f = size as f64;
        if f >= GB {
            format!("{:.1} GB", f / GB)
        } else if f >= MB {
            format!("{:.1} MB", f / MB)
        } else if f >= KB {
            format!("{:.1} KB", f / KB)
        } else {
            format!("{} B", size)
        }
    }

    fn collect_entries(games: Option<Vec<GameInfo>>) -> Vec<BackupEntry> {
        let all = backup_utils::list_all_backups();
        let mut entries = Vec::new();
        for (appid, backups) in all {
            let game_name = games
                .as_deref()
                .and_then(|g| g.iter().find(|x| x.app_id() == appid))
                .map(|g| g.name().to_string())
                .unwrap_or_else(|| format!("App {}", appid));
            for b in backups {
                let size = Self::dir_size(&b).unwrap_or(0);
                let created = backup_utils::format_backup_name(&b);
                entries.push(BackupEntry {
                    app_id: appid,
                    game_name: game_name.clone(),
                    path: b,
                    size,
                    created,
                    selected: false,
                });
            }
        }
        entries
    }

    fn start_refresh(&mut self, games: Option<&[GameInfo]>) {
        self.entries.clear();
        self.loading = true;
        let rx_slot = {
            let games_owned = games.map(|g| g.to_vec());
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                let entries = Self::collect_entries(games_owned);
                let _ = tx.send(entries);
            });
            rx
        };
        self.rx = Some(rx_slot);
    }

    fn prefix_for(app_id: u32, games: Option<&[GameInfo]>) -> Option<PathBuf> {
        if let Some(g) = games.and_then(|g| g.iter().find(|x| x.app_id() == app_id)) {
            return Some(g.prefix_path().to_path_buf());
        }
        if let Ok(libs) = steam::get_steam_libraries() {
            return steam::find_proton_prefix(app_id, &libs);
        }
        None
    }

    fn delete_selected(&mut self) {
        let paths: Vec<PathBuf> = self
            .entries
            .iter()
            .filter(|e| e.selected)
            .map(|e| e.path.clone())
            .collect();
        for p in paths {
            let _ = backup_utils::delete_backup(&p);
        }
        self.needs_refresh = true;
    }

    fn delete_all(&mut self) {
        for e in &self.entries {
            let _ = backup_utils::delete_backup(&e.path);
        }
        self.needs_refresh = true;
    }

    fn has_selection(&self) -> bool {
        self.entries.iter().any(|e| e.selected)
    }

    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool, games: Option<&[GameInfo]>) {
        if !*open {
            self.entries.clear();
            self.rx = None;
            self.loading = false;
            self.needs_refresh = true;
            return;
        }
        if self.needs_refresh && !self.loading {
            self.start_refresh(games);
        }

        if let Some(rx) = &self.rx {
            if let Ok(entries) = rx.try_recv() {
                self.entries = entries;
                self.loading = false;
                self.needs_refresh = false;
                self.rx = None;
            }
        }

        let mut should_close = false;
        let response = Modal::new(egui::Id::new("backup_manager"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Prefix Backups");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            should_close = true;
                        }
                    });
                });
                ui.horizontal(|ui| {
                    let delete_enabled = self.has_selection();
                    if ui.add_enabled(delete_enabled, egui::Button::new("Delete Selected")).clicked() {
                        if tfd::message_box_yes_no(
                            "Confirm",
                            "Delete selected backups?",
                            tfd::MessageBoxIcon::Warning,
                            tfd::YesNo::No,
                        ) == tfd::YesNo::Yes
                        {
                            self.delete_selected();
                        }
                    }
                    if ui.button("Delete All Backups").clicked() {
                        self.confirm_delete_all = true;
                    }
                });

                if self.loading {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        ui.label("Loading backups...");
                    });
                } else {
                    egui::Grid::new("backups_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.heading("Game Name");
                            ui.heading("App ID");
                            ui.heading("Backup");
                            ui.heading("Size");
                            ui.heading("Actions");
                            ui.end_row();

                            for entry in &mut self.entries {
                                ui.label(&entry.game_name);
                                ui.label(entry.app_id.to_string());
                                ui.label(&entry.created);
                                ui.label(Self::format_size(entry.size));
                                ui.horizontal(|ui| {
                                    if ui.button("Restore").clicked() {
                                        if let Some(prefix) = Self::prefix_for(entry.app_id, games) {
                                            match backup_utils::restore_prefix(&entry.path, &prefix) {
                                                Ok(_) => tfd::message_box_ok("Restore", "Prefix restored", tfd::MessageBoxIcon::Info),
                                                Err(e) => tfd::message_box_ok("Restore failed", &format!("{}", e), tfd::MessageBoxIcon::Error),
                                            };
                                        } else {
                                            tfd::message_box_ok("Restore failed", "Prefix path not found", tfd::MessageBoxIcon::Error);
                                        }
                                    }
                                    if ui.button("Delete").clicked() {
                                        match backup_utils::delete_backup(&entry.path) {
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
                                        self.needs_refresh = true;
                                    }
                                });
                                ui.checkbox(&mut entry.selected, "");
                                ui.end_row();
                            }
                        });
                }

                if self.confirm_delete_all {
                    if tfd::message_box_yes_no(
                        "Confirm",
                        "Are you sure you want to delete all backups? This action cannot be undone.",
                        tfd::MessageBoxIcon::Warning,
                        tfd::YesNo::No,
                    ) == tfd::YesNo::Yes
                    {
                        self.delete_all();
                    }
                    self.confirm_delete_all = false;
                }
            });

        if response.should_close() || should_close {
            *open = false;
        }
    }
}
