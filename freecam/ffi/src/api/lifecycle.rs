use std::path::Path;

use amm_client::link::ApiTable;

use crate::services::{FcBoundary, FcHandle};
use crate::types::{FcStatus, FcStr};

/// Registers freecam with the Alaydriem Mod Manager and returns the handle. `config_dir` is the mod's own LeviLamina
/// config folder, an absolute UTF-8 path, borrowed for the call; the manager keeps freecam's `config.hcl` there. Call
/// it from the game thread in the mod's enable, never in load. Returns null when the path is null or not UTF-8, when
/// the mod manager is not loaded, or when it refuses the registration; nothing is logged then, so the caller logs one
/// line with its own logger and installs no hooks.
#[unsafe(no_mangle)]
pub extern "C" fn fc_create(config_dir: *const FcStr) -> *mut FcHandle {
    FcBoundary::guard_value(std::ptr::null_mut(), || {
        // SAFETY: the shim passes a pointer to an FcStr it owns for the call, or null
        let Some(path) = (unsafe { FcBoundary::input(config_dir) }) else { return std::ptr::null_mut() };
        // SAFETY: the FcStr points to bytes the shim owns for the call
        let Some(path) = (unsafe { FcBoundary::text(path) }) else { return std::ptr::null_mut() };
        let Ok(table) = ApiTable::resolve() else { return std::ptr::null_mut() };
        match FcHandle::create(table, Path::new(&path)) {
            Ok(handle) => Box::into_raw(Box::new(handle)),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

/// Unregisters from the mod manager and frees the handle; the handle must not be used again. Every buffer borrowed
/// from fc_frame or fc_markers_shutdown is invalid after this call. Unhook first, so no call can enter Rust during it.
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
}
