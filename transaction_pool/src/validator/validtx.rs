use primitives::{transaction::{Recovered, Tx}, types::{TxHash, U256}};

use crate::identifier::{SenderId, TransactionId, TransactionOrigin};

#[derive(Debug, Clone)]
pub struct ValidPoolTransaction {
    pub transaction: Recovered,
    pub transaction_id: TransactionId,
    pub origin: TransactionOrigin,
    pub timestamp: std::time::Instant,
}

impl ValidPoolTransaction {

    pub fn tx(&self) -> &Recovered {
        &self.transaction
    }

    pub fn tid(&self) -> &TransactionId {
        &self.transaction_id
    }

    pub fn sender(&self) -> SenderId {
        self.tid().sender.clone()
    }

    pub fn hash(&self) -> TxHash {
        self.tx().tx().hash
    }


    pub fn is_underpriced(&self, other: &Self) -> bool {
        self.fee() < other.fee()
    }
}

impl Tx for ValidPoolTransaction {
    fn chain_id(&self) -> primitives::types::ChainId {
        self.tx().chain_id()
    }

    fn nonce(&self) -> u64 {
        self.tx().nonce()
    }

    fn to(&self) -> primitives::types::Address {
        self.tx().to()
    }

    fn fee(&self) -> u128 {
        self.tx().fee()
    }

    fn value(&self) -> U256 {
        self.tx().value()
    }
}
