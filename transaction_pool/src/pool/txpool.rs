use std::{collections::{btree_map::Entry, BTreeMap, HashMap}, sync::Arc};

use primitives::{transaction::Tx, types::{TxHash, U256}};

use crate::{error::{InsertErr, PoolError, PoolErrorKind, PoolResult}, identifier::{SenderId, SenderInfo, TransactionId}, pool::{best::BestTransactions, parked::{ParkedPool}, pending::PendingPool, state::{SubPool, TxState}}, validator::validtx::ValidPoolTransaction};


#[derive(Debug)]
pub struct TxPool {
    pub all_transaction: AllTransaction,
    sender_info: HashMap<SenderId, SenderInfo>,
    pub pending_pool: PendingPool,
    pub parked_pool: ParkedPool,
}

impl TxPool {
    pub fn new() -> Self {
        Self {
            all_transaction: AllTransaction::default(),
            sender_info: Default::default(),
            pending_pool: PendingPool::default(),
            parked_pool: ParkedPool::default(),
        }
    }

    pub fn contains_by_hash(&self, tx_hash: &TxHash) -> bool {
        self.all_transaction.contains_by_hash(&tx_hash)
    }

    pub fn contains_by_id(&self, tid: &TransactionId) -> bool {
        self.all_transaction.contains_by_id(tid)
    }

    pub fn reorg_transaction(&mut self, tx_id: &TransactionId, on_chain_balance: U256, on_chain_nonce: u64) {

        if !self.contains_by_id(tx_id) {
            return;
        }
        
        let pool_transaction = self.all_transaction.get_tx_by_id(tx_id).unwrap();

        let transaction = pool_transaction.transaction.clone();

        // update sender info
        self.sender_info
            .entry(transaction.sender())
            .or_default()
            .update(on_chain_nonce, on_chain_balance);

        // let tx_hash = transaction.hash();
        let new_state = TxState::new_with(&transaction, on_chain_balance, on_chain_nonce);
        let sub_pool: SubPool = new_state.into();

        if sub_pool.is_pending() {
            self.parked_pool.remove_transaction(tx_id);
            let _ = self.all_transaction.insert_transaction(transaction.clone(), on_chain_balance, on_chain_nonce);
            self.add_new_transaction(transaction, None, sub_pool);
        }
    }

    pub fn add_transaction(
        &mut self,
        transaction: ValidPoolTransaction,
        on_chain_balance: U256,
        on_chain_nonce: u64,
    ) -> PoolResult<TxHash> {
        // check whether new transaction is already inserted or not
        if self.contains_by_hash(&transaction.hash()) {
            return Err(PoolError::new(
                transaction.hash(),
                PoolErrorKind::AlreadyImported,
            ));
        }

        // update sender info
        self.sender_info
            .entry(transaction.sender())
            .or_default()
            .update(on_chain_nonce, on_chain_balance);

        let tx_hash = transaction.hash();

        match self.all_transaction.insert_transaction(Arc::new(transaction), on_chain_balance, on_chain_nonce) {
            Ok(InsertOk {
                transaction,
                replaced_tx,
                sub_pool,
            }) => {
                self.add_new_transaction(transaction.clone(), replaced_tx.clone(), sub_pool);
            }
            Err(err) => match err {
                InsertErr::Underpriced { transaction } => return Err(PoolError::new(
                    transaction.hash(),
                    PoolErrorKind::RelpacementUnderpriced(transaction),
                )),
                InsertErr::InvalidTransaction { transaction } => return Err(PoolError::new(
                    transaction.hash(),
                    PoolErrorKind::InvalidTransaction(transaction),
                )),
            },
        }
        Ok(tx_hash)
    }

    fn add_new_transaction(
        &mut self, 
        transaction: Arc<ValidPoolTransaction>, 
        replaced_tx: Option<(Arc<ValidPoolTransaction>, SubPool)>,
        subpool: SubPool,
    ) {
        if let Some((replaced, replaced_subpool)) = replaced_tx {
            self.remove_from_subpool(replaced.tid(), replaced_subpool);
        }
        self.add_transaction_to_subpool(transaction, subpool);
    }

    fn add_transaction_to_subpool(
        &mut self,
        transaction: Arc<ValidPoolTransaction>,
        subpool: SubPool,
    ) {
        match subpool {
            SubPool::Parked => {
                self.parked_pool.add_transaction(transaction);
            }
            SubPool::Pending => {
                self.pending_pool.add_transaction(transaction);
            }
        }
    }

    pub fn remove_transaction_by_id(
        &mut self,
        id: &TransactionId,
    ) -> Option<Arc<ValidPoolTransaction>> {
        let (tx, subpool) = self.all_transaction.remove_transaction_by_id(id)?;
        self.remove_from_subpool(tx.tid(), subpool)
    }

    pub fn remove_transaction_by_hash(&mut self, hash: TxHash) -> Option<Arc<ValidPoolTransaction>> {
        let (tx, subpool) = self.all_transaction.remove_transaction_by_hash(hash)?;
        self.remove_from_subpool(tx.tid(), subpool)
    }

    fn remove_from_subpool(
        &mut self,
        tx_id: &TransactionId,
        subpool: SubPool,
    ) -> Option<Arc<ValidPoolTransaction>> {
        let tx = match subpool {
            SubPool::Pending => self.pending_pool.remove_transaction(tx_id),
            SubPool::Parked => self.parked_pool.remove_transaction(tx_id),
        };

        if let Some(ref _tx) = tx {
            // info!("Removed transaction from a subpool: {:?}, ",tx);
        }
        tx
    }

    pub fn best_transactions(&self) -> BestTransactions {
        self.pending_pool.best()
    }
}

#[derive(Default, Debug)]
pub struct AllTransaction {
    // For lookup
    by_hash: HashMap<TxHash, Arc<ValidPoolTransaction>>,
    // For arranging
    txs: BTreeMap<TransactionId, PoolInternalTransaction>,
}

impl AllTransaction {
    pub fn contains_by_hash(&self, hash: &TxHash) -> bool {
        self.by_hash.contains_key(hash)
    }

    pub fn contains_by_id(&self, tid: &TransactionId) -> bool {
        self.txs.contains_key(tid)
    }

    pub fn get_tx_by_id(&self, tid: &TransactionId) -> Option<&PoolInternalTransaction> {
        self.txs.get(tid)
    }

    pub fn len(&self) -> usize {
        self.txs.len()
    }

    pub fn remove_transaction_by_id(
        &mut self,
        id: &TransactionId,
    ) -> Option<(Arc<ValidPoolTransaction>, SubPool)> {
        let internal = self.txs.remove(id)?;
        let hash = internal.transaction.hash();
        let tx = self.by_hash.remove(&hash)?;

        Some((tx, internal.sub_pool))
    }

    pub fn remove_transaction_by_hash(&mut self, hash: TxHash) -> Option<(Arc<ValidPoolTransaction>, SubPool)> {
        let tx = self.by_hash.remove(&hash)?;
        let tid = tx.tid();
        let internal = self.txs.remove(tid)?;

        Some((tx, internal.sub_pool))

    }

    pub fn insert_transaction(&mut self, transaction: Arc<ValidPoolTransaction>, on_chain_balance: U256, on_chain_nonce: u64) -> Result<InsertOk, InsertErr> {
        assert!(
            on_chain_nonce <= transaction.nonce(),
            "Invalid transaction due to nonce."
        );

        assert!(
            0 < transaction.fee(),
            "Invalid transaction due to fee."
        );

        let tx = transaction;
        let mut replaced_tx = None;

        let state = TxState::new_with(&tx, on_chain_balance, on_chain_nonce);

        let pool_tx = PoolInternalTransaction {
            transaction: Arc::clone(&tx),
            sub_pool: state.into(),
            state
        };

        match self.txs.entry(*pool_tx.transaction.tid()) {
            // Newly inserted transactionId
            Entry::Vacant(entry) => {
                self.by_hash.insert(pool_tx.transaction.hash(), Arc::clone(&tx));
                entry.insert(pool_tx);
            }
            // Already inserted transactionId
            // 1. compare price of both transactions
            // 2. if new tx wins, replace it.
            Entry::Occupied(mut entry) => {
                let old_tx: &ValidPoolTransaction = entry.get().transaction.as_ref();
                let new_tx = tx.as_ref();

                if new_tx.is_underpriced(old_tx) {
                    return Err(InsertErr::Underpriced { transaction: tx });
                }

                let new_hash = new_tx.hash();
                let new_tx = pool_tx.transaction.clone();
                let replaced: PoolInternalTransaction = entry.insert(pool_tx);
                self.by_hash.remove(&replaced.transaction.hash());
                self.by_hash.insert(new_hash, new_tx);

                replaced_tx = Some((replaced.transaction, replaced.sub_pool));
            }
        }

        Ok(InsertOk {
            transaction: tx,
            replaced_tx,
            sub_pool: state.into(),
        })
    }
}

/// Struct that notifies a transaction was inserted, along with additional info
pub struct InsertOk {
    transaction: Arc<ValidPoolTransaction>,
    replaced_tx: Option<(Arc<ValidPoolTransaction>, SubPool)>,
    sub_pool: SubPool,
}


#[derive(Debug)]
pub struct PoolInternalTransaction {
    pub transaction: Arc<ValidPoolTransaction>,
    pub sub_pool: SubPool,
    pub state: TxState,
}

#[cfg(test)]
mod tests {
    use k256::{ecdsa::{RecoveryId, Signature as ECDSASig, SigningKey}, EncodedPoint};
    use primitives::{signature::Signature, transaction::{SignedTransaction, Transaction}, types::{Address, U256}};
    use sha2::{Digest, Sha256};

    use crate::{mock::MockValidator};

    use super::*;

    fn create_key_pairs(seed: &[u8]) -> (SigningKey, Vec<u8>) {
        let private_key_random = Sha256::digest(&seed);
        let signing_key = SigningKey::from_bytes(&private_key_random).unwrap();

        let verifying_key = signing_key.clone().verifying_key().clone();
        let pubkey_uncompressed: EncodedPoint = verifying_key.to_encoded_point(false);
        let pubkey_bytes = pubkey_uncompressed.as_bytes();
        let address = pubkey_bytes[pubkey_bytes.len() - 20..].to_vec();
        (signing_key, address)
    }

    fn create_new_signed_tx(nonce: u64, fee: u128, value: U256, sender: &str, receiver: &str) -> SignedTransaction {
        let (signing_key, sender) = create_key_pairs(sender.as_bytes());
        let sender = Address::from_byte(sender.try_into().unwrap());
        dbg!(&sender.get_addr_hex());

        let (_, receiver) = create_key_pairs(receiver.as_bytes());
        let receiver = Address::from_byte(receiver.try_into().unwrap());
        dbg!(receiver.get_addr_hex());

        let tx = Transaction {
            chain_id: 0,
            nonce,
            to: receiver,
            fee,
            value,
        };

        let tx_hash = tx.encode_for_signing();
        let digest = Sha256::new_with_prefix(tx_hash.hash());
        let (sig, recid): (ECDSASig, RecoveryId) =
            signing_key.sign_digest_recoverable(digest).unwrap();
        let signature = Signature::from_sig(sig, recid);

        SignedTransaction::new(tx, signature, tx_hash)
    }

    #[test]
    fn test_insert_pending_pool() {

        let mut factory = MockValidator::default();
        let mut pool = TxPool::new();

        let signed_tx = create_new_signed_tx(0, 1, U256::from(1), "pint", "chain");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(2);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx, on_chain_balance, on_chain_nonce);
        
        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());
        assert_eq!(1, pool.all_transaction.len());

        let signed_tx = create_new_signed_tx(0, 1, U256::from(1), "apple", "banana");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(2);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx, on_chain_balance, on_chain_nonce);
        
        assert_eq!(2, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());
        assert_eq!(2, pool.all_transaction.len());

        let signed_tx = create_new_signed_tx(0, 2, U256::from(1), "apple", "banana");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(2);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx, on_chain_balance, on_chain_nonce);
        
        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(1, pool.parked_pool.len());
        assert_eq!(2, pool.all_transaction.len());
    }

    #[test]
    fn test_insert_parked_pool() {

        let mut factory = MockValidator::default();
        let mut pool = TxPool::new();

        // 1st parked tx
        let signed_tx = create_new_signed_tx(0, 1, U256::from(1), "pint", "chain");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(1);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx, on_chain_balance, on_chain_nonce);
        
        assert_eq!(0, pool.pending_pool.len());
        assert_eq!(1, pool.parked_pool.len());

        // 2nd parked tx
        let signed_tx = create_new_signed_tx(1, 1, U256::from(1), "pint", "chain");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(2);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx, on_chain_balance, on_chain_nonce);

        assert_eq!(0, pool.pending_pool.len());
        assert_eq!(2, pool.parked_pool.len());
    }

    #[test]
    fn test_insert_already_imported() {

        let mut factory = MockValidator::default();
        let mut pool = TxPool::new();
        let signed_tx = create_new_signed_tx(0, 1, U256::from(1), "pint", "chain");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(2);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        
        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());

        let _res = pool.add_transaction(vtx, on_chain_balance, on_chain_nonce);

        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());
    }

    #[test]
    fn test_replace_pending_pool() {

        let mut factory = MockValidator::default();
        let mut pool = TxPool::new();

        // old tx
        let signed_tx = create_new_signed_tx(0, 1, U256::from(1), "pint", "chain");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(4);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx, on_chain_balance, on_chain_nonce);
        
        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());

        // new tx
        let signed_tx = create_new_signed_tx(0, 2, U256::from(1), "pint", "chain");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(4);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx, on_chain_balance, on_chain_nonce);

        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());
    }

    #[test]
    #[should_panic(expected = "Invalid transaction")]
    fn test_insert_invalid_nonce() {

        let mut factory = MockValidator::default();
        let mut pool = TxPool::new();
        let signed_tx = create_new_signed_tx(0, 1, U256::from(1), "pint", "chain");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(2);
        let on_chain_nonce = 1;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
    }

    #[test]
    #[should_panic(expected = "Invalid transaction")]
    fn test_insert_invalid_fee() {

        let mut factory = MockValidator::default();
        let mut pool = TxPool::new();
        let signed_tx = create_new_signed_tx(0, 0, U256::from(1), "pint", "chain");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(2);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
    }

    #[test]
    fn test_remove_tx_from_pool() {
        let mut factory = MockValidator::default();
        let mut pool = TxPool::new();
        let signed_tx = create_new_signed_tx(0, 1, U256::from(1), "pint", "chain");
        let recovered_signed_tx = signed_tx.into_recovered().unwrap();
        let vtx = factory.validate(recovered_signed_tx);
        let on_chain_balance = U256::from(2);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        
        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());

        pool.remove_transaction_by_id(vtx.tid());

        assert_eq!(0, pool.all_transaction.len());
        assert_eq!(0, pool.pending_pool.len());
    }
}