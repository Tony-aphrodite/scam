use std::sync::Arc;

use primitives::handle::{Handle, PayloadBuilderHandleMessage};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

#[derive(Debug, Clone)]
pub struct PayloadBuilderHandle {
    inner: Arc<PayloadBuilderInner>,
}

impl PayloadBuilderHandle {
    pub fn new(to_manager_tx: UnboundedSender<PayloadBuilderHandleMessage>) -> Self {
        Self {
            inner: Arc::new(PayloadBuilderInner { to_manager_tx }),
        }
    }
}

impl Handle for PayloadBuilderHandle {
    type Msg = PayloadBuilderHandleMessage;

    fn send(&self, msg: Self::Msg) {
        if let Err(e) = self.inner.to_manager_tx.send(msg) {
            error!(error = ?e, "Failed to send PayloadBuilderHandleMessage.");
        }
    }
}

#[derive(Debug)]
pub struct PayloadBuilderInner {
    to_manager_tx: UnboundedSender<PayloadBuilderHandleMessage>,
}
