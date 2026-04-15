use std::{collections::BTreeSet, sync::Arc};

use crate::{pool::pending::PendingTransaction, validator::validtx::ValidPoolTransaction};

pub struct BestTransactions {
    pub independent: BTreeSet<PendingTransaction>,
}

impl BestTransactions {
    fn pop_best(&mut self) -> Option<PendingTransaction> {
        let res = self.independent.pop_last();
        res
    }
}

impl Iterator for BestTransactions {
    type Item = Arc<ValidPoolTransaction>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let best = match self.pop_best() {
                Some(best) => best,
                None => {
                    return None;
                }
            };
            return Some(best.transaction.clone());
        }
    }
}