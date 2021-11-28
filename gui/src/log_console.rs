use eframe::egui::{Response, Sense, Ui, Widget};

pub struct LogConsole;

impl Widget for LogConsole {
    fn ui(self, ui: &mut Ui) -> Response {
        super::logger::each_log(|log| {
            ui.monospace(log);
        });

        ui.interact(ui.available_rect_before_wrap(), ui.id(), Sense::hover())
    }
}
