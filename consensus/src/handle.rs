use std::sync::Arc;

use primitives::handle::{ConsensusHandleMessage, Handle};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

#[derive(Debug, Clone)]
pub struct ConsensusHandle {
    inner: Arc<ConsensusInner>,
}

impl ConsensusHandle {
    pub fn new(tx: UnboundedSender<ConsensusHandleMessage>) -> Self {
        Self {
            inner: Arc::new(ConsensusInner { to_manager_tx: tx }),
        }
    }
}

impl Handle for ConsensusHandle {
    type Msg = ConsensusHandleMessage;

    fn send(&self, msg: Self::Msg) {
        if let Err(e) = self.inner.to_manager_tx.send(msg) {
            error!(
                error = ?e,
                "Failed to send ConsensusHandleMessage."
            );
        }
    }
}

#[derive(Debug)]
pub struct ConsensusInner {
    to_manager_tx: UnboundedSender<ConsensusHandleMessage>,
}
