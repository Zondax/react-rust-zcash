use std::ffi::c_char;

#[repr(C)]
pub struct CTinData {
    pub(crate) path: *const u32,       // Pointer to array of 5 u32 values
    pub(crate) path_len: usize,        // Should be 5
    pub(crate) address: *const c_char, // Hex-encoded string representing a Script
    pub(crate) value: u64,             // u64 value representing an Amount
}
