use std::io::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkStartError {
    #[error("Listener Binding Error")]
    LinstenerBindingError(Error),
}
