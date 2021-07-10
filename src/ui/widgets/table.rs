use std::{fmt::Display, hash::Hash};

use eframe::egui::{self, Grid, Id, Layout, Response, Ui, Widget};

/// An ordinary table. Feature set is currently pretty limited.
///
/// - `R`: The data type of a single row displayed.
/// - `C`: The type of collection holding the rows to display. Any collection
///   implementing `AsRef<[R]>` can be used.
pub struct Table<R, C: AsRef<[R]>> {
    id_source: Id,
    rows: C,
    columns: Vec<Column<R>>,
}

/// Table column definition.
struct Column<R> {
    name: String,
    value_mapper: Box<dyn FnMut(&R) -> String>
}

impl<R, C: AsRef<[R]>> Table<R, C> {
    pub fn new(id_source: impl Hash, rows: C) -> Self {
        Self {
            id_source: Id::new(id_source),
            rows,
            columns: Vec::new(),
        }
    }

    pub fn column(mut self, name: impl Display, value_mapper: impl FnMut(&R) -> String + 'static) -> Self {
        self.columns.push(Column {
            name: name.to_string(),
            value_mapper: Box::new(value_mapper),
        });
        self
    }
}

impl<R, C: AsRef<[R]>> Widget for Table<R, C> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            Grid::new(self.id_source)
                .striped(true)
                .show(ui, move |ui| {
                    for column in self.columns.iter() {
                        ui.button(&column.name);
                    }

                    ui.end_row();

                    for row in self.rows.as_ref() {
                        for column in self.columns.iter_mut() {
                            ui.label((column.value_mapper)(row));
                        }

                        ui.end_row();
                    }
                })
                .response
        }).response
    }
}
