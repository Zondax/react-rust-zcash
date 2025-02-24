use std::str::FromStr;

use zcash_primitives::{
    legacy::Script,
    transaction::components::{Amount, OutPoint},
};

use crate::ZcashError;
// Helper functions to parse the data using the same logic as deserializers
pub(crate) fn parse_outpoint(hex_str: &str) -> Result<OutPoint, ZcashError> {
    let mut bytes = vec![0u8; 36];
    hex::decode_to_slice(hex_str, &mut bytes).map_err(|_| ZcashError::InvalidArgument)?;

    OutPoint::read(&bytes[..]).map_err(|_| ZcashError::InvalidOutpoint)
}

pub(crate) fn parse_public_key(key_str: &str) -> Result<secp256k1::PublicKey, ZcashError> {
    secp256k1::PublicKey::from_str(key_str).map_err(|_| ZcashError::InvalidArgument)
}

pub(crate) fn parse_script(hex_str: &str) -> Result<Script, ZcashError> {
    let bytes = hex::decode(hex_str).map_err(|_| ZcashError::InvalidArgument)?;

    Script::read(&bytes[..]).map_err(|_| ZcashError::InvalidScript)
}

pub(crate) fn parse_amount(value: u64) -> Result<Amount, ZcashError> {
    Amount::from_u64(value).map_err(|_| ZcashError::InvalidAmount)
}
