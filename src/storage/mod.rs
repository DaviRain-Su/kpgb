use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod ipfs;
pub mod github;
pub mod local;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub id: String,
    pub hash: String,
    pub size: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub content_type: String,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResult {
    pub id: String,
    pub url: Option<String>,
    pub metadata: StorageMetadata,
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn store(&self, content: &[u8], metadata: HashMap<String, String>) -> Result<StorageResult>;
    
    async fn retrieve(&self, id: &str) -> Result<Vec<u8>>;
    
    async fn exists(&self, id: &str) -> Result<bool>;
    
    async fn delete(&self, id: &str) -> Result<()>;
    
    async fn list(&self, prefix: Option<&str>) -> Result<Vec<StorageMetadata>>;
    
    fn storage_type(&self) -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum StorageBackend {
    Ipfs,
    GitHub,
    Local,
    S3,
}

#[derive(Clone)]
pub struct StorageManager {
    backends: std::sync::Arc<std::sync::Mutex<HashMap<StorageBackend, std::sync::Arc<dyn Storage>>>>,
    default_backend: StorageBackend,
}

impl StorageManager {
    pub fn new(default_backend: StorageBackend) -> Self {
        Self {
            backends: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            default_backend,
        }
    }
    
    pub fn add_backend(&mut self, backend_type: StorageBackend, backend: Box<dyn Storage>) {
        let mut backends = self.backends.lock().unwrap();
        backends.insert(backend_type, std::sync::Arc::from(backend));
    }
    
    pub fn get_backend(&self, backend_type: &StorageBackend) -> Option<std::sync::Arc<dyn Storage>> {
        let backends = self.backends.lock().unwrap();
        backends.get(backend_type).cloned()
    }
    
    pub fn default_backend(&self) -> std::sync::Arc<dyn Storage> {
        let backends = self.backends.lock().unwrap();
        backends.get(&self.default_backend)
            .expect("Default backend not configured")
            .clone()
    }
}