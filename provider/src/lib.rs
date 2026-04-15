pub mod error;
pub mod executor;
pub mod state;

pub use database::traits::DatabaseTrait;
use primitives::{
    block::Block,
    types::{Account, Address},
};
use std::sync::Arc;
use tracing::{error, info};

use crate::{error::ProviderError, executor::Executor, state::ExecutableState};

#[derive(Debug, Clone)]
pub struct ProviderFactory<DB: DatabaseTrait> {
    db: DB,
}

impl<DB: DatabaseTrait + Clone> ProviderFactory<DB> {
    pub fn get_next_difficulty(&self) -> u32 {
        let latest_header = self.db().get_latest_block_header();
        let prev_difficulty: u32 = latest_header.difficulty;
        // genesis
        if latest_header.height == 0 {
            return prev_difficulty;
        }
        let prev_header = match self.db().get_header(latest_header.height - 1) {
            Ok(header) => match header {
                Some(header) => header,
                None => return prev_difficulty,
            },
            Err(_e) => return prev_difficulty,
        };
        let time = latest_header.timestamp - prev_header.timestamp;

        let new_difficulty = if time <= 10 {
            prev_difficulty + 1
        } else if time <= 15 {
            prev_difficulty
        } else {
            prev_difficulty.saturating_sub(1)
        };
        new_difficulty
    }

    pub fn new(db: DB) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DB {
        &self.db
    }

    pub fn block_number(&self) -> u64 {
        self.db.latest_block_number()
    }

    pub fn latest(&self) -> Provider<DB> {
        let block_no = self.db.latest_block_number();
        self.state_by_block_number(block_no)
    }

    fn state_by_block_number(&self, block_no: u64) -> Provider<DB> {
        Provider {
            db: self.db.clone(),
            block_no: block_no,
        }
    }

    pub fn import_new_block(&self, block: Block) -> Result<(), ProviderError> {
        // execute state
        let provider = self.latest();
        let state = match provider.executable_state() {
            Ok(state) => state,
            Err(e) => {
                error!(error = ?e, "Failed to create new executor.");
                return Err(e);
            }
        };

        let mut executor = Executor::new(state);

        let (new_account_state, new_field_state) = match executor.execute_block(&block) {
            Ok((account, field)) => (account, field),
            Err(e) => {
                error!(error = ?e, "Failed to execute block.");
                return Err(ProviderError::ExecutionError(e));
            }
        };

        // update results
        info!("Imported New Block. {}", &block);
        let _ = self.db.update(new_account_state, new_field_state, block);
        Ok(())
    }
}

pub struct Provider<DB: DatabaseTrait> {
    db: DB,
    block_no: u64,
}

impl<DB: DatabaseTrait> Provider<DB> {
    pub fn basic_account(
        &self,
        address: Address,
    ) -> Result<Option<Account>, Box<dyn std::error::Error>> {
        Ok(self.db.basic(&address)?)
    }

    pub fn executable_state(&self) -> Result<ExecutableState, ProviderError> {
        let (accounts_base, field_base) = match self.db.get_state(self.block_no) {
            Ok((account, field)) => (account, field),
            Err(e) => return Err(ProviderError::DatabaseError(e)),
        };

        if accounts_base.is_none() || field_base.is_none() {
            return Err(ProviderError::StateNotExist(self.block_no));
        }
        // unwrap() is safe!
        let accounts_write = accounts_base.clone().unwrap();
        let accounts_base = Arc::new(accounts_base.unwrap());

        let field_write = field_base.clone().unwrap();
        let field_base = Arc::new(field_base.unwrap());

        Ok(ExecutableState {
            accounts_base,
            accounts_write,
            field_base,
            field_write,
        })
    }
}
