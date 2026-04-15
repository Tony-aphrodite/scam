use primitives::types::TxHash;
use std::sync::Arc;
use thiserror::Error;

use crate::validator::validtx::ValidPoolTransaction;

pub type PoolResult<T> = Result<T, PoolError>;

#[derive(Debug, Clone)]
pub struct PoolError {
    pub hash: TxHash,
    pub kind: PoolErrorKind,
}

impl PoolError {
    pub fn new(hash: TxHash, kind: PoolErrorKind) -> Self {
        Self { hash, kind }
    }
}

#[derive(Debug, Clone, Error)]
pub enum PoolErrorKind {
    #[error("Transaction is already imported")]
    AlreadyImported,
    #[error("Invalid transaction")]
    InvalidTransaction(Arc<ValidPoolTransaction>),
    #[error("Transaction is replaced underpriced")]
    RelpacementUnderpriced(Arc<ValidPoolTransaction>),
    #[error("Default import error")]
    ImportError,
    #[error("Invalid transaction with pool")]
    InvalidPoolTransactionError(InvalidPoolTransactionError),
}

#[derive(Debug, Error)]
pub enum InsertErr {
    #[error("Transaction is underpriced")]
    Underpriced {
        transaction: Arc<ValidPoolTransaction>,
    },
    #[error("Transaction is invalid")]
    InvalidTransaction {
        transaction: Arc<ValidPoolTransaction>,
    },
}

#[derive(Debug, Clone, Error)]
pub enum InvalidPoolTransactionError {
    #[error("Transaction has not enough fee")]
    NotEnoughFeeError,
    #[error("Transaction nonce is not consistent")]
    NonceIsNotConsistent,
    #[error("Transaction used coinbase_addr(0x0000...)")]
    UsingCoinbaseAddr,
}
