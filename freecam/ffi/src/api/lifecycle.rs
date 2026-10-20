use std::path::PathBuf;

use freecam_core::config::ConfigLoader;

use crate::services::{FcBoundary, FcHandle, LogBridge};
use crate::types::{FcLogFn, FcStatus, FcStr};

// returns null when the path is null or not UTF-8; a missing config file gives the defaults
#[unsafe(no_mangle)]
pub extern "C" fn fc_create(config_file: *const FcStr, log: FcLogFn) -> *mut FcHandle {
    LogBridge::install(log);
    FcBoundary::guard_value(std::ptr::null_mut(), || {
        // SAFETY: the shim passes a pointer to an FcStr it owns for the call, or null
        let Some(path) = (unsafe { FcBoundary::input(config_file) }) else { return std::ptr::null_mut() };
        // SAFETY: the FcStr points to bytes the shim owns for the call
        let Some(path) = (unsafe { FcBoundary::text(path) }) else { return std::ptr::null_mut() };
        let config = ConfigLoader::load(&PathBuf::from(path));
        Box::into_raw(Box::new(FcHandle::new(config)))
    })
}

/// Frees the handle; the handle must not be used again. Every buffer borrowed from fc_ui_frame, fc_frame
/// or fc_markers_shutdown is invalid after this call.
#[unsafe(no_mangle)]
pub extern "C" fn fc_destroy(handle: *mut FcHandle) {
    let _ = FcBoundary::guard(|| {
        if handle.is_null() {
            return FcStatus::InvalidArgument;
        }
        // SAFETY: a non-null handle came from fc_create and is destroyed exactly once
        drop(unsafe { Box::from_raw(handle) });
        FcStatus::Ok
    });
    LogBridge::uninstall();
}
