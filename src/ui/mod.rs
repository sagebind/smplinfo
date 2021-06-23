use eframe::{egui, epi};
use rfd::FileDialog;

use crate::workspace::Workspace;

pub struct App {
    workspace: Workspace,
}

impl Default for App {
    fn default() -> Self {
        Self {
            workspace: Workspace::default(),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        env!("CARGO_PKG_NAME")
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let workspace = &mut self.workspace;

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                egui::warn_if_debug_build(ui);
            });
        });

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Browser");

            if ui.button("Pick directory").clicked() {
                if let Some(dir) = FileDialog::new().pick_folder() {
                    // Open a new workspace.
                    *workspace = Workspace::open(dir);

                    // Kick off reloading the dir tree.
                    workspace.refresh_async();
                }
            }

            egui::containers::ScrollArea::auto_sized().show(ui, |ui| {
                for dir in workspace.root_directory().children() {
                    ui.label(dir.name());

                    for dir in dir.children() {
                        ui.label("\\- ".to_string() + dir.name());

                        for dir in dir.children() {
                            ui.label("    \\- ".to_string() + dir.name());
                        }
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Select a directory...");
        });
    }
}

pub fn main() {
    let app = App::default();
    let native_options = eframe::NativeOptions::default();
    app.workspace.refresh_async();

    eframe::run_native(Box::new(app), native_options);
}
