use std::array::TryFromSliceError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockImportError {
    #[error("Consensus block importer is Noob")]
    NoopImporter,
    #[error("Provider Error")]
    ProviderError,
    #[error("Block Height Error")]
    BlockHeightError,
    #[error("Block is already imported")]
    AlreadyImportedBlock,
    #[error("Block is not chained")]
    NotChainedBlock,
}

#[derive(Debug, Error)]
pub enum BlockValidatioError {
    #[error("Validation default Error")]
    DefaultError,
    #[error("Validator execution Error")]
    ExecutionError,
    #[error("Validator NotChainedBlock Error")]
    NotChainedBlock,
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("Invalid data to encode")]
    Invalid,
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Raw data is too short")]
    TooShortRawData(Vec<u8>),
    #[error("Invalid address")]
    InvalidAddress(AddressError),
    #[error("Invalid signature")]
    InvalidSignature(SignatureError),
    #[error("Slice error. Invalid raw data")]
    TryFromSliceError(TryFromSliceError),
}

impl From<TryFromSliceError> for DecodeError {
    fn from(err: TryFromSliceError) -> Self {
        Self::TryFromSliceError(err)
    }
}

#[derive(Debug, Clone, Error)]
pub enum AddressError {
    #[error("FromHexError")]
    FromHexError(hex::FromHexError),
    #[error("Address has invalid length")]
    InvalidLength(usize),
}

impl From<hex::FromHexError> for AddressError {
    fn from(err: hex::FromHexError) -> Self {
        Self::FromHexError(err)
    }
}

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Signature has invalid parity")]
    InvalidParity(u64),
}

#[derive(Debug, Clone, Error)]
pub enum RecoveryError {
    #[error("Recovery id Error")]
    RecIdError,
    #[error("Recovery key Error")]
    RecKeyError,
    #[error("Address Error")]
    AddressError(AddressError),
    #[error("Recover from digest Error")]
    RecoveryFromDigestError,
}

impl From<AddressError> for RecoveryError {
    fn from(err: AddressError) -> Self {
        Self::AddressError(err)
    }
}
