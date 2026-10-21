use curia::{Fields, Level, LogEvent, Logger};

use crate::services::FcBoundary;
use crate::types::{FcStatus, FcStr};

/// Logs one line from the shim through Curia, which forwards it to the mod manager's log for freecam. `level` is 0
/// error, 1 warn, 2 info, 3 debug or 4 trace. `message` and `file` are UTF-8, not null-terminated and borrowed for
/// the call; `file` may be null with a length of 0. Any thread. Before fc_create succeeds the line goes nowhere. A
/// null or non-UTF-8 message, or an unknown level, gives FC_STATUS_INVALID_ARGUMENT.
#[unsafe(no_mangle)]
pub extern "C" fn fc_log(
    level: u32,
    message: *const u8,
    message_len: usize,
    file: *const u8,
    file_len: usize,
    line: u32,
) -> FcStatus {
    FcBoundary::guard(|| {
        let level = match level {
            0 => Level::Error,
            1 => Level::Warn,
            2 => Level::Info,
            3 => Level::Debug,
            4 => Level::Trace,
            _ => return FcStatus::InvalidArgument,
        };
        if message.is_null() {
            return FcStatus::InvalidArgument;
        }
        // SAFETY: the shim passes message_len readable bytes it owns for the call
        let Some(text) = (unsafe { FcBoundary::text(&FcStr { ptr: message, len: message_len }) }) else {
            return FcStatus::InvalidArgument;
        };
        // SAFETY: the shim passes file_len readable bytes it owns for the call, or null with a length of 0
        let file = unsafe { FcBoundary::text(&FcStr { ptr: file, len: file_len }) }.filter(|file| !file.is_empty());
        if Logger::enabled(level, "alaydriem_freecam::shim") {
            Logger::emit(LogEvent {
                level,
                target: "alaydriem_freecam::shim".to_owned(),
                message: text,
                fields: Fields::new(),
                timestamp: chrono::Utc::now(),
                file,
                line: Some(line),
            });
        }
        FcStatus::Ok
    })
}
