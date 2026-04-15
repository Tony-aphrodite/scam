use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;
use database::mdbx::get_db_path;
use network::builder::NetworkConfig;
use node::{builder::LaunchContext, configs::BlockConfig};
use primitives::types::Address;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::init::init_txs;
mod init;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    address: IpAddr,

    #[arg(short, long, default_value_t = 33333)]
    port: u16,

    #[arg(short, long, default_value_t = 8888)]
    rpc_port: u16,

    // Default Miner Address: 28dcb1338b900419cd613a8fb273ae36e7ec2b1c
    #[arg(short, long, default_value_t = String::from("28dcb1338b900419cd613a8fb273ae36e7ec2b1c"))]
    miner_address: String,

    #[arg(long, default_value_t = false)]
    boot_node: bool,

    #[arg(short, long, default_value_t = false)]
    in_memory_db: bool,

    #[arg(long, default_value_t = false)]
    test: bool,

    #[arg(long, default_value_t = false)]
    remove_data: bool,

    #[arg(short, long, default_value_t = String::from("boot_node"))]
    name: String,
}

#[tokio::main]
async fn main() {
    // Enable backtraces if not already set
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    // Use colored formatting
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_level(true)
        .init();

    let args = Args::parse();
    info!(node_name = &args.name, "Trying to launch PintChain Node.");

    if args.remove_data {
        let pathbuf = get_db_path();
        if pathbuf.exists() {
            std::fs::remove_dir_all(&pathbuf).expect("Failed to remove DB directory");
        }
        info!("Database removed successfully");
    }

    // Initialize node
    let miner_address = Address::from_hex(args.miner_address).expect("Invalid miner address");
    let mut network_config = NetworkConfig::new(args.address, args.port, args.rpc_port);
    network_config.boot_node.is_boot_node = args.boot_node;
    let block_config = BlockConfig::new(miner_address);
    let launch_context = LaunchContext::new(network_config.clone(), block_config, args.in_memory_db);

    let node = match launch_context.launch().await {
        Ok(n) => n,
        Err(err) => {
            error!(error = ?err, "Failed to launch PintChain Node");
            return;
        }
    };

    info!("[Name: {}] Node launched successfully.", args.name);

    if args.boot_node && args.test {
        init_txs(&node);
    }

    // Graceful shutdown
    tokio::select! {
        _ = node.run_rpc(network_config) => {
            info!("RPC server shut down");
        },
        _ = signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }
}
