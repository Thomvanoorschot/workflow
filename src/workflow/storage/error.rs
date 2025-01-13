use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    Database(sqlx::Error),
    Serialization(serde_json::Error),
    UuidParse(uuid::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::UuidParse(e) => write!(f, "UUID parsing error: {}", e),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(e) => Some(e),
            Self::Serialization(e) => Some(e),
            Self::UuidParse(e) => Some(e),
        }
    }
}

impl From<sqlx::Error> for StorageError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

impl From<uuid::Error> for StorageError {
    fn from(err: uuid::Error) -> Self {
        Self::UuidParse(err)
    }
}
