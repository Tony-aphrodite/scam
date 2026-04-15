use primitives::{error::RecoveryError, types::TxHash};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum ExecutionError {
    #[error("State execution Error")]
    StateExecutionError(StateExecutionError),
    #[error("Transaction Recovery Error")]
    TransactionRecoveryError(RecoveryError),
    #[error("Total fee is diffrent")]
    TotalFeeisDifferent,
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Database Error")]
    DatabaseError(Box<dyn std::error::Error>),
    #[error("Execution Error")]
    ExecutionError(ExecutionError),
    #[error("State is not exist")]
    StateNotExist(u64),
}

#[derive(Clone, Debug, Error)]
pub enum StateExecutionError {
    #[error("Transaction execution Error")]
    TransactionExecutionError(TxHash, TxExecutionError),
}

#[derive(Clone, Debug, Error)]
pub enum TxExecutionError {
    #[error("Sender has not enough balance")]
    SenderHasNotEnoughBalance,
    #[error("State has no account")]
    SenderHasNoAccount,
    #[error("Invalid nonce value")]
    NonceError(u64, u64),
}
