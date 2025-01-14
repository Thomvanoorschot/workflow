use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Bincode error: {0}")]
    Bincode(#[from] Box<bincode::ErrorKind>),
}
