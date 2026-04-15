use std::{net::SocketAddr, sync::Arc};

use parking_lot::RwLock;
use primitives::handle::Handle;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    select,
    sync::mpsc::{self, UnboundedSender},
};
use tracing::{debug, error, info};

use crate::{NetworkHandle, NetworkHandleMessage};

#[derive(Debug, Clone)]
pub struct Peer {
    id: u64,
    addr: SocketAddr,
    tx: UnboundedSender<NetworkHandleMessage>,
    alive: bool,
}

impl Peer {
    pub fn new(id: u64, addr: SocketAddr, tx: UnboundedSender<NetworkHandleMessage>) -> Self {
        Self {
            id,
            addr,
            tx,
            alive: true,
        }
    }

pub fn send(&self, msg: NetworkHandleMessage) -> () {
    info!("send {:?} with {}", self.addr, msg);
    if let Err(e) = self.tx.send(msg) {
        error!(
            error = ?e,
            "Failed to send NetworkHandleMessage."
        );
    }
}

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn update_addr(&mut self, addr: SocketAddr) {
        self.addr = addr;
    }

    pub fn set_alive_false(&mut self) {
        self.alive = false;
    }

    pub fn set_alive_true(&mut self) {
        self.alive = true;
    }

    pub fn is_not_alive(&self) -> bool {
        !self.alive
    }
}

#[derive(Debug, Clone)]
pub struct PeerList {
    pub submission_id: u64,
    pub peers: Arc<RwLock<Vec<Peer>>>,
}

impl PeerList {
    pub fn new() -> Self {
        Self {
            submission_id: 0,
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn inner(&self) -> &RwLock<Vec<Peer>> {
        &self.peers
    }

    pub fn len(&self) -> usize {
        self.peers.read().len()
    }

    pub fn find_peer_by_addr(&self, addr: SocketAddr) -> Option<Peer> {
        let peers = self.peers.read();
        for peer in peers.iter() {
            if peer.addr == addr {
                return Some(peer.clone());
            }
        }
        None
    }

    pub fn set_alive_true(&mut self, addr: SocketAddr) {
        let mut peers = self.peers.write();
        for peer in peers.iter_mut() {
            if peer.addr == addr {
                peer.set_alive_true();
            }
        }
    }

    pub fn find_peer_by_id(&self, id: u64) -> Option<Peer> {
        let peers = self.peers.read();
        for peer in peers.iter() {
            if peer.id() == id {
                return Some(peer.clone());
            }
        }
        None
    }

    pub fn remove_peer_by_id(&mut self, pid: u64) {
        let mut peers = self.inner().write();
        peers.retain(|peer| peer.id != pid);
    }

    pub fn get_id(&mut self) -> u64 {
        let sub_id = self.submission_id;
        self.submission_id += 1;
        sub_id
    }
}

impl PeerList {
    pub fn insert_new_peer(
        &mut self,
        socket: TcpStream,
        addr: SocketAddr,
        network_handle: NetworkHandle,
    ) -> (Peer, u64) {
        let pid = self.get_id();
        let (tx, mut rx) = mpsc::unbounded_channel::<NetworkHandleMessage>();
        // tx is used for every componets who want to send peer msg
        // rx isolates socket
        let mut peers = self.peers.write();
        let new_peer = Peer::new(pid as u64, addr.clone(), tx);
        peers.push(new_peer.clone());

        let (mut read_socket, mut write_socket) = socket.into_split();

        // incoming loop
        let incoming = async move {
            info!("Peer {:?} incoming task has spawned.", addr);
            let mut buf = [0u8; 1024];
            loop {
                match read_socket.read(&mut buf).await {
                    Ok(0) => {
                        info!("Peer {:?} closed connection", addr);
                        network_handle.send(NetworkHandleMessage::RemovePeer(pid));
                        break;
                    }
                    Ok(n) => {
                        debug!("encoded {} data incomed", n);
                        let mut off: usize = 0;

                        while off < n {
                            match NetworkHandleMessage::decode(&buf[off..n], addr) {
                                Ok((res, used)) => match res {
                                    Some(decoded) => {
                                        let _ = network_handle.send(decoded);
                                        off += used;
                                    }
                                    None => {
                                        // It often occurs.. so it is not warning maybe..
                                        // warn!( addr = ?addr, "Invalid Request from Peer");
                                        off += used;
                                    }
                                },

                                Err(e) => {
                                    error!(
                                        error = ?e,
                                        "Failed to decode Network handle message from {:?}", addr
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            error = ?e,
                            "read error from {:?}", addr
                        );
                        break;
                    }
                }
            }
        };

        // outgoing loop
        let outgoing = async move {
            info!("Peer {:?} outgoing task has spawned.", addr);
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write_socket.write_all(&msg.encode()).await {
                    error!(
                        error = ?e,
                        "Failed to send to {:?}", addr
                    );
                    break;
                }
            }
        };

        let peers_ref = self.peers.clone();

        tokio::spawn(async move {
            select! {
                _ = incoming => {},
                _ = outgoing => {}
            }

            info!("Peer {:?} disconnected.", addr);
            let mut peers = peers_ref.write();
            peers.retain(|peer| peer.addr != addr);
        });

        (new_peer, pid as u64)
    }
}
