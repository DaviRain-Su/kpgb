use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use super::{Storage, StorageMetadata, StorageResult};
use crate::constants::{
    CONTENT_TYPE_OCTET_STREAM, DEFAULT_IPFS_API_URL, ERROR_IPFS_IMMUTABLE, METADATA_CONTENT_TYPE,
};
use sha2::{Digest, Sha256};

pub struct IpfsStorage {
    api_url: String,
    client: reqwest::Client,
}

impl IpfsStorage {
    pub fn new(api_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder().no_proxy().build()?;

        Ok(Self {
            api_url: api_url.to_string(),
            client,
        })
    }

    pub fn from_env() -> Result<Self> {
        let api_url =
            std::env::var("IPFS_API_URL").unwrap_or_else(|_| DEFAULT_IPFS_API_URL.to_string());
        Self::new(&api_url)
    }

    async fn ipfs_add(&self, content: Vec<u8>) -> Result<String> {
        let form =
            reqwest::multipart::Form::new().part("file", reqwest::multipart::Part::bytes(content));

        let response = self
            .client
            .post(format!("{}/api/v0/add", self.api_url))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS add failed: {}", response.status()));
        }

        let result: serde_json::Value = response.json().await?;
        let hash = result["Hash"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No hash in IPFS response"))?;

        Ok(hash.to_string())
    }

    async fn ipfs_cat(&self, cid: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .post(format!("{}/api/v0/cat?arg={}", self.api_url, cid))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS cat failed: {}", response.status()));
        }

        Ok(response.bytes().await?.to_vec())
    }

    async fn ipfs_pin(&self, cid: &str) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/api/v0/pin/add?arg={}", self.api_url, cid))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS pin failed: {}", response.status()));
        }

        Ok(())
    }
}

#[async_trait]
impl Storage for IpfsStorage {
    async fn store(
        &self,
        content: &[u8],
        metadata: HashMap<String, String>,
    ) -> Result<StorageResult> {
        // Calculate content hash
        let hash = Sha256::digest(content);
        let hash_str = hex::encode(hash);

        // Add to IPFS
        let cid = self.ipfs_add(content.to_vec()).await?;

        // Pin the content
        self.ipfs_pin(&cid).await?;

        Ok(StorageResult {
            id: cid.clone(),
            url: Some(format!("ipfs://{cid}")),
            metadata: StorageMetadata {
                id: cid,
                hash: hash_str,
                size: content.len(),
                created_at: chrono::Utc::now(),
                content_type: metadata
                    .get(METADATA_CONTENT_TYPE)
                    .unwrap_or(&CONTENT_TYPE_OCTET_STREAM.to_string())
                    .clone(),
                extra: metadata,
            },
        })
    }

    async fn retrieve(&self, id: &str) -> Result<Vec<u8>> {
        self.ipfs_cat(id).await
    }

    async fn exists(&self, id: &str) -> Result<bool> {
        let response = self
            .client
            .post(format!("{}/api/v0/object/stat?arg={}", self.api_url, id))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn delete(&self, _id: &str) -> Result<()> {
        Err(anyhow::anyhow!(ERROR_IPFS_IMMUTABLE))
    }

    async fn list(&self, _prefix: Option<&str>) -> Result<Vec<StorageMetadata>> {
        // List pinned content
        let response = self
            .client
            .post(format!("{}/api/v0/pin/ls", self.api_url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("IPFS pin ls failed: {}", response.status()));
        }

        let result: serde_json::Value = response.json().await?;
        let mut metadata_list = Vec::new();

        if let Some(keys) = result["Keys"].as_object() {
            for (cid, _) in keys {
                metadata_list.push(StorageMetadata {
                    id: cid.clone(),
                    hash: String::new(),
                    size: 0,
                    created_at: chrono::Utc::now(),
                    content_type: "unknown".to_string(),
                    extra: HashMap::new(),
                });
            }
        }

        Ok(metadata_list)
    }

    fn storage_type(&self) -> &'static str {
        "ipfs"
    }
}
