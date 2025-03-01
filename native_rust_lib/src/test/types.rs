use std::ffi::CString;

use serde::{Deserialize, Serialize};

use super::{deserialize_cstring, deserialize_cstring_vec};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestData {
    pub tx_init_data: TxInitData,
    #[serde(deserialize_with = "deserialize_cstring")]
    pub ledgerblob_initdata: CString,
    pub tx_data: TxData,
    #[serde(deserialize_with = "deserialize_cstring")]
    pub finalized: CString,
    pub signatures: Signatures,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxInitData {
    pub t_in: Vec<TInput>,
    pub t_out: Vec<TOutput>,
    pub s_spend: Vec<SSpend>,
    pub s_output: Vec<SOutput>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TInput {
    pub path: Vec<u32>,
    #[serde(deserialize_with = "deserialize_cstring")]
    pub address: CString,
    pub value: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TOutput {
    #[serde(deserialize_with = "deserialize_cstring")]
    pub address: CString,
    pub value: u64,
}

// Empty structs for now, you can expand these as needed
#[derive(Debug, Serialize, Deserialize)]
pub struct SSpend {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SOutput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxData {
    pub transparent_inputs: Vec<TransparentInput>,
    pub transparent_outputs: Vec<TOutput>,
    pub spend_inputs: Vec<SpendInput>,
    pub spend_outputs: Vec<SpendOutput>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransparentInput {
    #[serde(deserialize_with = "deserialize_cstring")]
    pub outp: CString,
    #[serde(deserialize_with = "deserialize_cstring")]
    pub pk: CString,
    #[serde(deserialize_with = "deserialize_cstring")]
    pub address: CString,
    pub value: u64,
}

// Empty structs for now, you can expand these as needed
#[derive(Debug, Serialize, Deserialize)]
pub struct SpendInput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpendOutput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signatures {
    #[serde(deserialize_with = "deserialize_cstring_vec")]
    pub transparent_sigs: Vec<CString>,
    #[serde(deserialize_with = "deserialize_cstring_vec")]
    pub sapling_sigs: Vec<CString>,
}
