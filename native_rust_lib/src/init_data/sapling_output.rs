use std::ffi::{c_char, c_uchar};

#[repr(C)]
pub struct CSaplingOutData {
    pub(crate) address: *const c_char, // Hex-encoded string representing a PaymentAddress
    pub(crate) value: u64,             // u64 value representing an Amount
    pub(crate) memo_type: u8,          // Single byte value
    pub(crate) has_ovk: bool,          // Boolean indicating if ovk is present
    pub(crate) ovk: *const c_uchar, // Optional hex-encoded string representing an OutgoingViewingKey
    pub(crate) ovk_len: usize,      // Should be 32 if ovk is present, 0 otherwise
}
