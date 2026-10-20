use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};

// a logger that keeps every record, so a test can count the warnings it caused
pub struct Logs {
    records: Mutex<Vec<(Level, String)>>,
}

static LOGS: Logs = Logs { records: Mutex::new(Vec::new()) };

impl Logs {
    // installs the logger once per test process; later calls change nothing
    pub fn install() {
        let _ = log::set_logger(&LOGS);
        log::set_max_level(LevelFilter::Trace);
    }

    // records at `level` whose message contains `text`
    pub fn count(level: Level, text: &str) -> usize {
        let records = LOGS.records.lock().unwrap();
        records.iter().filter(|(at, message)| *at == level && message.contains(text)).count()
    }
}

impl Log for Logs {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        self.records.lock().unwrap().push((record.level(), record.args().to_string()));
    }

    fn flush(&self) {}
}
