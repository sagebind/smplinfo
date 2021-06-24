use std::path::PathBuf;

use futures_lite::{
    future::{block_on, poll_once, Boxed},
    FutureExt,
};
use rfd::{AsyncFileDialog, FileHandle};

pub struct PickDirectoryTask {
    future: Option<Boxed<Option<FileHandle>>>,
}

impl PickDirectoryTask {
    pub fn new() -> Self {
        Self {
            future: Some(AsyncFileDialog::new().pick_folder().boxed()),
        }
    }

    pub fn poll(&mut self) -> Option<Option<PathBuf>> {
        if let Some(future) = self.future.as_mut() {
            if let Some(result) = block_on(poll_once(future)) {
                self.future = None;
                return Some(result.map(|handle| handle.path().to_path_buf()));
            }
        }

        None
    }
}
