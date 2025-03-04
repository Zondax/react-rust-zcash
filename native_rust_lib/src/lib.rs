mod deserializer;
mod error;
mod ffi;
mod init_data;
mod network;
mod parser;

mod signatures;

#[cfg(test)]
mod test;
#[cfg(test)]
pub(crate) use test::*;

mod transparent_input;
mod transparent_output;
pub use deserializer::{deserialize_cstring, deserialize_cstring_vec};

#[cfg(any(target_os = "android", test))]
mod android;
#[cfg(target_os = "android")]
pub use android::*;
#[cfg(test)]
pub use android::*;

pub use error::ZcashError;
pub use ffi::{
    add_signatures, add_transparent_input, add_transparent_output, build_transaction,
    calculate_fee, create_builder, destroy_builder, free_error_description, free_transaction_data,
};
pub use init_data::{CInitData, CSaplingInData, CSaplingOutData, CTinData, CToutData};
pub use network::NetworkType;
pub use signatures::Signatures;
pub use transparent_input::TransparentInput;
pub use transparent_output::TransparentOutput;
