#![windows_subsystem = "windows"]

use eframe::{egui::{self, Layout, ScrollArea, Window, vec2}, epi};
use id_tree::NodeId;

use self::{
    about_dialog::AboutDialog,
    directory_picker::PickDirectoryTask,
    widgets::{Table, TreeView},
    workspace::Workspace,
};

mod about_dialog;
mod directory_picker;
mod futures;
mod log_console;
pub mod logger;
mod tasks;
mod widgets;
mod workspace;

pub struct App {
    workspace: Workspace,
    pick_directory_task: Option<PickDirectoryTask>,
    about_window_open: bool,
    about_dialog: AboutDialog,
    selected_directory: Option<NodeId>,
    selected_sample_index: Option<usize>,
    developer_mode: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            workspace: Workspace::default(),
            pick_directory_task: None,
            about_window_open: false,
            about_dialog: AboutDialog::default(),
            selected_directory: None,
            selected_sample_index: None,
            developer_mode: false,
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

        if ctx.input().key_pressed(egui::Key::D) && ctx.input().modifiers.ctrl {
            self.developer_mode = !self.developer_mode;
            log::debug!("developer mode: {}", self.developer_mode);
        }

        if self.developer_mode {
            Window::new("Egui Settings").show(ctx, |ui| {
                ctx.settings_ui(ui);
            });
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                egui::menu::menu(ui, "Help", |ui| {
                    if ui.button("About").clicked() {
                        self.about_dialog.show();
                    }
                });
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

                if ui
                    .add(TreeView::new(
                        "dir_tree",
                        self.workspace.directories(),
                        &mut self.selected_directory,
                    ))
                    .changed()
                {
                    if let Some(node_id) = self.selected_directory.clone() {
                        self.selected_sample_index = None;
                        tasks::enqueue(move |app| {
                            app.workspace.set_selected_directory(&node_id);
                        });
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.heading("Wave Form");
            });

            let mut path = self.workspace.get_selected_directory()
                .and_then(|node_id| self.workspace.directories().get(&node_id).ok())
                .map(|node| node.data().path().to_string_lossy().into_owned())
                .unwrap_or_default();

            ui.add(egui::TextEdit::singleline(&mut path).enabled(false));

            ui.add(
                Table::new_selectable(
                    "file_list",
                    self.workspace.files(),
                    &mut self.selected_sample_index,
                )
                .column("Filename", |sample| sample.name().into_owned())
                .column("Note", |sample| {
                    if let Some(note) = sample.note() {
                        note.to_string()
                    } else {
                        "-".to_owned()
                    }
                }),
            );
        });

        if self.developer_mode {
            egui::TopBottomPanel::bottom("log_console")
            .resizable(true)
            .default_height(200.0)
            .show(ctx, |ui| {
                ScrollArea::auto_sized().show(ui, |ui| {
                    ui.add(log_console::LogConsole);
                });
            });
        }
    }
}

pub fn main() {
    logger::init();

    let app = App::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(Box::new(app), native_options);
}
