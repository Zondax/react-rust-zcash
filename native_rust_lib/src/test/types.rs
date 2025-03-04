use std::ffi::CString;

use serde::{Deserialize, Serialize};

use crate::{
    transparent_input::TransparentInput, transparent_output::TransparentOutput, Signatures,
};

use crate::deserializer::deserialize_cstring;

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

impl TestData {
    pub fn init_data_blob(&self) -> &str {
        &self.ledgerblob_initdata.to_str().unwrap()
    }

    pub fn finalized_tx_blob(&self) -> &str {
        &self.finalized.to_str().unwrap()
    }

    pub fn num_tinputs(&self) -> usize {
        self.tx_init_data.t_in.len()
    }

    pub fn num_toutputs(&self) -> usize {
        self.tx_init_data.t_out.len()
    }

    pub fn num_sspends(&self) -> usize {
        self.tx_init_data.s_spend.len()
    }

    pub fn num_soutputs(&self) -> usize {
        self.tx_init_data.s_output.len()
    }

    pub fn num_tsignatures(&self) -> usize {
        self.signatures.transparent_sigs.len()
    }

    pub fn transparent_sig(&self) -> &[CString] {
        &self.signatures.transparent_sigs
    }

    pub fn tinput_iter(&self) -> impl Iterator<Item = &TransparentInput> {
        self.tx_data.transparent_inputs.iter()
    }

    pub fn toutput_iter(&self) -> impl Iterator<Item = &TransparentOutput> {
        self.tx_data.transparent_outputs.iter()
    }
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
    pub transparent_outputs: Vec<TransparentOutput>,
    pub spend_inputs: Vec<SpendInput>,
    pub spend_outputs: Vec<SpendOutput>,
}

// Add this impl to get
// rust strings and values
// only for testing ppurposes(noticed the unwrappings)
impl TransparentInput {
    pub fn outp_str(&self) -> &str {
        self.outp.to_str().unwrap()
    }
    pub fn pk_str(&self) -> &str {
        self.pk.to_str().unwrap()
    }
    pub fn address_str(&self) -> &str {
        self.address.to_str().unwrap()
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

impl TransparentOutput {
    pub fn address_str(&self) -> &str {
        self.address.to_str().unwrap()
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

// Empty structs for now, you can expand these as needed
#[derive(Debug, Serialize, Deserialize)]
pub struct SpendInput {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpendOutput {}
