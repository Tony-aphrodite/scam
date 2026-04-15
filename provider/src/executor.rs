use std::{collections::HashMap, convert::Infallible};

use primitives::{block::Block, transaction::Recovered, types::{Account, Address, TxHash, B256, U256}, world::World};

use crate::{error::ExecutionError, state::{ExecutableState}};

pub struct Executor {
    pub state: ExecutableState,
    pub receipts: Vec<Receipt>,
}

impl Executor {

    pub fn new(state: ExecutableState) -> Self {
        Self { state, receipts: Vec::new() }
    }

    pub fn state(&mut self) -> &mut ExecutableState {
        &mut self.state
    }
    pub fn execute_transaction(&mut self, tx: &Recovered) 
    -> Result<Receipt, Infallible>{
        let mut receipt = Receipt { tx_hash: tx.hash(), fee: 0, success: true, error: None };
        receipt.fee = match self.state.execute_transaction(tx) {
            Ok(fee) => fee,
            Err(err) => {
                receipt.success = false;
                receipt.error = Some(ExecutionError::StateExecutionError(err));
                0
            }
        };
        self.receipts.push(receipt.clone());
        Ok(receipt)
    }

    // For validation external payload
    pub fn execute_block(&mut self, block: &Block) -> Result<(HashMap<Address, Account>, World), ExecutionError> {
        let transactions = &block.body;
        let proposer = block.header().proposer;
        let mut fee_sum = U256::ZERO;
        for transaction in transactions.iter() {
            let recovered = match transaction.clone().into_recovered() {
                Ok(recovered) => recovered,
                Err(e) => {
                    return Err(ExecutionError::TransactionRecoveryError(e));
                }
            };
            match self.execute_transaction(&recovered) {
                Ok(receipt) => {
                    fee_sum += U256::from(receipt.fee);
                }
                Err(_e) => {
                    continue;
                }
            }
        }

        if block.header.total_fee != fee_sum {
            return Err(ExecutionError::TotalFeeisDifferent);
        }

        // update mining results
        match self.state().accounts_write.get_mut(&proposer) {
            Some(account) => {
                account.add_balance(fee_sum);
            }
            None => {
                let mut new_account = Account::default();
                new_account.add_balance(fee_sum);
                self.state().accounts_write.insert(proposer, new_account);
            }
        }

        Ok((self.state.accounts_write.clone(), self.state.field_write.clone()))
    }

    pub fn calculate_state_root(&self) -> B256 {
        self.state.calculate_state_root()
    }
}



#[derive(Debug, Clone)]
pub struct Receipt {
    pub tx_hash: TxHash,
    pub fee: u128,
    pub success: bool,
    pub error: Option<ExecutionError>,
}