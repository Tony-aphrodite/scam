use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use parking_lot::RwLock;
use primitives::{
    block::{Block, Header},
    transaction::SignedTransaction,
    types::{Account, Address},
    world::World,
};
use tracing::warn;

use crate::{error::DatabaseError, genesis::genesis_accounts_info, traits::DatabaseTrait};

#[derive(Debug)]
pub struct InMemoryDB {
    accounts: RwLock<BTreeMap<u64, HashMap<Address, Account>>>,
    field: RwLock<BTreeMap<u64, World>>,
    blockchain: RwLock<BTreeMap<u64, Block>>,
    latest: RwLock<u64>,
}

impl InMemoryDB {
    // Addr: 28dcb1338b900419cd613a8fb273ae36e7ec2b1d, Seed: pint
    // Addr: 0534501c34f5a0f3fa43dc5d78e619be7edfa21a, Seed: chain
    pub fn genesis_state() -> Self {
        let mut db = Self::new();
        for (address, account) in genesis_accounts_info() {
            db.add_account(address, account).unwrap();
        }
        db
    }

    pub fn new() -> Self {
        let mut accounts: BTreeMap<u64, HashMap<Address, Account>> = BTreeMap::new();
        accounts.insert(0 as u64, HashMap::new());

        let mut field: BTreeMap<u64, World> = BTreeMap::new();
        field.insert(0, World::new());

        let mut blockchain: BTreeMap<u64, Block> = BTreeMap::new();
        let genesis_block = Block::genesis_block();
        blockchain.insert(0, genesis_block);

        Self {
            accounts: RwLock::new(accounts),
            field: RwLock::new(field),
            blockchain: RwLock::new(blockchain),
            latest: RwLock::new(0),
        }
    }

    pub fn add_account(
        &mut self,
        address: Address,
        account: Account,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.accounts.write();

        let latest_accounts = state.entry(*self.latest.read()).or_default();
        latest_accounts.insert(address, account);

        Ok(())
    }
}

impl DatabaseTrait for Arc<InMemoryDB> {
    fn latest_block_number(&self) -> u64 {
        *self.latest.read()
    }

    fn basic(&self, address: &Address) -> Result<Option<Account>, Box<dyn std::error::Error>> {
        let mut state = self.accounts.write();

        let latest_accounts: &mut HashMap<Address, Account> =
            state.entry(self.latest_block_number()).or_default();
        Ok(latest_accounts.get(address).or(None).cloned())
    }

    fn get_state(
        &self,
        block_no: u64,
    ) -> Result<(Option<HashMap<Address, Account>>, Option<World>), Box<dyn std::error::Error>>
    {
        let accounts = self.accounts.read();
        let mut account_base = None;
        if let Some(state_account) = accounts.get(&block_no) {
            account_base = Some(state_account.clone());
        }

        let field = self.field.read();
        let mut field_base = None;
        if let Some(state_field) = field.get(&block_no) {
            field_base = Some(state_field.clone());
        }

        Ok((account_base, field_base))
    }

    fn get_block(
        &self,
        block_no: u64,
    ) -> Result<Option<Block>, Box<dyn std::error::Error + 'static>> {
        let blockchain = self.blockchain.read();
        if let Some(block) = blockchain.get(&block_no) {
            Ok(Some(block.clone()))
        } else {
            Err(Box::new(DatabaseError::DataNotExists))
        }
    }

    fn get_block_by_hash(
        &self,
        hash: primitives::types::BlockHash,
    ) -> Result<Option<Block>, Box<dyn std::error::Error + 'static>> {
        let blockchain = self.blockchain.read();
        for (_heignt, block) in blockchain.iter() {
            let tmp_hash = block.header().calculate_hash();
            if hash == tmp_hash {
                return Ok(Some(block.clone()));
            }
        }
        Err(Box::new(DatabaseError::DataNotExists))
    }

    fn get_header(
        &self,
        block_no: u64,
    ) -> Result<Option<Header>, Box<dyn std::error::Error + 'static>> {
        let blockchain = self.blockchain.read();
        if let Some(block) = blockchain.get(&block_no) {
            Ok(Some(block.header().clone()))
        } else {
            Err(Box::new(DatabaseError::DataNotExists))
        }
    }

    fn get_latest_block_header(&self) -> primitives::block::Header {
        let blockchain = self.blockchain.read();
        let latest = self.latest_block_number();
        let block = blockchain.get(&latest).unwrap();
        block.header.clone()
    }

    fn update(
        &self,
        new_account_state: HashMap<Address, Account>,
        new_field_state: World,
        block: Block,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut latest = self.latest.write();
        *latest += 1;
        let mut state = self.accounts.write();
        state.insert(*latest, new_account_state);

        let mut field = self.field.write();
        field.insert(*latest, new_field_state);

        let mut blockchain = self.blockchain.write();
        blockchain.insert(*latest, block);
        warn!(block_height = ?latest, "DB updated new block.");

        Ok(())
    }

    fn remove_data(&self, height: u64) -> Result<(), Box<dyn std::error::Error>> {
        let latest = self.latest.read();
        if *latest != height {
            return Err(Box::new(DatabaseError::CannotRemove));
        }
        let cur = *latest;

        let mut accounts = self.accounts.write();
        accounts.remove(&cur);
        let mut field = self.field.write();
        field.remove(&cur);
        let mut blockchain = self.blockchain.write();
        blockchain.remove(&cur);
        let mut latest = self.latest.write();
        *latest -= 1;

        Ok(())
    }

    fn get_transaction_by_hash(
        &self,
        hash: primitives::types::TxHash,
    ) -> Result<Option<(SignedTransaction, u64)>, Box<dyn std::error::Error + 'static>> {
        let blockchain = self.blockchain.read();
        for (bno, block) in blockchain.iter() {
            for tx in block.body.iter() {
                if hash == tx.hash {
                    return Ok(Some((tx.clone(), *bno)));
                }
            }
        }
        Ok(None)
    }
}
