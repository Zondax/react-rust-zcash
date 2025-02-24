use ledger_chain_builder::hsmauth::MixedAuthorization;
use ledger_chain_builder::{hsmauth, txbuilder};
use once_cell::sync::Lazy;
use rand_core::OsRng;
use std::collections::HashMap;
use std::sync::Mutex;
use zcash_primitives::consensus::{MainNetwork, TestNetwork};
use zcash_primitives::transaction::{
    components::{sapling, transparent, TxOut},
    Authorization,
};

use crate::{
    NetworkType, TransactionSignatures, TransparentInputInfo, TransparentOutputInfo, ZcashError,
};

// Enum to store builders with different network types
enum NetworkBuilder {
    Mainnet(txbuilder::Builder<MainNetwork, OsRng, hsmauth::Unauthorized>),
    Testnet(txbuilder::Builder<TestNetwork, OsRng, hsmauth::Unauthorized>),

    // Fully authorized builders
    MainnetAuthorized(
        txbuilder::Builder<
            MainNetwork,
            OsRng,
            hsmauth::MixedAuthorization<transparent::Authorized, sapling::Authorized>,
        >,
    ),
    TestnetAuthorized(
        txbuilder::Builder<
            TestNetwork,
            OsRng,
            hsmauth::MixedAuthorization<transparent::Authorized, sapling::Authorized>,
        >,
    ),
}
// Global storage to keep track of builder instances
static BUILDERS: Lazy<Mutex<HashMap<u64, NetworkBuilder>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static NEXT_ID: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

#[no_mangle]
pub extern "C" fn create_builder(fee: u64, network_type: u8, id: &mut u64) -> u32 {
    let network = match NetworkType::try_from(network_type) {
        Ok(NetworkType::Mainnet) => {
            let builder = txbuilder::Builder::new_with_fee(MainNetwork, 0, fee);
            NetworkBuilder::Mainnet(builder)
        }
        Ok(NetworkType::Testnet) => {
            let builder = txbuilder::Builder::new_with_fee(TestNetwork, 0, fee);
            NetworkBuilder::Testnet(builder)
        }
        Err(e) => {
            return e as u32;
        }
    };

    let mut next_id = NEXT_ID.lock().unwrap();
    *id = *next_id;
    *next_id += 1;

    let mut builders = BUILDERS.lock().unwrap();
    builders.insert(*id, network);

    ZcashError::Success as u32
}

#[no_mangle]
pub extern "C" fn destroy_builder(builder_id: u64) -> u32 {
    let mut builders = BUILDERS.lock().unwrap();
    if builders.remove(&builder_id).is_some() {
        ZcashError::Success as u32
    } else {
        ZcashError::BuilderNotFound as u32
    }
}

#[no_mangle]
pub extern "C" fn add_transparent_input(builder_id: u64, input: TransparentInputInfo) -> u32 {
    let mut builders = BUILDERS.lock().unwrap();

    if let Some(builder) = builders.get_mut(&builder_id) {
        // Validate pointers
        if input.any_null() {
            return ZcashError::InvalidArgument as u32;
        }

        // Parse using our helper functions
        let outpoint = match input.outpoint() {
            Ok(outpoint) => outpoint,
            Err(e) => return e as u32,
        };

        let pubkey = match input.public_key() {
            Ok(pubkey) => pubkey,
            Err(e) => return e as u32,
        };

        let script = match input.address() {
            Ok(script) => script,
            Err(e) => return e as u32,
        };

        let amount = match input.amount() {
            Ok(amount) => amount,
            Err(e) => return e as u32,
        };

        // Create TxOut
        let tx_out = TxOut {
            value: amount,
            script_pubkey: script,
        };

        // Add transparent input based on network type
        let result = match builder {
            NetworkBuilder::Mainnet(builder) => {
                builder.add_transparent_input(pubkey, outpoint, tx_out)
            }
            NetworkBuilder::Testnet(builder) => {
                builder.add_transparent_input(pubkey, outpoint, tx_out)
            }
            _ => return ZcashError::AlreadyAuthorized as u32,
        };

        match result {
            Ok(_) => ZcashError::Success as u32,
            Err(e) => {
                let error = ZcashError::from(e);
                error as u32
            }
        }
    } else {
        ZcashError::BuilderNotFound as u32
    }
}

#[no_mangle]
pub extern "C" fn add_transparent_output(builder_id: u64, output: TransparentOutputInfo) -> u32 {
    let mut builders = BUILDERS.lock().unwrap();

    if let Some(builder) = builders.get_mut(&builder_id) {
        // Validate pointers
        if output.any_null() {
            return ZcashError::InvalidArgument as u32;
        }

        // Convert C string to Rust string

        // Parse using our helper functions
        let script = match output.address() {
            Ok(script) => script,
            Err(e) => return e as u32,
        };

        let amount = match output.amount() {
            Ok(amount) => amount,
            Err(e) => return e as u32,
        };

        // Add transparent output based on network type
        let result = match builder {
            NetworkBuilder::Mainnet(builder) => builder.add_transparent_output(script, amount),
            NetworkBuilder::Testnet(builder) => builder.add_transparent_output(script, amount),
            _ => return ZcashError::AlreadyAuthorized as u32,
        };

        // Return error code
        match result {
            Ok(_) => ZcashError::Success as u32,
            Err(e) => {
                let error = ZcashError::from(e);
                error as u32
            }
        }
    } else {
        ZcashError::BuilderNotFound as u32
    }
}

#[no_mangle]
pub extern "C" fn add_signatures(builder_id: u64, signatures: TransactionSignatures) -> u32 {
    let mut builders = BUILDERS.lock().unwrap();

    if let Some(builder) = builders.remove(&builder_id) {
        // Parse transparent signatures
        let Ok(transparent_signatures) = signatures.transparent_sigs() else {
            return ZcashError::InvalidArgument as u32;
        };

        // Sapling signatures are empty for now
        let sapling_signatures = vec![];

        // Handle different builder states
        let result = match builder {
            // Basic unauthorized builders - do sapling signatures first, then transparent
            NetworkBuilder::Mainnet(builder) => {
                // 1. Add sapling signatures first
                let builder_sapling_authorized =
                    match builder.add_signatures_spend(sapling_signatures) {
                        Ok(authorized) => authorized,
                        Err(e) => {
                            let error = ZcashError::from(e);
                            return error as _;
                        }
                    };

                // 2. Add transparent signatures after
                let builder_fully_authorized = match builder_sapling_authorized
                    .add_signatures_transparent(transparent_signatures)
                {
                    Ok(authorized) => authorized,
                    Err(e) => {
                        let error = ZcashError::from(e);
                        return error as _;
                    }
                };

                // Store the fully authorized builder back
                builders.insert(
                    builder_id,
                    NetworkBuilder::MainnetAuthorized(builder_fully_authorized),
                );
                ZcashError::Success
            }
            NetworkBuilder::Testnet(builder) => {
                // 1. Add sapling signatures first
                let builder_sapling_authorized =
                    match builder.add_signatures_spend(sapling_signatures) {
                        Ok(authorized) => authorized,
                        Err(e) => {
                            let error = ZcashError::from(e);
                            return error as _;
                        }
                    };

                // 2. Add transparent signatures after
                let builder_fully_authorized = match builder_sapling_authorized
                    .add_signatures_transparent(transparent_signatures)
                {
                    Ok(authorized) => authorized,
                    Err(e) => {
                        let error = ZcashError::from(e);
                        return error as _;
                    }
                };

                // Store the fully authorized builder back
                builders.insert(
                    builder_id,
                    NetworkBuilder::TestnetAuthorized(builder_fully_authorized),
                );
                ZcashError::Success
            }

            // For already authorized builders
            NetworkBuilder::MainnetAuthorized(_) | NetworkBuilder::TestnetAuthorized(_) => {
                // Can't add signatures to already authorized builder
                return ZcashError::AlreadyAuthorized as u32;
            }
        };

        result as u32
    } else {
        ZcashError::BuilderNotFound as u32
    }
}

// #[no_mangle]
// pub extern "C" fn finalize_transaction(
//     builder_id: u64,
//     result_ptr: *mut *mut u8,
//     result_len: *mut usize,
// ) -> u32 {
//     let mut builders = BUILDERS.lock().unwrap();
//
//     if let Some(builder) = builders.remove(&builder_id) {
//         // Add transparent signatures (this is a simplified version)
//         // In a real implementation, you'd need to handle signatures properly
//         let result = match builder {
//             NetworkBuilder::Mainnet(builder) => {
//                 let builder_authorized = builder.add_signatures_transparent(vec![]);
//                 match builder_authorized {
//                     Ok(authorized_builder) => authorized_builder.finalize_js(),
//                     Err(e) => Err(e),
//                 }
//             }
//             NetworkBuilder::Testnet(builder) => {
//                 let builder_authorized = builder.add_signatures_transparent(vec![]);
//                 match builder_authorized {
//                     Ok(authorized_builder) => authorized_builder.finalize_js(),
//                     Err(e) => Err(e),
//                 }
//             }
//         };
//
//         match result {
//             Ok(tx_bytes) => {
//                 unsafe {
//                     // Allocate memory for the result
//                     let buffer =
//                         libc::malloc(tx_bytes.len() * std::mem::size_of::<u8>()) as *mut u8;
//                     if buffer.is_null() {
//                         return ZcashError::ReadWriteError as u32;
//                     }
//
//                     // Copy the transaction bytes to the allocated memory
//                     std::ptr::copy_nonoverlapping(tx_bytes.as_ptr(), buffer, tx_bytes.len());
//
//                     // Set output parameters
//                     *result_ptr = buffer;
//                     *result_len = tx_bytes.len();
//                 }
//                 ZcashError::Success as u32
//             }
//             Err(e) => {
//                 let error = ZcashError::from(e);
//                 error as u32
//             }
//         }
//     } else {
//         ZcashError::BuilderNotFound as u32
//     }
// }
