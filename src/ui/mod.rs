use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
};

use eframe::{
    egui::{self, vec2, Layout, Ui},
    epi,
};
use id_tree::NodeId;

use crate::workspace::{Directory, Workspace};

use self::{
    about_dialog::AboutDialog,
    directory_picker::PickDirectoryTask,
    widgets::{Table, TreeView},
};

mod about_dialog;
mod directory_picker;
mod tasks;
mod widgets;

pub struct App {
    workspace: Workspace,
    pick_directory_task: Option<PickDirectoryTask>,
    about_window_open: bool,
    about_dialog: AboutDialog,
    selected_directory: Option<NodeId>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            workspace: Workspace::default(),
            pick_directory_task: None,
            about_window_open: false,
            about_dialog: AboutDialog::default(),
            selected_directory: None,
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

                egui::menu::menu(ui, "Help", |ui| {
                    if ui.button("About").clicked() {
                        // self.about_window_open = true;
                        self.about_dialog.show();
                    }
                });

                egui::warn_if_debug_build(ui);
            });
        });

        egui::SidePanel::left("side_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Browser");

                if ui.button("Pick directory").clicked() {
                    if self.pick_directory_task.is_none() {
                        self.pick_directory_task = Some(PickDirectoryTask::new());
                    }
                }

                if ui.add(TreeView::new("dir_tree", self.workspace.directories(), &mut self.selected_directory)).changed() {
                    if let Some(node_id) = self.selected_directory.clone() {
                        tasks::enqueue(move |app| {
                            app.workspace.set_selected_directory(&node_id);
                        });
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File List");

            if let Some(node_id) = self.workspace.get_selected_directory() {
                let node = self.workspace.directories().get(&node_id).unwrap();

                let mut path = node.data().path().to_string_lossy().into_owned();
                ui.add(egui::TextEdit::singleline(&mut path).enabled(false));

                egui::containers::ScrollArea::auto_sized().show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
                        // ui.centered_and_justified(|ui| {
                        ui.add(
                            Table::new("file_list")
                                .column("Filename")
                                .column("Note")
                                .rows(self.workspace.files(), |sample, ui| {
                                    ui.label(sample.name().as_ref());

                                    if let Some(note) = sample.note() {
                                        ui.label(note.to_string());
                                    } else {
                                        ui.label("-");
                                    }
                                }),
                        );
                    });
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

pub fn main() {
    let app = App::default();
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(Box::new(app), native_options);
}
