use anyhow::Result;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Storage, StorageMetadata, StorageResult};

#[derive(Debug, Serialize, Deserialize)]
struct GitHubContent {
    message: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<String>,
}

pub struct GitHubStorage {
    client: reqwest::Client,
    owner: String,
    repo: String,
    branch: String,
    token: String,
}

impl GitHubStorage {
    pub fn new(owner: String, repo: String, branch: String, token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            owner,
            repo,
            branch,
            token,
        }
    }
    
    pub fn from_env() -> Result<Self> {
        Ok(Self::new(
            std::env::var("GITHUB_OWNER")?,
            std::env::var("GITHUB_REPO")?,
            std::env::var("GITHUB_BRANCH").unwrap_or_else(|_| "main".to_string()),
            std::env::var("GITHUB_TOKEN")?,
        ))
    }
    
    fn api_url(&self, path: &str) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.repo, path
        )
    }
}

#[async_trait]
impl Storage for GitHubStorage {
    async fn store(&self, content: &[u8], metadata: HashMap<String, String>) -> Result<StorageResult> {
        let path = metadata.get("path")
            .ok_or_else(|| anyhow::anyhow!("GitHub storage requires 'path' in metadata"))?;
        
        let encoded_content = general_purpose::STANDARD.encode(content);
        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(content);
        let hash_str = hex::encode(hash);
        
        let github_content = GitHubContent {
            message: metadata.get("message")
                .unwrap_or(&format!("Add content to {}", path))
                .clone(),
            content: encoded_content,
            sha: None,
        };
        
        let response = self.client
            .put(&self.api_url(path))
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&github_content)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("GitHub API error: {}", response.status()));
        }
        
        let result: serde_json::Value = response.json().await?;
        let sha = result["commit"]["sha"].as_str()
            .ok_or_else(|| anyhow::anyhow!("No SHA in GitHub response"))?;
        
        Ok(StorageResult {
            id: sha.to_string(),
            url: Some(format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                self.owner, self.repo, self.branch, path
            )),
            metadata: StorageMetadata {
                id: sha.to_string(),
                hash: hash_str,
                size: content.len(),
                created_at: chrono::Utc::now(),
                content_type: metadata.get("content_type")
                    .unwrap_or(&"application/octet-stream".to_string())
                    .clone(),
                extra: metadata,
            },
        })
    }
    
    async fn retrieve(&self, id: &str) -> Result<Vec<u8>> {
        let response = self.client
            .get(&self.api_url(id))
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3.raw")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("GitHub API error: {}", response.status()));
        }
        
        let content = response.bytes().await?;
        Ok(content.to_vec())
    }
    
    async fn exists(&self, id: &str) -> Result<bool> {
        let response = self.client
            .head(&self.api_url(id))
            .header("Authorization", format!("token {}", self.token))
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
    
    async fn delete(&self, _id: &str) -> Result<()> {
        Err(anyhow::anyhow!("GitHub storage deletion not implemented"))
    }
    
    async fn list(&self, prefix: Option<&str>) -> Result<Vec<StorageMetadata>> {
        let path = prefix.unwrap_or("");
        let response = self.client
            .get(&self.api_url(path))
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("GitHub API error: {}", response.status()));
        }
        
        Ok(Vec::new())
    }
    
    fn storage_type(&self) -> &'static str {
        "github"
    }
}