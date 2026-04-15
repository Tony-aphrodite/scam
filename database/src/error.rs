use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Failed to encode block")]
    BlockEncodeError,
    #[error("Requested data does not exist in database")]
    DataNotExists,
    #[error("Database error itself")]
    DBError,
    #[error("Cannot Remove! Only latest can be removed")]
    CannotRemove,
}
