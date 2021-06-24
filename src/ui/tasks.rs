//! A primitive system for deferring a closure to execute in the UI thread
//! before the next frame.

use std::sync::mpsc::{self, Receiver, Sender};

use super::App;

type UiTask = Box<dyn FnOnce(&mut App)>;

thread_local! {
    static TASK_QUEUE: (Sender<UiTask>, Receiver<UiTask>) = mpsc::channel();
}

pub fn enqueue(task: impl FnOnce(&mut App) + 'static) {
    TASK_QUEUE.with(move |(sender, _)| {
        sender.send(Box::new(task));
    });
}

pub fn run_queued(app: &mut App) {
    TASK_QUEUE.with(move |(_, receiver)| {
        while let Ok(task) = receiver.try_recv() {
            task(app);
        }
    });
}
