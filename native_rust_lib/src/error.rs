use std::mem::transmute;

// Error type to be used in FFI
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum ZcashError {
    #[error("Success")]
    Success = 0,
    #[error("Anchor mismatch")]
    AnchorMismatch = 1,
    #[error("Change binding signature")]
    BindingSig = 2,
    #[error("Change is negative")]
    ChangeIsNegative = 3,
    #[error("Invalid address")]
    InvalidAddress = 4,
    #[error("Invalid address format")]
    InvalidAddressFormat = 5,
    #[error("Invalid address hash")]
    InvalidAddressHash = 6,
    #[error("Invalid amount")]
    InvalidAmount = 7,
    #[error("No change address")]
    NoChangeAddress = 8,
    #[error("No spend proof")]
    SpendProof = 9,
    #[error("Missing spend signature")]
    MissingSpendSig = 10,
    #[error("Invalid spend signature")]
    SpendSig = 11,
    #[error("Invalid spend signature")]
    InvalidSpendSig = 12,
    #[error("No spend signature")]
    NoSpendSig = 13,
    #[error("Invalid transparent signature")]
    TransparentSig = 14,
    #[error("Invalid finalization")]
    Finalization = 15,
    #[error("Min shielded outputs")]
    MinShieldedOutputs = 16,
    #[error("Builder no keys")]
    BuilderNoKeys = 17,
    #[error("Read write error")]
    ReadWriteError = 18,
    #[error("Invalid ovk hash seed")]
    InvalidOVKHashSeed = 19,
    #[error("Already authorized")]
    AlreadyAuthorized = 20,
    #[error("Unauthorized")]
    Unauthorized = 21,
    #[error("Unknown authorization")]
    UnknownAuthorization = 22,
    #[error("Invalid network")]
    InvalidNetwork = 23,
    #[error("Builder not found")]
    BuilderNotFound = 24,
    #[error("Invalid argument")]
    InvalidArgument = 25,
    #[error("Invalid outpoint")]
    InvalidOutpoint = 26,
    #[error("Invalid script")]
    InvalidScript = 27,
    #[error("Invalid spend path")]
    InvalidSpendPath = 28,
    #[error("Invalid output path")]
    InvalidOutputPath = 29,
    #[error("Unknown error")]
    Unknown = 999,
}

impl TryFrom<u32> for ZcashError {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            // Handle the special case for `Unknown`
            999 => Ok(ZcashError::Unknown),
            // Use `transmute` for values in [0..23]
            x if x <= 29 => Ok(unsafe { transmute(x) }),
            // Return an error for all other values
            _ => Err(()),
        }
    }
}

impl From<ledger_chain_builder::errors::Error> for ZcashError {
    fn from(value: ledger_chain_builder::errors::Error) -> Self {
        match value {
            ledger_chain_builder::errors::Error::AnchorMismatch => Self::AnchorMismatch,
            ledger_chain_builder::errors::Error::BindingSig => Self::BindingSig,
            ledger_chain_builder::errors::Error::ChangeIsNegative => Self::ChangeIsNegative,
            ledger_chain_builder::errors::Error::InvalidAddress => Self::InvalidAddress,
            ledger_chain_builder::errors::Error::InvalidAddressFormat => Self::InvalidAddressFormat,
            ledger_chain_builder::errors::Error::InvalidAddressHash => Self::InvalidAddressHash,
            ledger_chain_builder::errors::Error::InvalidAmount => Self::InvalidAmount,
            ledger_chain_builder::errors::Error::NoChangeAddress => Self::NoChangeAddress,
            ledger_chain_builder::errors::Error::SpendProof => Self::SpendProof,
            ledger_chain_builder::errors::Error::MissingSpendSig => Self::MissingSpendSig,
            ledger_chain_builder::errors::Error::SpendSig => Self::SpendSig,
            ledger_chain_builder::errors::Error::InvalidSpendSig => Self::InvalidSpendSig,
            ledger_chain_builder::errors::Error::NoSpendSig => Self::NoSpendSig,
            ledger_chain_builder::errors::Error::TransparentSig => Self::TransparentSig,
            ledger_chain_builder::errors::Error::Finalization => Self::Finalization,
            ledger_chain_builder::errors::Error::MinShieldedOutputs => Self::MinShieldedOutputs,
            ledger_chain_builder::errors::Error::BuilderNoKeys => Self::BuilderNoKeys,
            ledger_chain_builder::errors::Error::ReadWriteError => Self::ReadWriteError,
            ledger_chain_builder::errors::Error::InvalidOVKHashSeed => Self::InvalidOVKHashSeed,
            ledger_chain_builder::errors::Error::AlreadyAuthorized => Self::AlreadyAuthorized,
            ledger_chain_builder::errors::Error::Unauthorized => Self::Unauthorized,
            ledger_chain_builder::errors::Error::UnknownAuthorization => Self::UnknownAuthorization,
        }
    }
}

pub(crate) fn get_error_description(code: u32) -> String {
    match ZcashError::try_from(code) {
        Ok(err) => err.to_string(),
        Err(_) => "Unknown error".to_owned(),
    }
}
