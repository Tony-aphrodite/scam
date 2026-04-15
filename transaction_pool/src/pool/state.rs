use std::sync::Arc;

use primitives::{transaction::Tx, types::U256};

use crate::validator::validtx::ValidPoolTransaction;

#[derive(Debug, Clone)]
pub enum SubPool {
    Pending,
    Parked,
}

impl SubPool {
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Pending => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TxState {
    has_balance: bool,
    has_ancestor: bool,
}

impl TxState {
    pub fn new() -> Self {
        Self { has_balance: false, has_ancestor: false }
    }

    pub fn new_with(tx: &Arc<ValidPoolTransaction>, on_chain_balance: U256, on_chain_nonce: u64) -> Self {
        let mut state = TxState::new();
        if U256::from(tx.fee()) + tx.value() <= on_chain_balance {
            state.has_balance();
        } 

        if tx.nonce() > on_chain_nonce {
            state.has_ancestor();
        }
        state
    }

    pub fn has_balance(&mut self) {
        self.has_balance = true;
    }

    pub fn has_ancestor(&mut self) {
        self.has_ancestor = true;
    }
}

impl From<TxState> for SubPool {
    fn from(value: TxState) -> Self {
        match value.has_balance && !value.has_ancestor {
            true => SubPool::Pending,
            false => SubPool::Parked,
        }
    }
}
