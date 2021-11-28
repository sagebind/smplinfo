//! Custom widgets for egui.
//!
//! Egui is pretty dope, but the stock widget library is currently pretty small.
//! Maybe one day widgets such as these would be included upstream?

mod table;
mod tree;

pub use self::{table::Table, tree::TreeView};
