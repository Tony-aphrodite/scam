use std::{collections::BTreeMap, sync::Arc};

use crate::{identifier::TransactionId, ordering::PintOrdering, pool::best::BestTransactions, validator::validtx::ValidPoolTransaction};

#[derive(Default, Debug)]
pub struct PendingPool {
    pub ordering: PintOrdering,
    pub submission_id: u64,
    pub independent: BTreeMap<TransactionId, PendingTransaction>,
}

impl PendingPool {

    pub fn add_transaction(
        &mut self,
        tx: Arc<ValidPoolTransaction>,
        // Base fee of blocks. If Tx fee is under this, It should rejected!
    ) {
        assert!(
            !self.contains(tx.tid()),
            "transaction already included {:?}",
            self.independent.get(tx.tid()).unwrap().transaction
        );

        let tx_id = *tx.tid();
        let submission_id = self.next_id();
        let priority = self.ordering.priority(&tx);

        let tx = PendingTransaction {
            submission_id,
            transaction: tx,
            priority,
        };

        self.independent.insert(tx_id, tx);
    }

    pub fn remove_transaction(
        &mut self,
        id: &TransactionId,
    ) -> Option<Arc<ValidPoolTransaction>> {
        let tx = self.independent.remove(id)?;
        Some(tx.transaction)
    }

    fn contains(&self, id: &TransactionId) -> bool {
        self.independent.contains_key(id)
    }

    const fn next_id(&mut self) -> u64 {
        let id = self.submission_id;
        self.submission_id = self.submission_id.wrapping_add(1);
        id
    }

    pub fn len(&self) -> usize {
        self.independent.len()
    }

    pub fn best(&self) -> BestTransactions {
        BestTransactions {
            independent: self.independent.values().cloned().collect()
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingTransaction {
    pub submission_id: u64,
    pub transaction: Arc<ValidPoolTransaction>,
    pub priority: u128,
}

impl Ord for PendingTransaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority).then_with(|| self.submission_id.cmp(&other.submission_id))
    }
}

impl PartialOrd for PendingTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PendingTransaction {}
impl PartialEq for PendingTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}