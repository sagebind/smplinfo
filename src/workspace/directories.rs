use std::{fmt, path::{Path, PathBuf}};

use id_tree::Tree;

pub struct DirectoryTree {
    tree: Tree<Directory>,
}

pub struct Directory {
    name: Option<String>,
    path: PathBuf,
    expanded: bool,
}

impl Directory {
    pub fn new(path: impl Into<PathBuf>) -> Self {
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

impl fmt::Display for Directory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
    }
}
