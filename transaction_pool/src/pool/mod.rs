use std::sync::Arc;

use parking_lot::RwLock;
use provider::{DatabaseTrait, ProviderFactory};

use crate::{identifier::TransactionId, pool::{best::BestTransactions, txpool::TxPool}, validator::{validtx::ValidPoolTransaction, TransactionValidationOutcome, Validator}};

pub mod txpool;
pub mod pending;
pub mod parked;
pub mod state;
pub mod best;

#[derive(Debug)]
pub struct PoolInner<DB: DatabaseTrait> {
    validator: Validator<DB>,
    transaction_pool: RwLock<TxPool>,
}

impl<DB: DatabaseTrait> PoolInner<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self {
            validator: Validator::new(provider),
            transaction_pool: RwLock::new(TxPool::new()),
        }
    }

    pub fn validator(&self) -> &Validator<DB> {
        &self.validator
    }

    pub fn pool(&self) -> &RwLock<TxPool> {
        &self.transaction_pool
    }

    pub fn best_transactions(&self) -> BestTransactions {
        self.pool().read().best_transactions()
    }

    pub fn reorganize_pool(&self) {

        // 1. bring all txs in parked pool
        let mut pool = self.pool().write();
        let parked_txs: Vec<(TransactionId, Arc<ValidPoolTransaction>)> = pool.parked_pool.transactions()
            .iter()
            .map(|(tid, tx)| (*tid, tx.tx_clone()))
            .collect();


        for (tid, tx) in parked_txs.iter() {
            match self.validator().validate_transaction(tx.origin.clone(), tx.transaction.clone()) {
                TransactionValidationOutcome::Valid { transaction: _, balance, nonce } => {
                    pool.reorg_transaction(tid, balance, nonce);
                }
                TransactionValidationOutcome::Invalid { transaction: _, error: _ } => {
                    pool.remove_transaction_by_id(tid);
                }
                TransactionValidationOutcome::UnexpectedError(_tx_hash) => {

                }
            }
        }
    }
}

