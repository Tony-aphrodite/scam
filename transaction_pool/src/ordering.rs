use primitives::transaction::{Tx};

use crate::validator::validtx::ValidPoolTransaction;


#[derive(Default, Debug)]
pub struct PintOrdering;

impl PintOrdering {
    pub fn priority(&self, transaction: &ValidPoolTransaction) -> u128{
        transaction.fee().into()
    } 
}