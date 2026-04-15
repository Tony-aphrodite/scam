use std::sync::Arc;

use consensus::{ConsensusEngine, handle::ConsensusHandle, miner::Miner};
use database::{DBImpl, immemorydb::InMemoryDB, mdbx::MDBX};
use network::{
    builder::{NetworkBuilder, NetworkConfig},
    handle::NetworkHandle,
};
use payload::PayloadBuilder;
use primitives::handle::{ConsensusHandleMessage, NetworkHandleMessage};
use provider::ProviderFactory;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::info;
use transaction_pool::Pool;

use crate::{
    Node,
    configs::{BlockConfig, ExecConfig, PoolConfig, RpcConfig},
    error::NodeLaunchError,
};

pub struct LaunchContext {
    pub block_config: BlockConfig,
    pub pool_config: PoolConfig,
    pub network_config: NetworkConfig,
    pub rpc_config: RpcConfig,
    pub exec_config: ExecConfig,
    pub in_memory_db: bool,
}

impl LaunchContext {
    pub fn new(
        network_config: NetworkConfig,
        block_config: BlockConfig,
        in_memory_db: bool,
    ) -> Self {
        Self {
            block_config,
            pool_config: PoolConfig::default(),
            network_config,
            rpc_config: RpcConfig::default(),
            exec_config: ExecConfig::default(),
            in_memory_db: in_memory_db,
        }
    }
}

impl LaunchContext {
    pub async fn launch(self) -> Result<Node<DBImpl>, NodeLaunchError> {
        let Self {
            network_config,
            block_config,
            in_memory_db,
            ..
        } = self;
        // Build Provider

        let db = if !in_memory_db {
            info!("DB Launched with MDBX.");
            DBImpl::MDBX(MDBX::genesis_state())
        } else {
            info!("DB Launched with InMemoryDB.");
            DBImpl::InMemoryDB(Arc::new(InMemoryDB::genesis_state()))
        };
        let provider = ProviderFactory::new(db);
        // Build Pool
        let pool = Pool::new(provider.clone());
        // Build PayloadBuilder
        let builder =
            PayloadBuilder::new(block_config.miner_address, provider.clone(), pool.clone());
        let (builder_handle, builder_rx) = builder.start_builder();
        // Build Network
        let (tx, rx) = mpsc::unbounded_channel::<NetworkHandleMessage>();
        let rx_stream = UnboundedReceiverStream::new(rx);
        let network_handle = NetworkHandle::new(tx);
        // Build Miner
        let (miner_handle, miner_rx) = Miner::build_miner();

        // Build Consensus
        let (tx, consensus_rx) = mpsc::unbounded_channel::<ConsensusHandleMessage>();
        let consensus_handle = ConsensusHandle::new(tx);

        let consensus = ConsensusEngine::new(
            pool.clone(),
            builder_handle,
            Box::new(network_handle.clone()),
            provider.clone(),
            miner_handle,
            miner_rx,
            builder_rx,
        );

        let network_handle = NetworkBuilder::start_network(
            network_handle,
            rx_stream,
            Box::new(consensus_handle.clone()),
            pool.clone(),
            provider.clone(),
            network_config,
        )
        .await?;

        let consensus_handle = consensus.start_consensus(consensus_handle, consensus_rx);

        Ok(Node {
            provider,
            pool,
            consensus: Box::new(consensus_handle),
            network: Box::new(network_handle),
        })
    }
}
