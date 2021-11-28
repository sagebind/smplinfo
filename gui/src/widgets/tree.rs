use std::{collections::HashSet, fmt::Display, hash::Hash};

use eframe::egui::{Id, Label, Response, ScrollArea, Sense, Ui, Widget};
use id_tree::{NodeId, Tree};

const CLOSED_ARROW: &str = "⏵";
const OPEN_ARROW: &str = "⏷";

/// A tree view which supports an arbitrary tree of selectable labels with
/// collapsible nodes.
pub struct TreeView<'tree, 'selected, T> {
    id_source: Id,
    tree: &'tree Tree<T>,
    selected_node: &'selected mut Option<NodeId>,
    skip_root_node: bool,
}

impl<'tree, 'selected, T> TreeView<'tree, 'selected, T> {
    /// Create a new tree view to display a given tree of data.
    ///
    /// - `id_source`: Unique identifier for this widget.
    /// - `tree`: A tree to display.
    /// - `selected_node`: Used to get/set the currently selected node, if any.
    pub fn new(
        id_source: impl Hash,
        tree: &'tree Tree<T>,
        selected_node: &'selected mut Option<NodeId>,
    ) -> Self {
        Self {
            id_source: Id::new(id_source),
            tree,
            selected_node,
            skip_root_node: false,
        }
    }

    /// Enable or disable displaying the root node of the tree. If enabled, the
    /// top level of the tree will be "flattened" and only the direct child
    /// nodes of the root node will be displayed together with their ancestors.
    pub fn skip_root_node(mut self, skip: bool) -> Self {
        self.skip_root_node = skip;
        self
    }
}

impl<'tree, 'selected, T: Display> Widget for TreeView<'tree, 'selected, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut state = ui
            .memory()
            .id_data_temp
            .get_or_default::<State>(self.id_source.clone())
            .clone();

        let mut changed = false;

        let mut response = ui
            .vertical(|ui| {
                ScrollArea::auto_sized().show(ui, |ui| {
                    if let Some(id) = self.tree.root_node_id().cloned() {
                        if ui
                            .add(Node {
                                tree: self.tree,
                                node_id: &id,
                                selected_node: self.selected_node,
                                state: &mut state,
                            })
                            .changed()
                        {
                            changed = true;
                        }
                    }
                });
            })
            .response;

        if changed {
            response.mark_changed();
        }

        ui.memory().id_data_temp.insert(self.id_source, state);

        response
    }
}

struct Node<'tree, 'selected, 'state, T> {
    tree: &'tree Tree<T>,
    node_id: &'tree NodeId,
    selected_node: &'selected mut Option<NodeId>,
    state: &'state mut State,
}

impl<'tree, 'selected, 'state, T: Display> Widget for Node<'tree, 'selected, 'state, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let node = self.tree.get(self.node_id).unwrap();
        let is_open = self.state.is_open(self.node_id);
        let mut changed = false;

        let mut response = ui
            .vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            Label::new(if is_open { OPEN_ARROW } else { CLOSED_ARROW })
                                .sense(Sense::click()),
                        )
                        .clicked()
                    {
                        self.state.toggle_open(self.node_id);
                    }

                    if ui
                        .selectable_label(
                            self.selected_node.as_ref() == Some(self.node_id),
                            node.data(),
                        )
                        .clicked()
                    {
                        *self.selected_node = Some(self.node_id.clone());
                        changed = true;
                    }
                });

                if is_open && !node.children().is_empty() {
                    ui.indent("foo", |ui| {
                        for child_id in node.children() {
                            if ui
                                .add(Node {
                                    tree: self.tree,
                                    node_id: child_id,
                                    selected_node: self.selected_node,
                                    state: self.state,
                                })
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
                }
            })
            .response;

        if changed {
            response.mark_changed();
        }

        response
    }
}

#[derive(Clone, Debug, Default)]
struct State {
    open_nodes: HashSet<NodeId>,
}

impl State {
    fn is_open(&self, node_id: &NodeId) -> bool {
        self.open_nodes.contains(node_id)
    }

    fn toggle_open(&mut self, node_id: &NodeId) {
        if !self.open_nodes.remove(node_id) {
            self.open_nodes.insert(node_id.clone());
        }
    }
}
