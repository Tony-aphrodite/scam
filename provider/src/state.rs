use std::{collections::HashMap, sync::Arc};

use primitives::{
    merkle::calculate_merkle_root,
    transaction::{Recovered, Tx},
    types::{Account, Address, B256, U256},
    world::World,
};
use sha2::{Digest, Sha256};

use crate::error::{StateExecutionError, TxExecutionError};

// #[derive(Debug)]
// pub struct State {
//     accounts: Arc<HashMap<Address, Account>>,
//     field: Arc<World>,
// }

#[derive(Debug)]
pub struct ExecutableState {
    pub accounts_base: Arc<HashMap<Address, Account>>,
    pub accounts_write: HashMap<Address, Account>,
    pub field_base: Arc<World>,
    pub field_write: World,
}

impl ExecutableState {
    pub fn execute_transaction(
        &mut self,
        transaction: &Recovered,
    ) -> Result<u128, StateExecutionError> {
        let sender = transaction.signer();
        let receiver = transaction.to();

        let mut sender_account = match self.accounts_write.get(&sender) {
            Some(account) => account.clone(),
            // sender must have balance because of fee
            None => {
                return Err(StateExecutionError::TransactionExecutionError(
                    transaction.hash(),
                    TxExecutionError::SenderHasNoAccount,
                ));
            }
        };

        let mut receiver_account = match self.accounts_write.get(&receiver) {
            Some(account) => account.clone(),
            None => Account::default(),
        };

        if U256::from(transaction.fee()) > sender_account.balance() - transaction.value() {
            return Err(StateExecutionError::TransactionExecutionError(
                transaction.hash(),
                TxExecutionError::SenderHasNotEnoughBalance,
            ));
        }

        if sender_account.nonce() != transaction.nonce() {
            return Err(StateExecutionError::TransactionExecutionError(
                transaction.hash(),
                TxExecutionError::NonceError(sender_account.nonce, transaction.nonce()),
            ));
        }

        sender_account.sub_balance(transaction.value());
        sender_account.sub_balance(U256::from(transaction.fee()));
        receiver_account.add_balance(transaction.value());
        sender_account.increase_nonce();

        self.accounts_write.insert(sender, sender_account);
        self.accounts_write.insert(receiver, receiver_account);

        // TODO: Update World.
        Ok(transaction.fee())
    }

    pub fn calculate_state_root(&self) -> B256 {
        let mut entries: Vec<_> = self.accounts_write.iter().collect();
        // Address + ord!
        entries.sort_by_key(|(k, _)| *k);

        let mut entry_hashes: Vec<B256> = entries
            .iter()
            .map(|(k, v)| {
                let mut hasher = Sha256::new();
                hasher.update(k.get_addr_hex());
                hasher.update(v.balance.to_be_bytes::<32>());
                hasher.update(v.nonce.to_be_bytes());
                B256::from_slice(&hasher.finalize())
            })
            .collect();

        let world_hash = self.field_write.calculate_hash();
        entry_hashes.push(world_hash);

        let state_root = calculate_merkle_root(entry_hashes);
        state_root
    }
}
