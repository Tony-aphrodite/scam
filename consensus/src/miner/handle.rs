use std::sync::Arc;

use primitives::handle::{Handle, MinerHandleMessage};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

#[derive(Debug, Clone)]
pub struct MinerHandle {
    inner: Arc<MinerInner>,
}

impl MinerHandle {
    pub fn new(miner_tx: UnboundedSender<MinerHandleMessage>) -> Self {
        Self {
            inner: Arc::new(MinerInner {
                to_manager_tx: miner_tx,
            }),
        }
    }
}

impl Handle for MinerHandle {
    type Msg = MinerHandleMessage;

    fn send(&self, msg: Self::Msg) {
        if let Err(_e) = self.inner.to_manager_tx.send(msg) {
            error!("Failed to send MinerHandleMessage");
        }
    }
}

#[derive(Debug)]
pub struct MinerInner {
    to_manager_tx: UnboundedSender<MinerHandleMessage>,
}
