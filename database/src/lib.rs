use std::sync::Arc;

use primitives::{
    block::{Block, Header},
    transaction::SignedTransaction,
};

use crate::{immemorydb::InMemoryDB, mdbx::MDBX, traits::DatabaseTrait};

pub mod error;
pub mod genesis;
pub mod immemorydb;
pub mod mdbx;
pub mod traits;

#[derive(Clone, Debug)]
pub enum DBImpl {
    MDBX(MDBX),
    InMemoryDB(Arc<InMemoryDB>),
}

impl DatabaseTrait for DBImpl {
    fn latest_block_number(&self) -> u64 {
        match self {
            DBImpl::MDBX(db) => db.latest_block_number(),
            DBImpl::InMemoryDB(db) => db.latest_block_number(),
        }
    }

    fn basic(
        &self,
        address: &primitives::types::Address,
    ) -> Result<Option<primitives::types::Account>, Box<dyn std::error::Error>> {
        match self {
            DBImpl::MDBX(db) => db.basic(address),
            DBImpl::InMemoryDB(db) => db.basic(address),
        }
    }

    fn get_state(
        &self,
        block_no: u64,
    ) -> Result<
        (
            Option<
                std::collections::HashMap<primitives::types::Address, primitives::types::Account>,
            >,
            Option<primitives::world::World>,
        ),
        Box<dyn std::error::Error>,
    > {
        match self {
            DBImpl::MDBX(db) => db.get_state(block_no),
            DBImpl::InMemoryDB(db) => db.get_state(block_no),
        }
    }

    fn get_block(
        &self,
        block_no: u64,
    ) -> Result<Option<Block>, Box<dyn std::error::Error + 'static>> {
        match self {
            DBImpl::MDBX(db) => db.get_block(block_no),
            DBImpl::InMemoryDB(db) => db.get_block(block_no),
        }
    }

    fn get_header(
        &self,
        block_no: u64,
    ) -> Result<Option<Header>, Box<dyn std::error::Error + 'static>> {
        match self {
            DBImpl::MDBX(db) => db.get_header(block_no),
            DBImpl::InMemoryDB(db) => db.get_header(block_no),
        }
    }

    fn update(
        &self,
        new_account_state: std::collections::HashMap<
            primitives::types::Address,
            primitives::types::Account,
        >,
        new_field_state: primitives::world::World,
        new_block: primitives::block::Block,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DBImpl::MDBX(db) => db.update(new_account_state, new_field_state, new_block),
            DBImpl::InMemoryDB(db) => db.update(new_account_state, new_field_state, new_block),
        }
    }

    fn get_latest_block_header(&self) -> primitives::block::Header {
        match self {
            DBImpl::MDBX(db) => db.get_latest_block_header(),
            DBImpl::InMemoryDB(db) => db.get_latest_block_header(),
        }
    }

    fn get_block_by_hash(
        &self,
        hash: primitives::types::BlockHash,
    ) -> Result<Option<Block>, Box<dyn std::error::Error + 'static>> {
        match self {
            DBImpl::MDBX(db) => db.get_block_by_hash(hash),
            DBImpl::InMemoryDB(db) => db.get_block_by_hash(hash),
        }
    }

    fn remove_datas(&self, height: u64) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DBImpl::MDBX(db) => db.remove_datas(height),
            DBImpl::InMemoryDB(db) => db.remove_datas(height),
        }
    }

    fn remove_data(&self, height: u64) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DBImpl::MDBX(db) => db.remove_data(height),
            DBImpl::InMemoryDB(db) => db.remove_data(height),
        }
    }

    fn get_transaction_by_hash(
        &self,
        hash: primitives::types::TxHash,
    ) -> Result<Option<(SignedTransaction, u64)>, Box<dyn std::error::Error + 'static>> {
        match self {
            DBImpl::MDBX(db) => db.get_transaction_by_hash(hash),
            DBImpl::InMemoryDB(db) => db.get_transaction_by_hash(hash),
        }
    }
}
