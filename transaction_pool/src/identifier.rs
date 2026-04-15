use primitives::types::{Account, Address};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TransactionId {
    pub sender: SenderId,
    pub nonce: u64,
}

#[derive(Debug, Clone)]
pub enum TransactionOrigin {
    Local,
    External,
}

pub type SenderInfo = Account;
pub type SenderId = Address;
