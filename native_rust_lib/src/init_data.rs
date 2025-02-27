use std::convert::TryFrom;
use std::ffi::CStr;
use std::slice;

use ledger_chain_builder::data::{InitData, SaplingInData, SaplingOutData, TinData, ToutData};
use log::{error, info};
use zcash_primitives::keys::OutgoingViewingKey;
use zcash_primitives::transaction::components::Amount;

use crate::parser::{parse_script, payment_address_from_hex};
use crate::ZcashError;

pub const PATH_LEN: usize = 5;

mod sapling_input;
mod sapling_output;
mod transparent_input;
mod transparent_output;

pub use sapling_input::CSaplingInData;
pub use sapling_output::CSaplingOutData;
pub use transparent_input::CTinData;
pub use transparent_output::CToutData;

pub extern "C" fn get_inittx_data(
    data: CInitData,
    result_ptr: *mut *mut u8,
    result_len: *mut u64,
) -> u32 {
    let init_data = match InitData::try_from(data) {
        Ok(data) => data,
        Err(e) => return e as u32,
    };

    let tx_data = init_data.to_hsm_bytes();
    let tx_data_len = tx_data.len();

    unsafe {
        info!("allocating memory for tx_data: {}", tx_data_len);
        let buffer = libc::malloc(tx_data_len * std::mem::size_of::<u8>()) as *mut u8;
        if buffer.is_null() {
            error!("buffer is null - could not allocate");
            return ZcashError::ReadWriteError as u32;
        }

        // Copy the transaction bytes to the allocated memory
        std::ptr::copy_nonoverlapping(tx_data.as_ptr(), buffer, tx_data_len);

        // Set output parameters
        *result_ptr = buffer;
        *result_len = tx_data_len as _;
    }

    ZcashError::Success as u32
}

#[repr(C)]
pub struct CInitData {
    pub(crate) t_in: *const CTinData,
    pub(crate) t_in_len: usize,
    pub(crate) t_out: *const CToutData,
    pub(crate) t_out_len: usize,
    pub(crate) s_spend: *const CSaplingInData,
    pub(crate) s_spend_len: usize,
    pub(crate) s_output: *const CSaplingOutData,
    pub(crate) s_output_len: usize,
}

impl TryFrom<&CTinData> for TinData {
    type Error = ZcashError;

    fn try_from(c_tin: &CTinData) -> Result<Self, Self::Error> {
        unsafe {
            // Check path length
            if c_tin.path.is_null() || c_tin.path_len != PATH_LEN {
                return Err(ZcashError::InvalidSpendPath);
            }

            let path_slice = slice::from_raw_parts(c_tin.path, PATH_LEN);
            let mut path = [0u32; PATH_LEN];
            for (i, val) in path_slice.iter().enumerate().take(PATH_LEN) {
                path[i] = *val;
            }

            // Handle address
            if c_tin.address.is_null() {
                return Err(ZcashError::InvalidAddress);
            }

            let address_cstr = CStr::from_ptr(c_tin.address);
            let address_str = address_cstr
                .to_str()
                .map_err(|_| ZcashError::InvalidAddressFormat)?;

            let address = parse_script(address_str)?;

            // Handle value
            let value =
                Amount::from_i64(c_tin.value as i64).map_err(|_| ZcashError::InvalidAmount)?;

            Ok(TinData {
                path,
                address,
                value,
            })
        }
    }
}

impl TryFrom<CTinData> for TinData {
    type Error = ZcashError;

    fn try_from(c_tin: CTinData) -> Result<Self, Self::Error> {
        Self::try_from(&c_tin)
    }
}

impl TryFrom<&CToutData> for ToutData {
    type Error = ZcashError;

    fn try_from(c_tout: &CToutData) -> Result<Self, Self::Error> {
        unsafe {
            // Handle address
            if c_tout.address.is_null() {
                return Err(ZcashError::InvalidAddress);
            }

            let address_cstr = CStr::from_ptr(c_tout.address);
            let address_str = address_cstr
                .to_str()
                .map_err(|_| ZcashError::InvalidAddressFormat)?;

            let address = parse_script(address_str)?;

            // Handle value
            let value =
                Amount::from_i64(c_tout.value as i64).map_err(|_| ZcashError::InvalidAmount)?;

            Ok(ToutData { address, value })
        }
    }
}

impl TryFrom<CToutData> for ToutData {
    type Error = ZcashError;

    fn try_from(c_tout: CToutData) -> Result<Self, Self::Error> {
        Self::try_from(&c_tout)
    }
}

impl TryFrom<&CSaplingInData> for SaplingInData {
    type Error = ZcashError;

    fn try_from(c_spend: &CSaplingInData) -> Result<Self, Self::Error> {
        unsafe {
            // Handle address
            if c_spend.address.is_null() {
                return Err(ZcashError::InvalidAddress);
            }

            let address_cstr = CStr::from_ptr(c_spend.address);
            let address_str = address_cstr
                .to_str()
                .map_err(|_| ZcashError::InvalidAddressFormat)?;

            let address = payment_address_from_hex(address_str)?;

            // Handle value
            let value =
                Amount::from_i64(c_spend.value as i64).map_err(|_| ZcashError::InvalidAmount)?;

            Ok(SaplingInData {
                path: c_spend.path,
                address,
                value,
            })
        }
    }
}

impl TryFrom<CSaplingInData> for SaplingInData {
    type Error = ZcashError;

    fn try_from(c_spend: CSaplingInData) -> Result<Self, Self::Error> {
        Self::try_from(&c_spend)
    }
}

impl TryFrom<&CSaplingOutData> for SaplingOutData {
    type Error = ZcashError;

    fn try_from(c_output: &CSaplingOutData) -> Result<Self, Self::Error> {
        unsafe {
            // Handle address
            if c_output.address.is_null() {
                return Err(ZcashError::InvalidAddress);
            }

            let address_cstr = CStr::from_ptr(c_output.address);
            let address_str = address_cstr
                .to_str()
                .map_err(|_| ZcashError::InvalidAddressFormat)?;

            let address = payment_address_from_hex(address_str)?;

            // Handle value
            let value =
                Amount::from_i64(c_output.value as i64).map_err(|_| ZcashError::InvalidAmount)?;

            // Handle OVK
            let ovk = if c_output.has_ovk {
                if c_output.ovk.is_null() {
                    return Err(ZcashError::InvalidOVKHashSeed);
                }

                if c_output.ovk_len != 32 {
                    return Err(ZcashError::InvalidOVKHashSeed);
                }

                let ovk_bytes = slice::from_raw_parts(c_output.ovk, 32);
                let mut key = [0u8; 32];
                key.copy_from_slice(ovk_bytes);
                Some(OutgoingViewingKey(key))
            } else {
                None
            };

            Ok(SaplingOutData {
                address,
                value,
                memo_type: c_output.memo_type,
                ovk,
            })
        }
    }
}

impl TryFrom<CSaplingOutData> for SaplingOutData {
    type Error = ZcashError;

    fn try_from(c_output: CSaplingOutData) -> Result<Self, Self::Error> {
        Self::try_from(&c_output)
    }
}

impl TryFrom<&CInitData> for InitData {
    type Error = ZcashError;

    fn try_from(c_init_data: &CInitData) -> Result<Self, Self::Error> {
        unsafe {
            // Validate array pointers
            if (c_init_data.t_in_len > 0 && c_init_data.t_in.is_null())
                || (c_init_data.t_out_len > 0 && c_init_data.t_out.is_null())
                || (c_init_data.s_spend_len > 0 && c_init_data.s_spend.is_null())
                || (c_init_data.s_output_len > 0 && c_init_data.s_output.is_null())
            {
                return Err(ZcashError::InvalidArgument);
            }

            // Convert C arrays to Rust vectors with error handling
            // let mut t_in = Vec::with_capacity(c_init_data.t_in_len);
            let t_in = if c_init_data.t_in_len > 0 {
                slice::from_raw_parts(c_init_data.t_in, c_init_data.t_in_len)
                    .iter()
                    .map(|c_tin| TinData::try_from(c_tin))
                    .collect::<Result<Vec<TinData>, ZcashError>>()?
            } else {
                Vec::new()
            };

            let t_out = if c_init_data.t_out_len > 0 {
                slice::from_raw_parts(c_init_data.t_out, c_init_data.t_out_len)
                    .iter()
                    .map(|c_tout| ToutData::try_from(c_tout))
                    .collect::<Result<Vec<ToutData>, ZcashError>>()?
            } else {
                Vec::new()
            };

            let s_spend = if c_init_data.s_spend_len > 0 {
                slice::from_raw_parts(c_init_data.s_spend, c_init_data.s_spend_len)
                    .iter()
                    .map(|c_spend| SaplingInData::try_from(c_spend))
                    .collect::<Result<Vec<SaplingInData>, ZcashError>>()?
            } else {
                Vec::new()
            };

            let s_output = if c_init_data.s_output_len > 0 {
                slice::from_raw_parts(c_init_data.s_output, c_init_data.s_output_len)
                    .iter()
                    .map(|c_output| SaplingOutData::try_from(c_output))
                    .collect::<Result<Vec<SaplingOutData>, ZcashError>>()?
            } else {
                Vec::new()
            };

            Ok(InitData {
                t_in,
                t_out,
                s_spend,
                s_output,
            })
        }
    }
}

impl TryFrom<CInitData> for InitData {
    type Error = ZcashError;

    fn try_from(c_init_data: CInitData) -> Result<Self, Self::Error> {
        Self::try_from(&c_init_data)
    }
}

// #[cfg(target_os = "android")]
// use jni::{
//     objects::{JObject, JString},
//     signature::{Primitive, ReturnType},
//     JNIEnv,
// };
// impl CTinData {
//     pub unsafe fn from_java(env: &mut JNIEnv, obj: JObject) -> Result<Self, ZcashError> {
//         use log::{error, info};
//     }
// }
