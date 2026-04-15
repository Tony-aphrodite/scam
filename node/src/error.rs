use network::error::NetworkStartError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeLaunchError {
    #[error("Network Start Error")]
    NetworkStartError(NetworkStartError),
}

impl From<NetworkStartError> for NodeLaunchError {
    fn from(value: NetworkStartError) -> Self {
        Self::NetworkStartError(value)
    }
}
