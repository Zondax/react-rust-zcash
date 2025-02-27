mod builder;
mod error;
mod init_data;
pub(crate) mod memory;
mod network;
mod parser;
mod signatures;
mod transparent_input;
mod transparent_output;

// #[cfg(target_os = "android")]
#[cfg(any(target_os = "android", test))]
mod android;
#[cfg(target_os = "android")]
pub use android::*;
#[cfg(test)]
pub use android::*;

pub use builder::{
    add_signatures, add_transparent_input, add_transparent_output, build_transaction,
    create_builder, destroy_builder,
};
pub use error::ZcashError;
pub use init_data::{CInitData, CSaplingInData, CSaplingOutData, CTinData, CToutData};
pub use memory::free_transaction_data;
pub use network::NetworkType;
pub use signatures::TransactionSignatures;
pub use transparent_input::TransparentInputInfo;
pub use transparent_output::TransparentOutputInfo;

#[no_mangle]
pub extern "C" fn calculate_fee(n_tin: usize, n_tout: usize, n_spend: usize, n_sout: usize) -> u64 {
    ledger_app_builder::builder::Builder::calculate_zip0317_fee(n_tin, n_tout, n_spend, n_sout)
        .into()
}
