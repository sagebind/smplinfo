use std::{fmt::Display, hash::Hash};

use eframe::egui::{Grid, Id, Response, Ui, Widget};

pub struct Table<'rows, R> {
    id_source: Id,
    columns: Vec<String>,
    rows: Option<&'rows [R]>,
    row_widget: Option<Box<dyn FnMut(&R, &mut Ui)>>,
}

impl<'rows, R> Table<'rows, R> {
    pub fn new(id_source: impl Hash) -> Self {
        Self {
            id_source: Id::new(id_source),
            columns: Vec::new(),
            rows: None,
            row_widget: None,
        }
    }

    pub fn column(mut self, name: impl Display) -> Self {
        self.columns.push(name.to_string());
        self
    }

    pub fn rows(mut self, rows: &'rows [R], widget: impl FnMut(&R, &mut Ui) + 'static) -> Self {
        self.rows = Some(rows);
        self.row_widget = Some(Box::new(widget));
        self
    }
}

impl<'rows, R> Widget for Table<'rows, R> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        Grid::new(self.id_source)
            .striped(true)
            .show(ui, move |ui| {
                for column in self.columns {
                    ui.button(column);
                }

                ui.end_row();

                if let Some(rows) = self.rows.take() {
                    for row in rows {
                        (self.row_widget.as_mut().unwrap())(row, ui);
                        ui.end_row();
                    }
                }
            })
            .response
    }
}
