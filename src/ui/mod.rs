use std::path::PathBuf;

use eframe::{egui::{self, Ui, vec2}, epi};
use futures_lite::future::{self, block_on, poll_once};
use id_tree::NodeId;
use rfd::AsyncFileDialog;

use crate::workspace::{Directory, Workspace};

pub struct App {
    workspace: Workspace,
    folder_picker: Option<future::Boxed<Option<PathBuf>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            workspace: Workspace::default(),
            folder_picker: None,
        }
    }
}

impl App {
    fn poll_updates(&mut self) {
        let mut clear_picker = false;

        if let Some(future) = self.folder_picker.as_mut() {
            if let Some(result) = block_on(poll_once(future)) {
                clear_picker = true;

                if let Some(dir) = result {
                    // Open a new workspace.
                    self.workspace.open(dir);

                    // Kick off reloading the dir tree.
                    self.workspace.refresh_async();
                }
            }
        }

        if clear_picker {
            self.folder_picker = None;
        }

        self.workspace.update();
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        env!("CARGO_PKG_NAME")
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        self.poll_updates();

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                egui::warn_if_debug_build(ui);
            });
        });

        egui::SidePanel::left("side_panel", 300.0).show(ctx, |ui| {
            ui.heading("Browser");

            if ui.button("Pick directory").clicked() {
                if self.folder_picker.is_none() {
                    self.folder_picker = Some(Box::pin(async {
                        AsyncFileDialog::new().pick_folder().await.map(|handle| handle.path().to_path_buf())
                    }));
                }
            }

            egui::containers::ScrollArea::auto_sized().show(ui, |ui| {
                if let Some(id) = self.workspace.directories().root_node_id() {
                    directory_tree(ui, &self.workspace, id);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File List");

            if let Some(node_id) = self.workspace.get_selected_directory() {
                let node = self.workspace.directories().get(&node_id).unwrap();
                ui.heading(node.data().path().to_string_lossy().as_ref());
            } else {
                ui.heading("Select a directory...");
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.heading("Wave Form");
            });
        });
    }
}

fn directory_tree(ui: &mut Ui, workspace: &Workspace, node_id: &NodeId) {
    if let Ok(node) = workspace.directories().get(node_id) {
        if ui.selectable_label(workspace.is_selected(node_id), node.data().name()).clicked() {
            workspace.set_selected_directory(node_id);
        }

        if !node.children().is_empty() {
            ui.group(|ui| {
                for child_id in node.children() {
                    directory_tree(ui, workspace, child_id);
                }
            });
        }
    }
}

pub fn main() {
    let app = App::default();
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(Box::new(app), native_options);
}
