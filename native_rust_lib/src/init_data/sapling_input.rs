use std::ffi::c_char;

#[repr(C)]
pub struct CSaplingInData {
    pub(crate) path: u32,              // Single u32 value
    pub(crate) address: *const c_char, // Hex-encoded string representing a PaymentAddress
    pub(crate) value: u64,             // u64 value representing an Amount
}
