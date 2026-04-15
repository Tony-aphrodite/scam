use std::{collections::BTreeMap, sync::Arc};

use crate::{identifier::TransactionId, validator::validtx::ValidPoolTransaction};

#[derive(Default, Debug)]
pub struct ParkedPool {
    submission_id: u64,
    by_id: BTreeMap<TransactionId, ParkedTransaction>,
}

impl ParkedPool {
    pub fn add_transaction(&mut self, tx: Arc<ValidPoolTransaction>) {
        let id = *tx.tid();
        let submission_id = self.next_id();
        let tx = ParkedTransaction {
            submission_id,
            transaction: tx,
        };

        self.by_id.insert(id, tx);
    }

    pub fn remove_transaction(
        &mut self,
        id: &TransactionId,
    ) -> Option<Arc<ValidPoolTransaction>> {
        let tx = self.by_id.remove(id)?;
        Some(tx.transaction)
    }

    const fn next_id(&mut self) -> u64 {
        let id = self.submission_id;
        self.submission_id = self.submission_id.wrapping_add(1);
        id
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn transactions(&self) -> &BTreeMap<TransactionId, ParkedTransaction> {
        &self.by_id
    }


}

#[derive(Debug)]
pub struct ParkedTransaction {
    submission_id: u64,
    transaction: Arc<ValidPoolTransaction>,
}

impl ParkedTransaction {
    pub fn id(&self) -> u64 {
        self.submission_id
    }

    pub fn tx(&self) -> &ValidPoolTransaction {
        &self.transaction
    }

    pub fn tx_clone(&self) -> Arc<ValidPoolTransaction> {
        self.transaction.clone()
    }
}