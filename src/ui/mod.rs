use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
};

use eframe::{
    egui::{self, vec2, Ui},
    epi,
};
use id_tree::NodeId;

use crate::workspace::{Directory, Workspace};

use self::directory_picker::PickDirectoryTask;

mod directory_picker;
mod tasks;

pub struct App {
    workspace: Workspace,
    pick_directory_task: Option<PickDirectoryTask>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            workspace: Workspace::default(),
            pick_directory_task: None,
        }
    }
}

impl App {
    fn poll_updates(&mut self) {
        if let Some(task) = self.pick_directory_task.as_mut() {
            if let Some(result) = task.poll() {
                self.pick_directory_task = None;

                if let Some(dir) = result {
                    // Open a new workspace.
                    self.workspace.open(dir);
                }
            }
        }

        self.workspace.update();
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        env!("CARGO_PKG_NAME")
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        tasks::run_queued(self);

        self.poll_updates();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                egui::warn_if_debug_build(ui);
            });
        });

        egui::SidePanel::left("side_panel").default_width(200.0).show(ctx, |ui| {
            ui.heading("Browser");

            if ui.button("Pick directory").clicked() {
                if self.pick_directory_task.is_none() {
                    self.pick_directory_task = Some(PickDirectoryTask::new());
                }
            }

            egui::containers::ScrollArea::auto_sized().show(ui, |ui| {
                if let Some(id) = self.workspace.directories().root_node_id().cloned() {
                    directory_tree(ui, &self.workspace, &id);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File List");

            if let Some(node_id) = self.workspace.get_selected_directory() {
                let node = self.workspace.directories().get(&node_id).unwrap();
                ui.heading(node.data().path().to_string_lossy().as_ref());

                egui::containers::ScrollArea::auto_sized().show(ui, |ui| {
                    for sample in self.workspace.files() {
                        if let Some(note) = sample.note() {
                            ui.label(format!("{} -- {}", sample.name(), note));
                        } else {
                            ui.label(sample.name().as_ref());
                        }
                    }
                });
            } else {
                ui.heading("Select a directory...");
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.group(|ui| {
                    ui.heading("Wave Form");
                });
            });
        });
    }
}

fn directory_tree(ui: &mut Ui, workspace: &Workspace, node_id: &NodeId) {
    if let Ok(node) = workspace.directories().get(node_id) {
        if ui
            .selectable_label(workspace.is_selected(node_id), node.data().name())
            .clicked()
        {
            let node_id = node_id.clone();

            tasks::enqueue(move |app| {
                app.workspace.set_selected_directory(&node_id);
            });
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
