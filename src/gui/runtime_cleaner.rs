use crate::utils::runtime_cleaner::{delete_item, scan, RuntimeItem, ScanResults};
use eframe::egui::{self, Modal};
use open;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use tinyfiledialogs as tfd;

pub struct RuntimeCleanerWindow {
    results: ScanResults,
    loading: bool,
    rx: Option<Receiver<ScanResults>>,
    needs_refresh: bool,
}

impl RuntimeCleanerWindow {
    pub fn new() -> Self {
        Self {
            results: ScanResults::default(),
            loading: false,
            rx: None,
            needs_refresh: true,
        }
    }

    fn start_scan(&mut self) {
        self.loading = true;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let res = scan();
            let _ = tx.send(res);
        });
        self.rx = Some(rx);
    }

    fn any_selected(&self) -> bool {
        self.results.install_folders.iter().any(|i| i.selected)
            || self.results.prefixes.iter().any(|i| i.selected)
            || self.results.shader_caches.iter().any(|i| i.selected)
            || self.results.tools.iter().any(|i| i.selected)
    }

    fn select_all(&mut self, val: bool) {
        for list in [
            &mut self.results.install_folders,
            &mut self.results.prefixes,
            &mut self.results.shader_caches,
            &mut self.results.tools,
        ] {
            for item in list.iter_mut() {
                item.selected = val;
            }
        }
    }

    fn delete_selected(&mut self) {
        for list in [
            &mut self.results.install_folders,
            &mut self.results.prefixes,
            &mut self.results.shader_caches,
            &mut self.results.tools,
        ] {
            let mut idx = 0;
            while idx < list.len() {
                if list[idx].selected {
                    if delete_item(&list[idx]).is_ok() {
                        list.remove(idx);
                        continue;
                    }
                }
                idx += 1;
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        if !*open {
            self.rx = None;
            self.loading = false;
            self.needs_refresh = true;
            return;
        }

        if self.needs_refresh && !self.loading {
            self.start_scan();
        }

        if let Some(rx) = &self.rx {
            if let Ok(res) = rx.try_recv() {
                self.results = res;
                self.loading = false;
                self.needs_refresh = false;
                self.rx = None;
            }
        }

        let mut should_close = false;
        let response = Modal::new(egui::Id::new("runtime_cleaner"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Steam Runtime Cleaner");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            should_close = true;
                        }
                    });
                });

                ui.horizontal(|ui| {
                    if ui.button("Select All").clicked() {
                        self.select_all(true);
                    }
                    if ui.button("Deselect All").clicked() {
                        self.select_all(false);
                    }
                    if ui
                        .add_enabled(self.any_selected(), egui::Button::new("Delete Selected"))
                        .clicked()
                    {
                        if tfd::message_box_yes_no(
                            "Confirm",
                            "Delete selected items?",
                            tfd::MessageBoxIcon::Warning,
                            tfd::YesNo::No,
                        ) == tfd::YesNo::Yes
                        {
                            self.delete_selected();
                        }
                    }
                });

                ui.separator();

                if self.loading {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        ui.label("Scanning...");
                    });
                    return;
                }

                Self::show_group(
                    ui,
                    "Orphaned Install Folders",
                    &mut self.results.install_folders,
                );
                Self::show_group(ui, "Orphaned Proton Prefixes", &mut self.results.prefixes);
                Self::show_group(ui, "Unused Shader Caches", &mut self.results.shader_caches);
                Self::show_group(ui, "Broken Custom Proton Versions", &mut self.results.tools);
            });

        if response.should_close() || should_close {
            *open = false;
        }
    }

    fn show_group(ui: &mut egui::Ui, title: &str, items: &mut Vec<RuntimeItem>) {
        egui::CollapsingHeader::new(title)
            .default_open(true)
            .show(ui, |ui| {
                for item in items.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut item.selected, "");
                        if ui
                            .button("📂")
                            .on_hover_text("Show in File Manager")
                            .clicked()
                        {
                            let _ = open::that(&item.path);
                        }
                        let lbl = if let Some(id) = item.app_id {
                            format!("{} (AppID {})", item.path.display(), id)
                        } else {
                            item.path.display().to_string()
                        };
                        ui.label(lbl);
                        ui.label(egui::RichText::new(&item.reason).italics());
                        if !item.verified {
                            ui.label(
                                egui::RichText::new("[unverified]").color(egui::Color32::YELLOW),
                            );
                        }
                    });
                }
                if items.is_empty() {
                    ui.label("None found");
                }
            });
    }
}
