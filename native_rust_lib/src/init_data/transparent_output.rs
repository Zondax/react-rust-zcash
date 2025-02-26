use std::ffi::c_char;

#[repr(C)]
pub struct CToutData {
    pub(crate) address: *const c_char, // Hex-encoded string representing a Script
    pub(crate) value: u64,             // u64 value representing an Amount
}
