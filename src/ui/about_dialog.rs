use futures_lite::{future::Boxed, FutureExt};
use rfd::{AsyncMessageDialog, MessageButtons};

use crate::futures::poll_once_blocking;

#[derive(Default)]
pub struct AboutDialog {
    future: Option<Boxed<bool>>,
}

impl AboutDialog {
    pub fn is_active(&self) -> bool {
        self.future.is_some()
    }

    pub fn show(&mut self) {
        if let Some(future) = self.future.as_mut() {
            if let Some(_) = poll_once_blocking(future) {
                self.future = None;
            } else {
                return;
            }
        }

        let future = AsyncMessageDialog::new()
            .set_title("About")
            .set_description(&about_string())
            .set_buttons(MessageButtons::Ok)
            .show()
            .boxed();

        self.future = Some(future);
    }
}

fn about_string() -> String {
    if cfg!(debug_assertions) {
        format!(
            "{} {} (debug build)",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )
    } else {
        format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}
