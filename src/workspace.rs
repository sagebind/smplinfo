use std::{cell::RefCell, env, fs::read_dir, path::{Path, PathBuf}, sync::mpsc, sync::{Arc, Mutex}, thread};

use id_tree::{InsertBehavior, Node, NodeId, Tree};

pub struct Workspace {
    path: Option<PathBuf>,
    directories: Tree<Directory>,
    selected_directory: RefCell<Option<NodeId>>,
    loader_channel: (
        mpsc::Sender<Tree<Directory>>,
        mpsc::Receiver<Tree<Directory>>,
    ),
}

impl Workspace {
    pub fn open(&mut self, path: impl Into<PathBuf>) {
        self.path = Some(path.into());
        self.directories = Tree::new();
        *self.selected_directory.borrow_mut() = None;
    }

    pub fn directories(&self) -> &Tree<Directory> {
        &self.directories
    }

    pub fn is_selected(&self, node_id: &NodeId) -> bool {
        self.selected_directory.borrow().as_ref() == Some(node_id)
    }

    pub fn get_selected_directory(&self) -> Option<NodeId> {
        self.selected_directory.borrow().clone()
    }

    pub fn set_selected_directory(&self, node_id: &NodeId) {
        *self.selected_directory.borrow_mut() = Some(node_id.clone());
    }

    /// Get latest async updates.
    pub fn update(&mut self) {
        while let Ok(tree) = self.loader_channel.1.try_recv() {
            self.directories = tree;
        }
    }

    /// Refresh the directory list in the workspace asynchronously.
    pub fn refresh_async(&self) {
        if let Some(path) = self.path.as_ref() {
            let path = path.clone();
            let sender = self.loader_channel.0.clone();

            thread::spawn(move || {
                let mut tree = Tree::new();

                let root = Directory::new(path.clone());
                let root_id = tree
                    .insert(Node::new(root), InsertBehavior::AsRoot)
                    .unwrap();

                load_directories(path, &mut tree, root_id);

                let _ = sender.send(tree);
            });
        }
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            path: None,
            directories: Tree::new(),
            selected_directory: RefCell::new(None),
            loader_channel: mpsc::channel(),
        }
    }
}

pub struct Directory {
    name: Option<String>,
    path: PathBuf,
    expanded: bool,
}

impl Directory {
    fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        Self {
            name: path.file_name().map(|s| s.to_string_lossy().into_owned()),
            path,
            expanded: false,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unknown>")
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

fn load_directories(path: impl AsRef<Path>, tree: &mut Tree<Directory>, parent_id: NodeId) {
    for entry in read_dir(path).unwrap() {
        let entry = entry.unwrap();

        if entry.metadata().unwrap().is_dir() {
            let child = Directory::new(entry.path());
            let child_id = tree
                .insert(Node::new(child), InsertBehavior::UnderNode(&parent_id))
                .unwrap();

            load_directories(entry.path(), tree, child_id);
        }
    }
}
