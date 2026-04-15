pub mod handle;

use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};

use primitives::{
    handle::{MinerHandleMessage, MinerResultMessage},
    types::B256,
};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info};

use crate::miner::handle::MinerHandle;

#[derive(Debug)]
pub struct Miner {
    miner_rx: UnboundedReceiver<MinerHandleMessage>,
    consensus_tx: UnboundedSender<MinerResultMessage>,
    epoch: Arc<AtomicU64>,
    worker: Arc<AtomicUsize>,
}

impl Miner {
    pub fn new(
        miner_rx: UnboundedReceiver<MinerHandleMessage>,
        consensus_tx: UnboundedSender<MinerResultMessage>,
    ) -> Self {
        Self {
            miner_rx,
            consensus_tx,
            epoch: Default::default(),
            worker: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn start_channel(self) {
        tokio::spawn(async move {
            info!("Miner channel starts.");
            let Miner {
                mut miner_rx,
                consensus_tx,
                epoch,
                worker,
            } = self;
            let mut token: Option<tokio_util::sync::CancellationToken> = None;
            loop {
                if let Some(msg) = miner_rx.recv().await {
                    debug!("Miner received message. {}", msg);
                    match msg {
                        MinerHandleMessage::NewPayload(payload_header) => {
                            // spawn payload mining task
                            let consensus_tx = consensus_tx.clone();
                            let _epoch = epoch.clone();

                            // this order should be same as header hash function
                            let mut hasher = Sha256::new();
                            hasher.update(payload_header.previous_hash.hash());
                            hasher.update(payload_header.transaction_root);
                            hasher.update(payload_header.state_root);
                            hasher.update(payload_header.timestamp.to_be_bytes());
                            hasher.update(payload_header.proposer.get_addr());
                            hasher.update(payload_header.difficulty.to_be_bytes());
                            hasher.update(payload_header.height.to_be_bytes());

                            token = Some(tokio_util::sync::CancellationToken::new());
                            let child = token.as_ref().unwrap().child_token();

                            worker.fetch_add(1, Ordering::Relaxed);
                            let worker_cloned = worker.clone();

                            tokio::task::spawn_blocking(move || {
                                let mut nonce: u64 = 0;
                                let difficulty = payload_header.difficulty;
                                loop {
                                    if nonce % 10000 == 0 && child.is_cancelled() {
                                        worker_cloned.fetch_sub(1, Ordering::Relaxed);
                                        if let Err(e) =
                                            consensus_tx.send(MinerResultMessage::MiningHalted)
                                        {
                                            error!(error = ?e, "Failed to send MingResultMessage.");
                                        }
                                        return;
                                    }
                                    let mut new_hasher = hasher.clone();
                                    new_hasher.update(nonce.to_be_bytes());
                                    let result = B256::from_slice(&new_hasher.finalize());
                                    if meets_target(result, difficulty) {
                                        // Mining Ok!
                                        worker_cloned.fetch_sub(1, Ordering::Relaxed);
                                        let header = payload_header.clone().into_header(nonce);
                                        if let Err(e) = consensus_tx
                                            .send(MinerResultMessage::MiningSuccess(header))
                                        {
                                            error!(error = ?e, "Failed to send MingResultMessage.");
                                        }
                                        return;
                                    }
                                    nonce += 1;
                                }
                            });
                        }
                        MinerHandleMessage::HaltMining => {
                            if worker.load(Ordering::Relaxed) == 0 {
                                if let Err(e) = consensus_tx.send(MinerResultMessage::MiningHalted)
                                {
                                    error!(error = ?e, "Failed to send MingResultMessage.");
                                }
                            } else {
                                // MinerResultMessage will be sent by mining task!
                                token.map(|token| token.cancel());
                                token = None;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn build_miner() -> (MinerHandle, UnboundedReceiver<MinerResultMessage>) {
        let (miner_tx, miner_rx) = mpsc::unbounded_channel::<MinerHandleMessage>();
        let (consensus_tx, consensus_rx) = mpsc::unbounded_channel::<MinerResultMessage>();

        let miner_handle = MinerHandle::new(miner_tx);
        let miner = Miner::new(miner_rx, consensus_tx);

        miner.start_channel();
        (miner_handle, consensus_rx)
    }
}

fn meets_target(result: B256, difficulty: u32) -> bool {
    let mut remains = difficulty;

    for byte in result.0 {
        if remains >= 8 {
            if byte != 0 {
                return false;
            } else {
                remains -= 8;
            }
        } else if remains > 0 {
            let mask = 0xFF << (8 - remains);
            if byte & mask != 0 {
                return false;
            } else {
                return true;
            }
        } else {
            return true;
        }
    }

    remains == 0
}
