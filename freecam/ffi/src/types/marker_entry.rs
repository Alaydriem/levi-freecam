use freecam_core::structs::{MarkerEntry, MarkerKind};

use crate::types::FcVec3;

// one slot change for the client: draw or move the shape under `id`, or remove it when `kind` is 0
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FcMarkerEntry {
    pub id: u64,
    /// 0 remove (only `id` is used), 1 line, 2 box, 3 arrow
    pub kind: u8,
    /// start of a line or arrow, the minimum corner of a box
    pub location: FcVec3,
    /// end of a line or arrow, the size of a box
    pub end: FcVec3,
    /// ARGB
    pub color: u32,
    pub dimension: i32,
}

impl From<&MarkerEntry> for FcMarkerEntry {
    fn from(value: &MarkerEntry) -> Self {
        let kind = match value.shape.kind {
            MarkerKind::Remove => 0,
            MarkerKind::Line => 1,
            MarkerKind::Box => 2,
            MarkerKind::Arrow => 3,
        };
        Self {
            id: value.id,
            kind,
            location: FcVec3::from(&value.shape.location),
            end: FcVec3::from(&value.shape.end),
            color: value.shape.color,
            dimension: value.dimension,
        }
    }
}
