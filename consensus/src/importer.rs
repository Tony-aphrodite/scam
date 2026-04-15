use primitives::{
    block::{Block, BlockValidationResult},
    error::{BlockImportError, BlockValidatioError},
};
use provider::{DatabaseTrait, ProviderFactory, executor::Executor};

#[derive(Debug)]
pub struct BlockImporter<DB: DatabaseTrait> {
    provider: ProviderFactory<DB>,
}

impl<DB: DatabaseTrait> BlockImporter<DB> {
    pub fn new(provider: ProviderFactory<DB>) -> Self {
        Self { provider }
    }

    pub fn import_new_block(&self, block: Block) -> Result<(), BlockImportError> {
        if block.header.height > self.provider.block_number() + 1 {
            return Err(BlockImportError::BlockHeightError);
        }
        if block.header.height != self.provider.block_number() + 1 {
            return Err(BlockImportError::AlreadyImportedBlock);
        }
        if block.header().previous_hash
            != self
                .provider
                .db()
                .get_latest_block_header()
                .calculate_hash()
        {
            return Err(BlockImportError::NotChainedBlock);
        }
        let res = self.validate_block(&block)?;
        if res.success {
            if let Err(_e) = self.provider.import_new_block(block) {
                return Err(BlockImportError::ProviderError);
            }
        }

        Ok(())
    }

    fn validate_block(&self, block: &Block) -> Result<BlockValidationResult, BlockImportError> {
        // validate block with no state
        let mut result: BlockValidationResult = self.validate_block_with_no_state(&block)?;

        let state_provider = self.provider.latest();
        let executable_state = match state_provider.executable_state() {
            Ok(exec_state) => exec_state,
            Err(_e) => return Err(BlockImportError::ProviderError),
        };

        let mut executor = Executor::new(executable_state);

        // validate block with state
        match executor.execute_block(&block) {
            Ok((_, _)) => {
                result.success();
            }

            Err(_e) => {
                result.failed();
                result.add_error(BlockValidatioError::ExecutionError);
            }
        }

        Ok(result)
    }

    fn validate_block_with_no_state(
        &self,
        _block: &Block,
    ) -> Result<BlockValidationResult, BlockImportError> {
        let success = true;
        let error: Option<BlockValidatioError> = None;

        Ok(BlockValidationResult { success, error })
    }
}
