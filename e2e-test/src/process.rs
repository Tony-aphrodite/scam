use std::net::IpAddr;

use network::builder::NetworkConfig;
use node::{builder::LaunchContext, configs::BlockConfig};
use primitives::types::Address;
use tokio::signal;
use tracing::{error, info};

pub struct NodeConfig {
    pub name: String,
    pub address: IpAddr,
    pub port: u16,
    pub rpc_port: u16,
    pub miner_address: String,
    pub boot_node: bool,
}

pub fn build_source() {}

// Launch with InMemoryDB
pub async fn launch_test_node(config: NodeConfig) -> anyhow::Result<()> {
    let NodeConfig {
        name,
        address,
        port,
        rpc_port,
        miner_address,
        boot_node,
    } = config;

    let in_memory = true;
    let miner_address = Address::from_hex(miner_address)?;

    let mut network_config = NetworkConfig::new(address, port, rpc_port);
    network_config.boot_node.is_boot_node = boot_node;
    let block_config = BlockConfig::new(miner_address);
    let launch_context = LaunchContext::new(network_config.clone(), block_config, in_memory);

    let node = match launch_context.launch().await {
        Ok(node) => node,
        Err(err) => {
            error!(error = ?err, "Failed to launch PintChain Node.");
            return Err(err.into());
        }
    };

    info!("[ Name: {} ] PintChain Node launcing Ok.", name);

    tokio::task::spawn(async move {
        tokio::select! {
            _ = node.run_rpc(network_config) => {
                info!("Rpc Server has been shutdown");
            },
            _ = signal::ctrl_c() => {
                info!("Ctrl_C: Gracefully shutdown Node..");
            }
        }
    });

    Ok(())
}
