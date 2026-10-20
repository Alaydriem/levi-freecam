// level: 1 error, 2 warn, 3 info, 4 debug, 5 trace; message is UTF-8 and lives only for the call
pub type FcLogFn = Option<extern "C" fn(level: u8, message: *const u8, len: usize)>;
