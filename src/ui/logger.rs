use std::{collections::VecDeque, sync::Mutex};

use once_cell::sync::Lazy;

static BUFFER: Lazy<Mutex<VecDeque<String>>> = Lazy::new(|| Mutex::new(VecDeque::new()));
const BUFFER_LIMIT: usize = 128;

pub fn init() {
    static INSTANCE: MemoryLogger = MemoryLogger;

    log::set_logger(&INSTANCE).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
}

pub fn each_log(mut f: impl FnMut(String)) {
    let buffer = BUFFER.lock().unwrap();

    for log in buffer.iter() {
        f(log.clone());
    }
}

pub struct MemoryLogger;

impl log::Log for MemoryLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut buffer = BUFFER.lock().unwrap();

        buffer.truncate(BUFFER_LIMIT);
        buffer.push_back(format!("{} - {}", record.level(), record.args()))
    }

    fn flush(&self) {}
}
