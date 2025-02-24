use ledger_chain_builder::{hsmauth, txbuilder};
use once_cell::sync::Lazy;
use rand_core::OsRng;
use std::collections::HashMap;
use std::sync::Mutex;
use zcash_primitives::consensus::{MainNetwork, TestNetwork};
use zcash_primitives::transaction::components::TxOut;

use crate::{NetworkType, TransparentInputInfo, TransparentOutputInfo, ZcashError};

// Enum to store builders with different network types
enum NetworkBuilder {
    Mainnet(txbuilder::Builder<MainNetwork, OsRng, hsmauth::Unauthorized>),
    Testnet(txbuilder::Builder<TestNetwork, OsRng, hsmauth::Unauthorized>),
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
