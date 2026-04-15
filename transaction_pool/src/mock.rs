use std::{time::Instant};

use primitives::transaction::{Recovered, Tx};

use crate::{identifier::{TransactionId, TransactionOrigin}, validator::validtx::ValidPoolTransaction};

#[derive(Default)]
pub struct MockValidator;

impl MockValidator {

    pub fn validate(&mut self, tx: Recovered) -> ValidPoolTransaction {
        let tid = TransactionId {
            sender: tx.signer(),
            nonce: tx.nonce()
        };
        ValidPoolTransaction {
            transaction: tx,
            transaction_id: tid,
            origin: TransactionOrigin::External,
            timestamp: Instant::now(),
        }
    }
}
