use std::sync::Arc;

use primitives::handle::{Handle, NetworkHandleMessage};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

#[derive(Clone, Debug)]
pub struct NetworkHandle {
    inner: Arc<NetworkInner>,
}

impl NetworkHandle {
    pub fn new(tx: UnboundedSender<NetworkHandleMessage>) -> Self {
        Self {
            inner: Arc::new(NetworkInner { to_manager_tx: tx }),
        }
    }
}

impl Handle for NetworkHandle {
    type Msg = NetworkHandleMessage;

    fn send(&self, msg: Self::Msg) {
        if let Err(e) = self.inner.to_manager_tx.send(msg) {
            error!(error = ?e, "Failed to send NetworkHandleMessage.");
        }
    }
}

#[derive(Debug)]
pub struct NetworkInner {
    to_manager_tx: UnboundedSender<NetworkHandleMessage>,
}
