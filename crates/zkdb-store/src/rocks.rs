use crate::{Store, StoreError, StoreResult};
use async_trait::async_trait;
use rocksdb::{Options, DB};
use std::path::Path;

pub struct RocksStore {
    db: DB,
}

impl RocksStore {
    /// Creates a new RocksDB store at the specified path
    pub fn new<P: AsRef<Path>>(path: P) -> StoreResult<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = DB::open(&opts, path).map_err(|e| StoreError::Storage(e.to_string()))?;

        Ok(Self { db })
    }
}

#[async_trait]
impl Store for RocksStore {
    async fn put(&self, key: &str, value: &[u8]) -> StoreResult<()> {
        self.db
            .put(key.as_bytes(), value)
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, key: &str) -> StoreResult<Vec<u8>> {
        self.db
            .get(key.as_bytes())
            .map_err(|e| StoreError::Storage(e.to_string()))?
            .ok_or_else(|| StoreError::NotFound(key.to_string()))
    }

    async fn delete(&self, key: &str) -> StoreResult<()> {
        self.db
            .delete(key.as_bytes())
            .map_err(|e| StoreError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> StoreResult<bool> {
        let exists = self
            .db
            .get(key.as_bytes())
            .map_err(|e| StoreError::Storage(e.to_string()))?
            .is_some();
        Ok(exists)
    }
}

impl Drop for RocksStore {
    fn drop(&mut self) {
        // RocksDB will flush and close automatically
    }
}
