use std::sync::Arc;

use primitives::{block::Block, transaction::Recovered, types::TxHash};
use provider::{DatabaseTrait, ProviderFactory};
use tracing::info;

use crate::{
    error::{PoolError, PoolErrorKind, PoolResult},
    identifier::TransactionOrigin,
    pool::{PoolInner, best::BestTransactions},
    validator::TransactionValidationOutcome,
};

pub mod error;
pub mod identifier;
pub mod mock;
pub mod ordering;
pub mod pool;
pub mod validator;

#[derive(Debug, Clone)]
pub struct Pool<DB: DatabaseTrait> {
    pool: Arc<PoolInner<DB>>,
}

impl<DB: DatabaseTrait> Pool<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self {
            pool: Arc::new(PoolInner::new(provider)),
        }
    }

    pub fn remove_block_transactions(&self, block: &Block) {
        let tx_hashes: Vec<TxHash> = block.body.iter().map(|tx| tx.hash).collect();
        let mut pool = self.pool.pool().write();
        for hash in tx_hashes {
            pool.remove_transaction_by_hash(hash);
        }
    }

    pub fn add_transaction(
        &self,
        origin: TransactionOrigin,
        transaction: Recovered,
    ) -> PoolResult<TxHash> {
        let (_hash, outcome) = self.validate(origin, transaction);
        match outcome {
            TransactionValidationOutcome::Valid {
                transaction,
                balance,
                nonce,
            } => self
                .pool
                .pool()
                .write()
                .add_transaction(transaction, balance, nonce),
            TransactionValidationOutcome::Invalid { transaction, error } => {
                let pool_error = PoolError {
                    hash: transaction.hash(),
                    kind: PoolErrorKind::InvalidPoolTransactionError(error),
                };
                return Err(pool_error);
            }
            TransactionValidationOutcome::UnexpectedError(tx_hash) => {
                let pool_error = PoolError {
                    hash: tx_hash,
                    kind: crate::error::PoolErrorKind::ImportError,
                };
                return Err(pool_error);
            }
        }
    }

    pub fn validate(
        &self,
        origin: TransactionOrigin,
        transaction: Recovered,
    ) -> (TxHash, TransactionValidationOutcome) {
        let hash = transaction.hash();
        let outcome = self
            .pool
            .validator()
            .validate_transaction(origin, transaction);

        (hash, outcome)
    }

    pub fn best_transactions(&self) -> BestTransactions {
        self.pool.best_transactions()
    }

    pub fn reorganize_pool(&self) {
        self.pool.reorganize_pool();
        self.print_pool();
    }

    // for debug!
    pub fn print_pool(&self) {
        let pool = self.pool.pool().read();
        info!(
            "Pool txs info: All: {}, Pending: {}, Parked: {}",
            pool.all_transaction.len(),
            pool.pending_pool.len(),
            pool.parked_pool.len()
        )
    }

    pub fn check_pending_pool_len(&self) -> usize {
        let pool = self.pool.pool().read();
        pool.pending_pool.len()
    }
}
