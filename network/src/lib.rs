use std::net::{IpAddr, SocketAddr};

use primitives::{
    handle::{ConsensusHandleMessage, Handle, NetworkHandleMessage},
    types::BlockHash,
};

use provider::{DatabaseTrait, ProviderFactory};
use rand::{rng, seq::IndexedRandom};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tracing::{debug, error, info, warn};
use transaction_pool::{Pool, error::PoolErrorKind, identifier::TransactionOrigin};

use crate::{
    builder::{BootNode, NetworkConfig},
    handle::NetworkHandle,
    peer::PeerList,
};

pub mod builder;
pub mod error;
pub mod handle;
pub mod peer;

pub struct NetworkManager<DB: DatabaseTrait> {
    listener: TcpListener,
    pub provider: ProviderFactory<DB>,
    network_handle: NetworkHandle,
    from_handle_rx: UnboundedReceiverStream<NetworkHandleMessage>,
    pool: Pool<DB>,
    
    peers: PeerList,
    consensus: Box<dyn Handle<Msg = ConsensusHandleMessage>>,
    config: NetworkConfig,
}

impl<DB: DatabaseTrait + Sync + Send + 'static> NetworkManager<DB> {
    fn start_loop(self, is_boot_node: bool) {
        tokio::spawn(async move {
            info!("Network channel starts.");
            let mut this: NetworkManager<DB> = self;

            // Peer Connection Task loop
            let handle_cloned = this.network_handle.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    handle_cloned.send(NetworkHandleMessage::PeerConnectionTest);
                    info!("Start PeerConnectionTest");
                }
            });

            loop {
                tokio::select! {
                    // New Peer
                    Ok((mut socket, addr)) = this.listener.accept() => {
                        let peer_len = this.peers.len();
                        if peer_len >= this.config.max_peer_size {
                            info!("Can't accept a new peer. max_peer_size: {}", this.config.max_peer_size);

                            let redirect = {
                                let peers = this.peers.inner().read();
                                let mut rng = rng();
                                peers.choose(&mut rng).map(|p| p.addr().clone())
                            };

                            if let Some(socket_addr) = redirect{
                                let msg_string = socket_addr.to_string();

                                if let Err(e) = socket.write_all(msg_string.as_bytes()).await {
                                    error!(error = ?e, "Failed to send one-shot msg");
                                }
                            }
                        } else {

                            info!("New peer: {}", addr);

                            if let Err(e) = socket.write_all("Ok".as_bytes()).await {
                                error!(error = ?e, "Failed to send one-shot msg");

                                continue;
                            }

                            let (peer, pid) = this.peers.insert_new_peer(socket, addr, this.network_handle.clone());

                            peer.send(NetworkHandleMessage::Hello(pid, this.config.address, this.config.port));
                        }
                    }

                    // NetworkHandle Message
                    Some(msg) = this.from_handle_rx.next() => {
                        debug!("Received message: {}", msg);
                        match msg {
                            NetworkHandleMessage::PeerConnectionTest => {
                                let mut peers = this.peers.inner().write();

                                for peer in peers.iter_mut() {
                                    if peer.is_not_alive() { continue }; // PeerConnectionTest is ongoing already
                                    peer.set_alive_false();
                                    peer.send(NetworkHandleMessage::Ping(this.config.address, this.config.port));
                                    let peer_id = peer.id();
                                    let handle_cloned = this.network_handle.clone();
                                    tokio::task::spawn(async move {
                                        tokio::time::sleep(Duration::from_secs(10)).await;
                                        handle_cloned.send(NetworkHandleMessage::RemoveUnresponsivePeer(peer_id));
                                    });
                                }
                            }
                            NetworkHandleMessage::NewTransaction(signed) => {
                                let origin = TransactionOrigin::External;
                                let recovered = match signed.clone().into_recovered() {
                                    Ok(recovered) => recovered,
                                    Err(e) => {
                                        error!(error = ?e, "NewTransaction Recover Error.");
                                        continue;
                                    }
                                };
                                let recovered_cloned = recovered.clone();
                                let res = this.pool.add_transaction(origin, recovered);
                                if this.pool.check_pending_pool_len() >= 1 {
                                    this.consensus.send(ConsensusHandleMessage::NewTransaction(recovered_cloned));
                                }

                                match res {
                                    Ok(_tx_hash) => {
                                        // broadcast to peer
                                        for peer in this.peers.inner().read().iter() {
                                            peer.send(NetworkHandleMessage::NewTransaction(signed.clone()));
                                        }
                                    }
                                    Err(e) => {
                                        let kind = e.kind.clone();
                                        if matches!(kind, PoolErrorKind::AlreadyImported) {
                                            warn!("Already imported Tx: {:?}", &e.hash);
                                        } else {
                                            error!(error = ?e, "New Transaction Pool Error.");
                                        }


                                        continue;
                                    }
                                }
                            }
                            NetworkHandleMessage::NewPayload(block) => {
                                this.consensus.send(ConsensusHandleMessage::ImportBlock(block));
                            }

                            NetworkHandleMessage::BroadcastBlock(block) => {
                                for peer in this.peers.inner().read().iter() {
                                    peer.send(NetworkHandleMessage::NewPayload(block.clone()));
                                }
                            }

                            NetworkHandleMessage::RequestDataResponseFinished => {
                                info!("Finished Syncronizing");
                            }

                            NetworkHandleMessage::RequestDataResponse(from, address, port) => {
                                info!("RequestDataResponse is occured by {} {}", address, port);
                                let socket_addr = SocketAddr::from((address, port));
                                if let Some(peer) = this.peers.inner().read().iter().find(|peer| {
                                    *peer.addr() == socket_addr
                                }) {
                                    // send block datas
                                    let latest = this.provider.db().latest_block_number();
                                    for i in from..latest+1 {
                                        match this.provider.db().get_block(i) {
                                            Ok(block) => if let Some(bloc) = block {
                                                peer.send(NetworkHandleMessage::NewPayload(bloc));
                                            }
                                            Err(e) => {
                                                error!(error = ?e, "Failed to get block in db.");
                                                break;
                                            }
                                        }
                                    }
                                    info!("Block Sync Ok! {} {}", address, port);
                                    // peer.send(NetworkHandleMessage::RequestDataResponseFinished);
                                } else {
                                    info!("Can't find peer! {} {}", address, port);
                                }
                            }

                            // request db, pool data to
                            NetworkHandleMessage::RequestData(from) => {
                                if this.peers.len() == 0 {
                                    info!("Can't find peer.");
                                    continue;
                                }
                                let peer = &this.peers.inner().read()[0];

                                peer.send(NetworkHandleMessage::RequestDataResponse(from,this.config.address, this.config.port));
                                info!("Requested Data.");
                            }

                            NetworkHandleMessage::HandShake(pid, address, port) => {
                                let socket_addr = SocketAddr::from((address, port));
                                let mut binding = this.peers.inner().write();
                                let peer = match binding.iter_mut().find(|peer| {

                                    peer.id() == pid
                                }) {
                                    Some(peer) => peer,
                                    None => {
                                        warn!("Handshake: Can't find peer");
                                        continue;
                                    }
                                };
                                peer.update_addr(socket_addr);

                                info!("Handshake completed with {:?}", socket_addr);

                            }

                            NetworkHandleMessage::Hello(pid, address, port) => {
                                let socket_addr = SocketAddr::from((address, port));
                                let peer = match this.peers.find_peer_by_addr(socket_addr) {
                                    Some(peer) => peer,
                                    None => {
                                        warn!("Hello: Can't find peer while Hello");
                                        continue;
                                    }
                                };
                                peer.send(NetworkHandleMessage::HandShake(pid, this.config.address, this.config.port));

                                if !is_boot_node {
                                    info!("Try to synchronize db and mem-pool.");
                                    this.network_handle.send(NetworkHandleMessage::RequestData(1));
                                }
                            }

                            NetworkHandleMessage::RemovePeer(pid) => {
                                this.peers.remove_peer_by_id(pid);
                            }

                            NetworkHandleMessage::RemoveUnresponsivePeer(pid) => {
                                let peer = match this.peers.find_peer_by_id(pid) {
                                    Some(peer) => peer,
                                    None => {
                                        warn!("Can't find peer while RemoveUnresponsivePeer");
                                        continue;
                                    }
                                };
                                if peer.is_not_alive() {
                                    debug!("This peer is not alive. Remove pid:{}", pid);
                                    this.peers.remove_peer_by_id(pid);
                                }
                                else {
                                    debug!("This peer is alive. pid:{}", pid);
                                }
                            }

                            NetworkHandleMessage::BroadcastTransaction(signed) => {
                                for peer in this.peers.inner().read().iter() {
                                    peer.send(NetworkHandleMessage::NewTransaction(signed.clone()));
                                }
                            }
                            NetworkHandleMessage::ReorgChainData => {
                                if this.peers.len() == 0 {
                                    info!("Can't find peer.");
                                    continue;
                                }
                                let peer = &this.peers.inner().read()[0];
                                peer.send(NetworkHandleMessage::RequestChainData(this.config.address, this.config.port));
                            }

                            NetworkHandleMessage::RequestChainData(ip_addr, port) => {
                                let socket_addr = SocketAddr::from((ip_addr, port));
                                let binding = this.peers.inner().read();
                                let peer = match binding.iter().find(|peer| {
                                    *peer.addr() == socket_addr
                                }) {
                                    Some(peer) => peer,
                                    None => {
                                        warn!("RequestChainData: Can't find peer");
                                        continue;
                                    }
                                };

                                let latest_bno = this.provider.db().latest_block_number();
                                let mut block_hash_vec: Vec<BlockHash> = Vec::new();
                                let start_bno = if latest_bno >= 16 {
                                    latest_bno - 16
                                } else {
                                    0
                                };

                                for i in start_bno..latest_bno+1 {
                                    match this.provider.db().get_header(i) {
                                        Ok(header) => {
                                            if let Some(headr) = header {
                                                block_hash_vec.push(headr.calculate_hash());
                                            }
                                        }
                                        Err(e) => {
                                            error!(error = ?e, "RequestChainData: Can't get block hash.");
                                            break;
                                        }
                                    }
                                }

                                peer.send(NetworkHandleMessage::RespondChainDataResult(block_hash_vec.len() as u64, block_hash_vec));
                            }

                            NetworkHandleMessage::RespondChainDataResult(_len, hash_vec) => {
                                let mut found = false;
                                for hash in hash_vec.iter().rev() {
                                    match this.provider.db().get_block_by_hash(hash.clone()) {
                                        Ok(Some(block)) => {
                                            found = true;
                                            let height = block.header().height;
                                            // delete datas
                                            if let Err(e) = this.provider.db().remove_datas(height) {
                                                error!(error = ?e, "RequestChainData: Failed to clean db datas.");
                                                break;
                                            }
                                            // then request new data
                                            this.network_handle.send(NetworkHandleMessage::RequestData(height+1));
                                            break;
                                        }
                                        Ok(None) => {
                                            continue;
                                        }
                                        Err(_e) => {
                                            continue;
                                        }
                                    }
                                }

                                // reorg chain from scratch
                                if !found {
                                    if let Err(e) = this.provider.db().remove_datas(0) {
                                        error!(error = ?e, "RequestChainData: Failed to clean db datas.");
                                        break;
                                    }
                                    // then request new data
                                    this.network_handle.send(NetworkHandleMessage::RequestData(1));
                                }
                            }
                            NetworkHandleMessage::Ping (ip_addr, port) => {
                                let socket_addr = SocketAddr::from((ip_addr, port));
                                let peer = match this.peers.find_peer_by_addr(socket_addr) {
                                    Some(peer) => peer,
                                    None => {
                                        error!(addr = ?socket_addr, "Can't find this peer while Ping");
                                        continue;
                                    }
                                };

                                peer.send(NetworkHandleMessage::Pong(this.config.address, this.config.port));
                            }
                            NetworkHandleMessage::Pong(ip_addr, port)=> {
                                let socket_addr = SocketAddr::from((ip_addr, port));
                                this.peers.set_alive_true(socket_addr);
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn connect_with_boot_node(
        &mut self,
        _ip_addr: IpAddr,
        _port: u16,
        boot_node: &BootNode,
    ) {
        if boot_node.is_boot_node() {
            return;
        }

        let mut addr: SocketAddr = boot_node.socket_addr();
        for _ in 0..5 {
            match TcpStream::connect(addr).await {
                Ok(mut socket) => {
                    info!("Connected to node: {}", addr);
                    let mut buf = vec![0u8; 128];

                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            continue;
                        }
                        Ok(n) => {
                            if n < 2 {
                                continue;
                            };
                            // Ok
                            let msg = String::from_utf8_lossy(&buf[..2]);
                            if msg == "Ok" {
                                let (_peer, _) = self.peers.insert_new_peer(
                                    socket,
                                    addr,
                                    self.network_handle.clone(),
                                );
                                match NetworkHandleMessage::decode(&buf[2..n], addr) {
                                    Ok((res, _used)) => match res {
                                        Some(decoded) => {
                                            let _ = self.network_handle.send(decoded);
                                        }
                                        None => {}
                                    },

                                    Err(e) => {
                                        error!(error = ?e, "Failed to decode Network handle message from {:?}.", addr);
                                    }
                                };
                                break;
                            }
                            let msg = String::from_utf8_lossy(&buf[..n]);
                            info!("Boot node redirect: {}", msg);

                            if let Ok(new_addr) = msg.parse::<SocketAddr>() {
                                addr = new_addr;
                                continue;
                            } else {
                                error!(msg = ?msg, "Invalid redirect address received.");
                            }
                        }
                        Err(e) => {
                            error!(error = ?e, "Failed to read from node {}", addr);
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!(error = ?e, "Failed to connect to the node {}", addr);
                    break;
                }
            }
        }
    }
}

impl<DB: DatabaseTrait + std::fmt::Debug> std::fmt::Debug for NetworkManager<DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkManager")
            .field("listener", &self.listener)
            .field("handle", &self.network_handle)
            .field("from_handle_rx", &self.from_handle_rx)
            .field("pool", &self.pool)
            .field("peers", &self.peers)
            .field("config", &self.config)
            .finish()
    }
}

#[derive(Debug)]
pub struct NoopConsensusHandle;

impl Handle for NoopConsensusHandle {
    type Msg = ConsensusHandleMessage;
    fn send(&self, _block: Self::Msg) {
        // Do nothing
    }
}
