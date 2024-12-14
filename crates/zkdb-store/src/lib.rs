use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Value not found for key: {0}")]
    NotFound(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

impl From<std::io::Error> for StoreError {
    fn from(err: std::io::Error) -> Self {
        StoreError::Io(err.to_string())
    }
}

pub type StoreResult<T> = Result<T, StoreError>;

#[async_trait]
pub trait Store: Send + Sync {
    /// Store a value and return its location reference
    async fn put(&self, key: &str, value: &[u8]) -> StoreResult<()>;

    /// Retrieve a value by its key
    async fn get(&self, key: &str) -> StoreResult<Vec<u8>>;

    /// Delete a value by its key
    async fn delete(&self, key: &str) -> StoreResult<()>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> StoreResult<bool>;
}

/// Basic file-based implementation
pub mod file;
/// RocksDB-based implementation
pub mod rocks;
