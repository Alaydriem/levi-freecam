use std::sync::atomic::{AtomicUsize, Ordering};

use log::{Level, LevelFilter, Log, Metadata, Record};

use crate::types::FcLogFn;

// forwards `log` records to the shim's LeviLamina logger
pub struct LogBridge {
    sink: AtomicUsize,
}

/// cbindgen:ignore
static BRIDGE: LogBridge = LogBridge { sink: AtomicUsize::new(0) };

impl LogBridge {
    // the first call installs the bridge; later calls only replace the sink
    pub fn install(sink: FcLogFn) {
        let address = sink.map_or(0, |f| f as usize);
        BRIDGE.sink.store(address, Ordering::SeqCst);
        if log::set_logger(&BRIDGE).is_ok() {
            log::set_max_level(LevelFilter::Info);
        }
    }

    pub fn uninstall() {
        BRIDGE.sink.store(0, Ordering::SeqCst);
    }

    fn level_code(level: Level) -> u8 {
        match level {
            Level::Error => 1,
            Level::Warn => 2,
            Level::Info => 3,
            Level::Debug => 4,
            Level::Trace => 5,
        }
    }
}

impl Log for LogBridge {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        self.sink.load(Ordering::SeqCst) != 0
    }

    fn log(&self, record: &Record<'_>) {
        let address = self.sink.load(Ordering::SeqCst);
        if address == 0 {
            return;
        }
        // SAFETY: a non-zero address was stored from a valid `extern "C" fn` of exactly this signature in `install`
        let sink: extern "C" fn(u8, *const u8, usize) = unsafe { std::mem::transmute(address) };
        let message = record.args().to_string();
        sink(Self::level_code(record.level()), message.as_ptr(), message.len());
    }

    fn flush(&self) {}
}
