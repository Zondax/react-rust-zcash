use ledger_chain_builder::{hsmauth, txbuilder, txprover};
use log::{error, info};
use once_cell::sync::Lazy;
use rand_core::OsRng;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::path::Path;
use std::sync::Mutex;
use zcash_primitives::consensus::{self, MainNetwork, TestNetwork};
use zcash_primitives::transaction::components::{sapling, transparent, TxOut};
use zcash_primitives::transaction::TxVersion;

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
            NetworkBuilder::TestnetAuthorized(..) | NetworkBuilder::MainnetAuthorized(..) => {
                return ZcashError::AlreadyAuthorized as u32
            }
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

#[no_mangle]
pub extern "C" fn build_transaction(
    builder_id: u64,
    spend_path: *const c_char,
    output_path: *const c_char,
    tx_version: u8,
    result_ptr: *mut *mut u8,
    result_len: *mut usize,
) -> u32 {
    info!("Building transaction for builder: {}", builder_id);
    // Validate input pointers
    if spend_path.is_null() || output_path.is_null() {
        error!("Build parameters are null");
        return ZcashError::InvalidArgument as u32;
    }

    // Convert C strings to Rust strings
    let spend_path_str = unsafe {
        match CStr::from_ptr(spend_path).to_str() {
            Ok(s) => s,
            Err(_) => return ZcashError::InvalidArgument as u32,
        }
    };

    let output_path_str = unsafe {
        match CStr::from_ptr(output_path).to_str() {
            Ok(s) => s,
            Err(_) => return ZcashError::InvalidArgument as u32,
        }
    };
    info!("MyRustModule: spend path: {}", spend_path_str);
    info!("MyRustModule: output path: {}", output_path_str);

    // Read file contents
    let spend_params_bytes = match std::fs::read(spend_path_str) {
        Ok(bytes) => {
            info!("Successfully read spend params: {} bytes", bytes.len());
            bytes
        }
        Err(e) => {
            error!("Failed to read spend params file: {}", e);
            return ZcashError::InvalidArgument as u32;
        }
    };

    let output_params_bytes = match std::fs::read(output_path_str) {
        Ok(bytes) => {
            info!("Successfully read output params: {} bytes", bytes.len());
            bytes
        }
        Err(e) => {
            error!("Failed to read output params file: {}", e);
            return ZcashError::InvalidArgument as u32;
        }
    };

    info!("MyRustModule: tx version: {}", tx_version);

    // Parse tx_version
    let tx_ver = match tx_version {
        4 => Some(TxVersion::Sapling),
        5 => Some(TxVersion::Zip225),
        _ => None,
    };

    let mut builders = BUILDERS.lock().unwrap();

    if let Some(builder) = builders.remove(&builder_id) {
        info!("Got builder from list");
        // Handle based on builder state
        let result = match builder {
            // Only unauthorized builders can be built
            NetworkBuilder::Mainnet(mut builder) => {
                info!("building mainnet");

                // Use from_bytes method with catch_unwind for safety
                let prover_result = std::panic::catch_unwind(|| {
                    info!("calling prover::from_bytes");
                    txprover::LocalTxProver::from_bytes(&spend_params_bytes, &output_params_bytes)
                });
                info!("prover created");
                println!("prover created");

                let mut prover = match prover_result {
                    Ok(prover) => {
                        info!("prover created successfully");
                        prover
                    }
                    Err(e) => {
                        // Try to extract panic message
                        let panic_msg = if let Some(s) = e.downcast_ref::<String>() {
                            s.clone()
                        } else if let Some(s) = e.downcast_ref::<&str>() {
                            s.to_string()
                        } else {
                            "Unknown panic in prover creation".to_string()
                        };

                        error!("Prover creation failed with panic: {}", panic_msg);
                        return ZcashError::InternalError as u32;
                    }
                };

                let build_result = builder.build(consensus::BranchId::Nu6, tx_ver, &mut prover);

                match build_result {
                    Ok(hsm_data) => {
                        info!("hsm_data OK");
                        // Put the builder back in the map
                        builders.insert(builder_id, NetworkBuilder::Mainnet(builder));

                        // Get the bytes from HsmTxData
                        match hsm_data.to_hsm_bytes() {
                            Ok(bytes) => {
                                unsafe {
                                    // Allocate memory for the result
                                    info!("allocating memory");
                                    let buffer =
                                        libc::malloc(bytes.len() * std::mem::size_of::<u8>())
                                            as *mut u8;
                                    if buffer.is_null() {
                                        error!(
                                            "buffer is null - could not allocate: {}",
                                            bytes.len()
                                        );
                                        return ZcashError::ReadWriteError as u32;
                                    }

                                    // Copy the transaction bytes to the allocated memory
                                    std::ptr::copy_nonoverlapping(
                                        bytes.as_ptr(),
                                        buffer,
                                        bytes.len(),
                                    );

                                    // Set output parameters
                                    *result_ptr = buffer;
                                    *result_len = bytes.len();
                                }
                                info!("success");
                                ZcashError::Success as u32
                            }
                            Err(e) => {
                                error!("Error: {:?}", e);
                                ZcashError::ReadWriteError as u32
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        let error = ZcashError::from(e);
                        error as u32
                    }
                }
            }
            NetworkBuilder::Testnet(mut builder) => {
                // Similar implementation for testnet
                // ...

                // For now, just a placeholder to avoid leaving incomplete code
                error!("Testnet implementation not completed");
                ZcashError::InvalidArgument as u32
            }
            // Already authorized builders can't be built
            NetworkBuilder::MainnetAuthorized(_) | NetworkBuilder::TestnetAuthorized(_) => {
                error!("Already authorized");
                return ZcashError::AlreadyAuthorized as u32;
            }
        };

        result
    } else {
        ZcashError::BuilderNotFound as u32
    }
}

// #[no_mangle]
// pub extern "C" fn build_transaction_with_bytes(
//     builder_id: u64,
//     spend_params_ptr: *const u8,
//     spend_params_len: usize,
//     output_params_ptr: *const u8,
//     output_params_len: usize,
//     tx_version: u8,
//     result_ptr: *mut *mut u8,
//     result_len: *mut usize,
// ) -> u32 {
//     info!(
//         "Building transaction with bytes for builder: {}",
//         builder_id
//     );
//
//     // Validate input pointers
//     if spend_params_ptr.is_null() || output_params_ptr.is_null() {
//         error!("Build parameters are null");
//         return ZcashError::InvalidArgument as u32;
//     }
//
//     // Create slices from the raw pointers
//     let spend_params_bytes =
//         unsafe { std::slice::from_raw_parts(spend_params_ptr, spend_params_len) };
//     let output_params_bytes =
//         unsafe { std::slice::from_raw_parts(output_params_ptr, output_params_len) };
//
//     info!(
//         "Got parameter bytes: spend={} bytes, output={} bytes",
//         spend_params_len, output_params_len
//     );
//
//     // Parse tx_version
//     let tx_ver = match tx_version {
//         4 => Some(TxVersion::Sapling),
//         5 => Some(TxVersion::Zip225),
//         _ => None,
//     };
//
//     let mut builders = BUILDERS.lock().unwrap();
//
//     if let Some(builder) = builders.remove(&builder_id) {
//         info!("Got builder from list");
//
//         // Handle based on builder state
//         let result = match builder {
//             // Only unauthorized builders can be built
//             NetworkBuilder::Mainnet(mut builder) => {
//                 info!("building mainnet");
//
//                 // Use from_bytes method to create prover
//                 let prover_result = std::panic::catch_unwind(|| {
//                     txprover::LocalTxProver::from_bytes(spend_params_bytes, output_params_bytes)
//                 });
//
//                 let mut prover = match prover_result {
//                     Ok(prover) => {
//                         info!("prover created successfully");
//                         prover
//                     }
//                     Err(e) => {
//                         // Try to extract panic message
//                         let panic_msg = if let Some(s) = e.downcast_ref::<String>() {
//                             s.clone()
//                         } else if let Some(s) = e.downcast_ref::<&str>() {
//                             s.to_string()
//                         } else {
//                             "Unknown panic in prover creation".to_string()
//                         };
//
//                         error!("Prover creation failed with panic: {}", panic_msg);
//                         return ZcashError::InternalError as u32;
//                     }
//                 };
//
//                 let build_result = builder.build(consensus::BranchId::Nu6, tx_ver, &mut prover);
//
//                 match build_result {
//                     Ok(hsm_data) => {
//                         info!("hsm_data OK");
//                         // Put the builder back in the map
//                         builders.insert(builder_id, NetworkBuilder::Mainnet(builder));
//
//                         // Get the bytes from HsmTxData
//                         match hsm_data.to_hsm_bytes() {
//                             Ok(bytes) => {
//                                 unsafe {
//                                     // Allocate memory for the result
//                                     info!("allocating memory");
//                                     let buffer =
//                                         libc::malloc(bytes.len() * std::mem::size_of::<u8>())
//                                             as *mut u8;
//                                     if buffer.is_null() {
//                                         error!(
//                                             "buffer is null - could not allocate: {}",
//                                             bytes.len()
//                                         );
//                                         return ZcashError::ReadWriteError as u32;
//                                     }
//
//                                     // Copy the transaction bytes to the allocated memory
//                                     std::ptr::copy_nonoverlapping(
//                                         bytes.as_ptr(),
//                                         buffer,
//                                         bytes.len(),
//                                     );
//
//                                     // Set output parameters
//                                     *result_ptr = buffer;
//                                     *result_len = bytes.len();
//                                 }
//                                 info!("success");
//                                 ZcashError::Success as u32
//                             }
//                             Err(e) => {
//                                 error!("Error: {:?}", e);
//                                 ZcashError::ReadWriteError as u32
//                             }
//                         }
//                     }
//                     Err(e) => {
//                         error!("Error: {:?}", e);
//                         let error = ZcashError::from(e);
//                         error as u32
//                     }
//                 }
//             }
//             NetworkBuilder::Testnet(mut builder) => {
//                 // Similar implementation for testnet
//                 // ...
//                 ZcashError::InvalidArgument as u32 // Placeholder
//             }
//             // Already authorized builders can't be built
//             NetworkBuilder::MainnetAuthorized(_) | NetworkBuilder::TestnetAuthorized(_) => {
//                 error!("Already authorized");
//                 return ZcashError::AlreadyAuthorized as u32;
//             }
//         };
//
//         result
//     } else {
//         ZcashError::BuilderNotFound as u32
//     }
// }

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
#[cfg(test)]
mod test_builder {
    use super::*;
    use std::ffi::CString;
    use std::path::PathBuf;
    use std::ptr;

    #[test]
    fn test_transaction_flow() {
        // Step 1: Calculate fee (1 transparent input, 1 transparent output, 0 Sapling)
        let fee = crate::calculate_fee(1, 1, 0, 0);
        println!("Calculated fee: {}", fee);

        // Step 2: Create a builder
        let mut builder_id: u64 = 0;
        let create_result = create_builder(fee, NetworkType::Mainnet as u8, &mut builder_id);
        assert_eq!(create_result, ZcashError::Success as u32);
        println!("Created builder with ID: {}", builder_id);

        // Step 3: Add transparent input
        let outp_cstr = CString::new(
            "000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let pk_cstr =
            CString::new("031f6d238009787c20d5d7becb6b6ad54529fc0a3fd35088e85c2c3966bfec050e")
                .unwrap();
        let addr_in_cstr =
            CString::new("1976a9140f71709c4b828df00f93d20aa2c34ae987195b3388ac").unwrap();

        let input = TransparentInputInfo::from_raw(
            outp_cstr.as_ptr(),
            pk_cstr.as_ptr(),
            addr_in_cstr.as_ptr(),
            50000,
        );

        let add_input_result = add_transparent_input(builder_id, input);
        assert_eq!(add_input_result, ZcashError::Success as u32);
        println!("Added transparent input");

        // Step 4: Add transparent output
        let addr_out_cstr =
            CString::new("1976a914000000000000000000000000000000000000000088ac").unwrap();

        let output = TransparentOutputInfo::from_raw(
            addr_out_cstr.as_ptr(),
            10000, // 50000 - 10000 = 40000 (change/fee)
        );

        let add_output_result = add_transparent_output(builder_id, output);
        assert_eq!(add_output_result, ZcashError::Success as u32);
        println!("Added transparent output");

        // Step 5: Build the transaction
        // Find parameter files - first try current directory, then try relative to Cargo.toml
        let mut spend_path = PathBuf::from("sapling-spend.params");
        let mut output_path = PathBuf::from("sapling-output.params");

        if !spend_path.exists() {
            // Try to find in parent directories (up to 3 levels)
            for _ in 0..3 {
                spend_path = PathBuf::from("../").join(spend_path);
                output_path = PathBuf::from("../").join(output_path);
                if spend_path.exists() && output_path.exists() {
                    break;
                }
            }
        }

        println!("Using spend params at: {:?}", spend_path);
        println!("Using output params at: {:?}", output_path);

        assert!(spend_path.exists(), "Sapling spend params file not found");
        assert!(output_path.exists(), "Sapling output params file not found");

        // Print file sizes to verify they are correct
        let spend_size = std::fs::metadata(&spend_path).unwrap().len();
        let output_size = std::fs::metadata(&output_path).unwrap().len();
        println!("Spend params size: {} bytes", spend_size);
        println!("Output params size: {} bytes", output_size);

        // Convert paths to C strings
        let spend_path_cstr = CString::new(spend_path.to_str().unwrap()).unwrap();
        let output_path_cstr = CString::new(output_path.to_str().unwrap()).unwrap();

        let mut result_ptr: *mut u8 = ptr::null_mut();
        let mut result_len: usize = 0;

        let build_result = build_transaction(
            builder_id,
            spend_path_cstr.as_ptr(),
            output_path_cstr.as_ptr(),
            5, // Use Zip225 version
            &mut result_ptr,
            &mut result_len,
        );

        // Check if the transaction building was successful
        if build_result == ZcashError::Success as u32 {
            println!(
                "Successfully built transaction. Result length: {}",
                result_len
            );
            // Free the memory allocated by the build_transaction function
            if !result_ptr.is_null() {
                unsafe {
                    libc::free(result_ptr as *mut libc::c_void);
                }
            }
        } else {
            println!(
                "Failed to build transaction with error code: {}",
                build_result
            );
        }

        // Clean up
        let destroy_result = destroy_builder(builder_id);
        assert_eq!(destroy_result, ZcashError::Success as u32);
        println!("Destroyed builder");
    }

    // Helper functions
    fn assert_eq<T: std::cmp::PartialEq + std::fmt::Debug>(a: T, b: T) {
        if a != b {
            panic!("assertion failed: {:?} != {:?}", a, b);
        }
    }
}
