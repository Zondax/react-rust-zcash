use std::{
    ffi::{c_char, CStr},
    str::FromStr,
};

use zcash_primitives::{
    legacy::Script,
    transaction::components::{Amount, OutPoint},
};

use crate::{
    parser::{parse_outpoint, parse_public_key, parse_script},
    ZcashError,
};

// C-compatible structs for transparent operations
#[repr(C)]
pub struct TransparentInputInfo {
    outp: *const c_char,
    pk: *const c_char,
    address: *const c_char,
    value: u64,
}

impl TransparentInputInfo {
    pub fn from_raw(
        outp: *const c_char,
        pk: *const c_char,
        address: *const c_char,
        value: u64,
    ) -> Self {
        Self {
            outp,
            pk,
            address,
            value,
        }
    }

    pub fn outpoint(&self) -> Result<OutPoint, ZcashError> {
        unsafe {
            let outp_str = CStr::from_ptr(self.outp)
                .to_str()
                .map_err(|_| ZcashError::InvalidArgument)?;

            parse_outpoint(outp_str)
        }
    }

    pub fn public_key(&self) -> Result<secp256k1::PublicKey, ZcashError> {
        unsafe {
            let pk_str = CStr::from_ptr(self.pk)
                .to_str()
                .map_err(|_| ZcashError::InvalidArgument)?;

            parse_public_key(pk_str)
        }
    }

    pub fn address(&self) -> Result<Script, ZcashError> {
        unsafe {
            let address_str = CStr::from_ptr(self.address)
                .to_str()
                .map_err(|_| ZcashError::InvalidArgument)?;
            parse_script(address_str)
        }
    }

    pub fn amount(&self) -> Result<Amount, ZcashError> {
        Amount::from_u64(self.value).map_err(|_| ZcashError::InvalidArgument)
    }

    pub fn any_null(&self) -> bool {
        self.outp.is_null() || self.pk.is_null() || self.address.is_null()
    }
}
