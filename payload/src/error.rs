use provider::error::ProviderError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PayloadBuilderError {
    #[error("Provider Error")]
    ProviderError(ProviderError),
}

impl From<ProviderError> for PayloadBuilderError {
    fn from(value: ProviderError) -> Self {
        Self::ProviderError(value)
    }
}
