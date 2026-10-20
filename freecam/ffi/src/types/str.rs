// UTF-8 bytes, not null-terminated; each function states who owns them and for how long. ptr may be null when len is 0
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FcStr {
    pub ptr: *const u8,
    pub len: usize,
}
