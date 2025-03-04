use crate::ZcashError;

// Network type enum (C-compatible)
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum NetworkType {
    Mainnet = 0,
    Testnet = 1,
}

impl TryFrom<u8> for NetworkType {
    type Error = ZcashError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NetworkType::Mainnet),
            1 => Ok(NetworkType::Testnet),
            _ => Err(ZcashError::InvalidNetwork),
        }
    }
}
