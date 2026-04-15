use primitives::types::Address;

#[derive(Default)]
pub struct PoolConfig {}
#[derive(Default)]
pub struct RpcConfig {}
#[derive(Default)]
pub struct BlockConfig {
    pub miner_address: Address
}

impl BlockConfig {
    pub fn new(miner_address: Address) -> Self {
        Self { miner_address }
    }
}
#[derive(Default)]
pub struct ExecConfig {}
