use crate::{Store, StoreError, StoreResult};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct FileStore {
    base_path: PathBuf,
}

impl FileStore {
    pub async fn new<P: AsRef<Path>>(base_path: P) -> StoreResult<Self> {
        let base_path = base_path.as_ref().to_owned();
        fs::create_dir_all(&base_path).await?;
        Ok(Self { base_path })
    }

    fn key_to_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }

    async fn ensure_parent_exists(&self, path: &Path) -> StoreResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Store for FileStore {
    async fn put(&self, key: &str, value: &[u8]) -> StoreResult<()> {
        let path = self.key_to_path(key);
        self.ensure_parent_exists(&path).await?;
        fs::write(path, value).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> StoreResult<Vec<u8>> {
        let path = self.key_to_path(key);
        fs::read(path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StoreError::NotFound(key.to_string()),
            _ => StoreError::Io(e.to_string()),
        })
    }

    async fn delete(&self, key: &str) -> StoreResult<()> {
        let path = self.key_to_path(key);
        fs::remove_file(path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StoreError::NotFound(key.to_string()),
            _ => StoreError::Io(e.to_string()),
        })
    }

    async fn exists(&self, key: &str) -> StoreResult<bool> {
        let path = self.key_to_path(key);
        Ok(path.exists())
    }
}
