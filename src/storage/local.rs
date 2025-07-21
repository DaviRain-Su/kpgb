use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

use super::{Storage, StorageMetadata, StorageResult};

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    fn get_path(&self, id: &str) -> PathBuf {
        self.base_path.join(id)
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn store(
        &self,
        content: &[u8],
        metadata: HashMap<String, String>,
    ) -> Result<StorageResult> {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(content);
        let hash_str = hex::encode(hash);

        let id = metadata.get("filename").unwrap_or(&hash_str).clone();

        let file_path = self.get_path(&id);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&file_path, content).await?;

        Ok(StorageResult {
            id: id.clone(),
            url: Some(format!("file://{}", file_path.display())),
            metadata: StorageMetadata {
                id,
                hash: hash_str,
                size: content.len(),
                created_at: chrono::Utc::now(),
                content_type: metadata
                    .get("content_type")
                    .unwrap_or(&"application/octet-stream".to_string())
                    .clone(),
                extra: metadata,
            },
        })
    }

    async fn retrieve(&self, id: &str) -> Result<Vec<u8>> {
        let file_path = self.get_path(id);
        let content = fs::read(&file_path).await?;
        Ok(content)
    }

    async fn exists(&self, id: &str) -> Result<bool> {
        let file_path = self.get_path(id);
        Ok(file_path.exists())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let file_path = self.get_path(id);
        fs::remove_file(&file_path).await?;
        Ok(())
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<StorageMetadata>> {
        let search_path = match prefix {
            Some(p) => self.base_path.join(p),
            None => self.base_path.clone(),
        };

        let mut metadata_list = Vec::new();
        let mut entries = fs::read_dir(&search_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                let path = entry.path();
                let metadata = entry.metadata().await?;

                metadata_list.push(StorageMetadata {
                    id: path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    hash: String::new(),
                    size: metadata.len() as usize,
                    created_at: metadata
                        .created()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
                        .unwrap_or_else(chrono::Utc::now),
                    content_type: "application/octet-stream".to_string(),
                    extra: HashMap::new(),
                });
            }
        }

        Ok(metadata_list)
    }

    fn storage_type(&self) -> &'static str {
        "local"
    }
}
