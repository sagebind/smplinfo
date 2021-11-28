use std::{borrow::Cow, cell::RefCell, env, fs::read_dir, path::{Path, PathBuf}, sync::mpsc, sync::{Arc, Mutex}, thread};

use id_tree::{InsertBehavior, Node, NodeId, Tree};
use threadpool::ThreadPool;

pub mod directories;
mod manager;

pub use directories::Directory;

use smplinfo::sample::Sample;

pub enum Event {
    DirectoryTreeUpdated(directories::DirectoryTree),
}

pub enum Command {
    /// Open a directory as a workspace.
    Open(PathBuf),
}

pub struct Workspace {
    pool: ThreadPool,
    path: Option<PathBuf>,
    directories: Tree<Directory>,
    selected_directory: Option<NodeId>,
    files: Vec<Sample>,
    loader_channel: (
        mpsc::Sender<Tree<Directory>>,
        mpsc::Receiver<Tree<Directory>>,
    ),
    file_loader_channel: (
        mpsc::Sender<Sample>,
        mpsc::Receiver<Sample>,
    ),
}

impl Workspace {
    pub fn open(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.path = Some(path.clone());
        self.directories = Tree::new();
        self.files.clear();
        self.selected_directory = None;

        self.refresh_async();
    }

    pub fn directories(&self) -> &Tree<Directory> {
        &self.directories
    }

    pub fn files(&self) -> &[Sample] {
        &self.files
    }

    pub fn is_selected(&self, node_id: &NodeId) -> bool {
        self.selected_directory.as_ref() == Some(node_id)
    }

    pub fn get_selected_directory(&self) -> Option<NodeId> {
        self.selected_directory.clone()
    }

    pub fn set_selected_directory(&mut self, node_id: &NodeId) {
        self.selected_directory = Some(node_id.clone());
        self.files.clear();

        let sender = self.file_loader_channel.0.clone();
        let path = self.directories.get(node_id).unwrap().data().path().to_path_buf();
        println!("selected directory changed to {:?}", path);

        // TODO: Cancel any existing tasks loading samples.
        self.pool.execute(move || {
            for entry in read_dir(path).unwrap() {
                let entry = entry.unwrap();

                if entry.metadata().unwrap().is_file() {
                    if let Ok(sample) = Sample::read(entry.path()) {
                        sender.send(sample);
                    }
                }
            }
        });
    }

    /// Get latest async updates.
    pub fn update(&mut self) {
        while let Ok(tree) = self.loader_channel.1.try_recv() {
            self.directories = tree;
        }

        while let Ok(file) = self.file_loader_channel.1.try_recv() {
            self.files.push(file);
        }
    }

    /// Refresh the directory list in the workspace asynchronously.
    pub fn refresh_async(&self) {
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

        if let Some(path) = self.path.as_ref() {
            let path = path.clone();
            let sender = self.loader_channel.0.clone();

            self.pool.execute(move || {
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
            pool: ThreadPool::new(4),
            path: None,
            directories: Tree::new(),
            selected_directory: None,
            files: Vec::new(),
            loader_channel: mpsc::channel(),
            file_loader_channel: mpsc::channel(),
        }
    }
}
