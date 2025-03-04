use std::ffi::{c_char, CStr};

use zcash_primitives::{legacy::Script, transaction::components::Amount};

use crate::{parser::parse_script, ZcashError};

#[repr(C)]
pub struct CTransparentOutput {
    address: *const c_char,
    value: u64,
}

impl CTransparentOutput {
    pub fn from_raw(address: *const c_char, value: u64) -> Self {
        Self { address, value }
    }
}

impl CTransparentOutput {
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
        self.address.is_null()
    }
}
