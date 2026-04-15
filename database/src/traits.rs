use std::{collections::HashMap};

use primitives::{block::{Block, Header}, transaction::SignedTransaction, types::{Account, Address, BlockHash, TxHash}, world::World};

use crate::error::DatabaseError;

pub trait DatabaseTrait: Send + Sync + Clone + 'static + Sized {
    fn latest_block_number(&self) -> u64;
    fn basic(&self, address: &Address) -> Result<Option<Account>, Box<dyn std::error::Error>>;
    fn get_state(&self, block_no: u64) -> Result<(Option<HashMap<Address, Account>>, Option<World>), Box<dyn std::error::Error>>;
    fn get_block(&self, block_no: u64) -> Result<Option<Block>, Box<dyn std::error::Error>>;
    fn get_block_by_hash(&self, hash: BlockHash) -> Result<Option<Block>, Box<dyn std::error::Error>>;
    fn get_transaction_by_hash(&self, hash: TxHash) -> Result<Option<(SignedTransaction, u64)>, Box<dyn std::error::Error>>;
    fn get_header(&self, block_no: u64) -> Result<Option<Header>, Box<dyn std::error::Error>>;
    fn update(&self, new_account_state: HashMap<Address, Account>, new_field_state: World, new_block: Block)
        -> Result<(), Box<dyn std::error::Error>>;
    fn get_latest_block_header(&self) -> Header;
    // remove all datas in front of height
    fn remove_datas(&self, height: u64) -> Result<(), Box<dyn std::error::Error>> {
        let latest = self.latest_block_number();
        for cur in (height + 1)..(latest + 1) {
            if let Err(_e) = self.remove_data(cur) {
                return Err(Box::new(DatabaseError::DBError));
            }
        }

        Ok(())
    }
    fn remove_data(&self, height: u64) -> Result<(), Box<dyn std::error::Error>>;
}