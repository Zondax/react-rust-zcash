use secp256k1::ecdsa::Signature;
use std::ffi::{c_char, CStr};

use crate::ZcashError;

// Structure for collecting signatures for a transaction
#[repr(C)]
pub struct TransactionSignatures {
    pub transparent_sigs: *const *const c_char,
    pub transparent_sigs_len: usize,
}

impl TransactionSignatures {
    pub fn transparent_sigs(&self) -> Result<Vec<Signature>, ZcashError> {
        let transparent_signatures =
            if self.transparent_sigs_len > 0 && !self.transparent_sigs.is_null() {
                let sigs_ptr = unsafe {
                    std::slice::from_raw_parts(self.transparent_sigs, self.transparent_sigs_len)
                };

                let mut parsed_sigs = Vec::with_capacity(sigs_ptr.len());
                for sig_ptr in sigs_ptr {
                    if sig_ptr.is_null() {
                        return Err(ZcashError::InvalidArgument);
                    }

                    let sig_str = unsafe {
                        CStr::from_ptr(*sig_ptr)
                            .to_str()
                            .map_err(|_| ZcashError::InvalidArgument)?
                    };

                    // Parse DER signature
                    let bytes = hex::decode(sig_str).map_err(|_| ZcashError::InvalidArgument)?;

                    let Ok(sig) = secp256k1::ecdsa::Signature::from_der(&bytes) else {
                        return Err(ZcashError::InvalidArgument);
                    };
                    parsed_sigs.push(sig);
                }
                parsed_sigs
            } else {
                vec![]
            };
        Ok(transparent_signatures)
    }
}
