use std::sync::Arc;

use primitives::{transaction::{Recovered, Tx}, types::{TxHash, COINBASE_ADDR, U256}};
use provider::{DatabaseTrait, Provider, ProviderFactory};

use crate::{error::InvalidPoolTransactionError, identifier::{TransactionId, TransactionOrigin}, validator::validtx::ValidPoolTransaction};

pub mod validtx;

#[derive(Debug)]
pub struct Validator<DB: DatabaseTrait> {
    inner: Arc<ValidatorInner<DB>>,
}

impl<DB: DatabaseTrait> Validator<DB> {
    pub fn validate_transaction(&self, origin: TransactionOrigin, transaction: Recovered) -> TransactionValidationOutcome {
        let res = self.inner.validate_one(origin, transaction, None);
        res
    }
}

impl<DB: DatabaseTrait> Validator<DB> {
    pub fn new(provier: ProviderFactory<DB>) -> Self {
        Self {
            inner: Arc::new(ValidatorInner::new(provier)),
        }
    }
}

#[derive(Debug)]
pub struct ValidatorInner<DB: DatabaseTrait> {
    provider: ProviderFactory<DB>,
}

impl<DB: DatabaseTrait> ValidatorInner<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self {
            provider,
        }
    }

    // maybe_state: specific state
    // if maybe_state is none, validates tx according to latest block
    pub fn validate_one(&self, origin: TransactionOrigin, transaction: Recovered, mut maybe_state: Option<Provider<DB>>) 
    -> TransactionValidationOutcome{
        match self.validate_one_no_state(transaction) {
            Ok(transaction) => {
                if maybe_state.is_none() {
                    maybe_state = Some(self.provider.latest());
                }

                let state = maybe_state.unwrap();
                self.validate_one_against_state(origin, transaction, state)
            }
            Err(e) => {
                e
            }
        }
    }

    fn validate_one_no_state(&self, transaction: Recovered) -> Result<Recovered, TransactionValidationOutcome> {
        if transaction.fee() <= 0 {
            return Err(TransactionValidationOutcome::Invalid{
                transaction: transaction, 
                error: InvalidPoolTransactionError::NotEnoughFeeError
            });
        }

        if transaction.signer() == COINBASE_ADDR {
            return Err(TransactionValidationOutcome::Invalid{
                transaction: transaction, 
                error: InvalidPoolTransactionError::UsingCoinbaseAddr
            });
        }

        Ok(transaction)
    }

    fn validate_one_against_state(&self, origin: TransactionOrigin, transaction: Recovered, state: Provider<DB>)
    -> TransactionValidationOutcome {
        let account = match state.basic_account(transaction.signer()) {
            Ok(account) => account.unwrap_or_default(),
            Err(_err) => {
                return TransactionValidationOutcome::UnexpectedError(transaction.hash());
            }
        };

        // Checks nonce >= on_chain_node
        if transaction.nonce() < account.nonce {
            return TransactionValidationOutcome::Invalid{
                transaction,
                error: InvalidPoolTransactionError::NonceIsNotConsistent,
            }
        }

        let valid_pool_transaction = ValidPoolTransaction {
            transaction: transaction.clone(),
            transaction_id: TransactionId {
                sender: transaction.signer(),
                nonce: transaction.nonce(),
            },
            origin,
            timestamp: std::time::Instant::now(),
        };

        TransactionValidationOutcome::Valid {
            transaction: valid_pool_transaction,
            balance: account.balance,
            nonce: account.nonce,
        }
    }
}

#[derive(Debug)]
pub enum TransactionValidationOutcome {
    Valid {
        transaction: ValidPoolTransaction,
        balance: U256,
        nonce: u64
    },
    Invalid {
        transaction: Recovered,
        error: InvalidPoolTransactionError,
    },
    UnexpectedError(TxHash),
}

impl TransactionValidationOutcome {
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Valid {..} => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use database::{immemorydb::InMemoryDB, mdbx::MDBX};
    use primitives::{transaction::SignedTransaction, types::{Account, U256}};

    use crate::identifier::TransactionOrigin;

    use super::*;

    fn new_transaction() -> SignedTransaction {
        // let tx = Transaction {chain_id: 0, nonce: 0, to: receiver, fee: 1, value: U256::from(1)};
        // sender: 28dcb1338b900419cd613a8fb273ae36e7ec2b1d, receiver: 0534501c34f5a0f3fa43dc5d78e619be7edfa21a
        let raw = "000000000000000000000000000000000534501c34f5a0f3fa43dc5d78e619be7edfa21a000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001edb54ca220c37699400def4d371e024ef94c7b62cd88d65d6e544807f957174c6aed902279a2abcd976584998e4c2b886e2a252ed375d44000f842e0d362fcea00";
        let data = hex::decode(raw).unwrap();
        let (signed_tx, _) = SignedTransaction::decode(&data).unwrap();
        signed_tx
    }

    fn new_zero_fee_transaction() -> SignedTransaction {
        // let tx = Transaction {chain_id: 0, nonce: 0, to: receiver, fee: 1, value: U256::from(1)};
        // sender: 28dcb1338b900419cd613a8fb273ae36e7ec2b1d, receiver: 0534501c34f5a0f3fa43dc5d78e619be7edfa21a
        let raw = "000000000000000000000000000000000534501c34f5a0f3fa43dc5d78e619be7edfa21a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001edb54ca220c37699400def4d371e024ef94c7b62cd88d65d6e544807f957174c6aed902279a2abcd976584998e4c2b886e2a252ed375d44000f842e0d362fcea00";
        let data = hex::decode(raw).unwrap();
        let (signed_tx, _) = SignedTransaction::decode(&data).unwrap();
        signed_tx
    }

    #[test]
    fn test_validate_pending_transaction() {
        let transaction = new_transaction();
        let recovered = transaction.into_recovered().unwrap();
        let mut db = MDBX::genesis_state();
        db.add_account(recovered.signer(), Account::new(recovered.nonce(), U256::MAX)).unwrap();

        let provider = ProviderFactory::new(db);
        let validator = Validator::new(provider);

        let outcome: TransactionValidationOutcome =
            validator.validate_transaction(TransactionOrigin::External, recovered.clone());

        assert!(outcome.is_valid());
        dbg!(outcome);
    }

    #[test]
    fn test_validate_parked_transaction() {
        let transaction = new_transaction();
        let recovered = transaction.into_recovered().unwrap();
        let mut db = InMemoryDB::new();
        db.add_account(recovered.signer(), Account::new(recovered.nonce(), U256::ZERO)).unwrap();

        let db = Arc::new(db);


        let provider = ProviderFactory::new(db);
        let validator = Validator::new(provider);

        let outcome: TransactionValidationOutcome =
            validator.validate_transaction(TransactionOrigin::External, recovered.clone());

        assert!(outcome.is_valid());
        dbg!(outcome);
    }

    #[test]
    fn test_validate_invalid_fee_transaction() {
        let transaction = new_zero_fee_transaction();
        let recovered = transaction.into_recovered().unwrap();
        let mut db = InMemoryDB::new();
        db.add_account(recovered.signer(), Account::new(recovered.nonce(), U256::MAX)).unwrap();

        let db = Arc::new(db);

        let provider = ProviderFactory::new(db);
        let validator = Validator::new(provider);

        let outcome: TransactionValidationOutcome =
            validator.validate_transaction(TransactionOrigin::External, recovered.clone());

        assert!(!outcome.is_valid());
        dbg!(outcome);
    }

    #[test]
    fn test_validate_invalid_nonce_transaction() {
        let transaction = new_transaction();
        let recovered = transaction.into_recovered().unwrap();
        let mut db = InMemoryDB::new();
        db.add_account(recovered.signer(), Account::new(recovered.nonce() + 1, U256::MAX)).unwrap();

        let db = Arc::new(db);

        let provider = ProviderFactory::new(db);
        let validator = Validator::new(provider);

        let outcome: TransactionValidationOutcome =
            validator.validate_transaction(TransactionOrigin::External, recovered.clone());

        assert!(!outcome.is_valid());
        dbg!(outcome);
    }
}
