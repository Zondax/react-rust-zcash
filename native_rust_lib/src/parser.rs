use std::str::FromStr;

use crate::ZcashError;
use zcash_primitives::{
    legacy::Script, sapling::PaymentAddress, transaction::components::OutPoint,
};

// Helper functions to parse the data using the same logic as deserializers
//
pub fn parse_outpoint(hex_str: &str) -> Result<OutPoint, ZcashError> {
    let mut bytes = vec![0u8; 36];
    hex::decode_to_slice(hex_str, &mut bytes).map_err(|_| ZcashError::InvalidArgument)?;

    OutPoint::read(&bytes[..]).map_err(|_| ZcashError::InvalidOutpoint)
}

pub fn parse_public_key(key_str: &str) -> Result<secp256k1::PublicKey, ZcashError> {
    secp256k1::PublicKey::from_str(key_str).map_err(|_| ZcashError::InvalidArgument)
}

pub fn parse_script(hex_str: &str) -> Result<Script, ZcashError> {
    let bytes = hex::decode(hex_str).map_err(|_| ZcashError::InvalidArgument)?;

    Script::read(&bytes[..]).map_err(|_| ZcashError::InvalidScript)
}

pub fn payment_address_from_hex(hex_str: &str) -> Result<PaymentAddress, ZcashError> {
    let mut bytes = [0u8; 43];

    // Decode hex string to bytes
    hex::decode_to_slice(hex_str, &mut bytes).map_err(|_| ZcashError::InvalidAddressFormat)?;

    // Convert bytes to PaymentAddress
    PaymentAddress::from_bytes(&bytes).ok_or(ZcashError::InvalidAddress)
}
