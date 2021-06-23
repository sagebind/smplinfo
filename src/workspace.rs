use std::{
    env,
    fs::read_dir,
    path::{Path, PathBuf},
    sync::mpsc,
    sync::{Arc, Mutex},
    thread,
};

pub struct Workspace {
    path: PathBuf,
    root_directory: Directory,
    loader_channel: (mpsc::Sender<Directory>, mpsc::Receiver<Directory>),
}

impl Workspace {
    pub fn open(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        Self {
            path: path.to_path_buf(),
            root_directory: Directory {
                name: path.file_name().map(|s| s.to_string_lossy().into_owned()),
                children: Vec::new(),
            },
            loader_channel: mpsc::channel(),
        }
    }

    pub fn root_directory(&mut self) -> &Directory {
        while let Ok(dir) = self.loader_channel.1.try_recv() {
            self.root_directory = dir;
        }

        &self.root_directory
    }

    /// Refresh the directory list in the workspace asynchronously.
    pub fn refresh_async(&self) {
        let path = self.path.clone();
        let sender = self.loader_channel.0.clone();

        thread::spawn(move || {
            let dir = Directory::load(path);
            let _ = sender.send(dir);
        });
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::open(env::current_dir().unwrap())
    }
}

pub struct Directory {
    name: Option<String>,
    children: Vec<Directory>,
}

impl Directory {
    fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        let mut dir = Self {
            name: path.file_name().map(|s| s.to_string_lossy().into_owned()),
            children: Vec::new(),
        };

        for entry in read_dir(path).unwrap() {
            let entry = entry.unwrap();

            if entry.metadata().unwrap().is_dir() {
                dir.children.push(Self::load(entry.path()));
            }
        }

        dir
    }

    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unknown>")
    }

    pub fn children(&self) -> &[Directory] {
        &self.children
    }
}
