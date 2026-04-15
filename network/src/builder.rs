use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use primitives::handle::{ConsensusHandleMessage, Handle, NetworkHandleMessage};
use provider::{DatabaseTrait, ProviderFactory};
use tokio::net::TcpListener;
use tokio_stream::wrappers::UnboundedReceiverStream;
use transaction_pool::Pool;

use crate::{NetworkHandle, NetworkManager, error::NetworkStartError, peer::PeerList};

pub struct NetworkBuilder;

impl NetworkBuilder {
    pub async fn start_network<DB: DatabaseTrait + Send + Sync + 'static>(
        network_handle: NetworkHandle,
        rx_stream: UnboundedReceiverStream<NetworkHandleMessage>,
        consensus: Box<dyn Handle<Msg = ConsensusHandleMessage>>,
        pool: Pool<DB>,
        provider: ProviderFactory<DB>,
        cfg: NetworkConfig,
    ) -> Result<NetworkHandle, NetworkStartError> {
        // Server Binding
        let listener = match TcpListener::bind((cfg.address, cfg.port)).await {
            Ok(listener) => listener,
            Err(err) => return Err(NetworkStartError::LinstenerBindingError(err)),
        };

        let mut network_manager = NetworkManager {
            listener,
            provider,
            network_handle: network_handle.clone(),
            from_handle_rx: rx_stream,
            pool,
            peers: PeerList::new(),
            consensus,
            config: cfg.clone(),
        };

        // Finding peer from Boot Node
        // Initially, I implemented function that connects only boot node and never fails.
        network_manager
            .connect_with_boot_node(cfg.address, cfg.port, &cfg.boot_node)
            .await;
        // Network loop Start
        network_manager.start_loop(cfg.boot_node.is_boot_node());

        Ok(network_handle)
    }
}

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub address: IpAddr,
    pub port: u16,
    pub rpc_port: u16,
    pub max_peer_size: usize,
    pub boot_node: BootNode,
}

impl NetworkConfig {
    pub fn new(address: IpAddr, port: u16, rpc_port: u16) -> Self {
        Self {
            address,
            port,
            rpc_port,
            max_peer_size: 2,
            boot_node: BootNode::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BootNode {
    pub is_boot_node: bool,
    address: IpAddr,
    port: u16,
}

impl BootNode {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }

    pub fn is_boot_node(&self) -> bool {
        self.is_boot_node
    }
}

impl Default for BootNode {
    fn default() -> Self {
        Self {
            is_boot_node: true,
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 33333,
        }
    }
}
