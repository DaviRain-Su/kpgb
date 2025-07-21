pub mod config;
pub mod generator;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub author: String,
    pub base_url: String,
    pub base_path: Option<String>,
    pub ipfs_gateway: String,
    pub posts_per_page: usize,
    pub enable_rss: bool,
    pub theme: String,
}

impl SiteConfig {
    pub fn path(&self, path: &str) -> String {
        if let Some(base_path) = &self.base_path {
            if path.starts_with('/') {
                format!("{}{}", base_path.trim_end_matches('/'), path)
            } else {
                format!("{}/{}", base_path.trim_end_matches('/'), path)
            }
        } else {
            path.to_string()
        }
    }
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "My IPFS Blog".to_string(),
            description: "A decentralized blog powered by IPFS".to_string(),
            author: "Anonymous".to_string(),
            base_url: "http://localhost:8080".to_string(),
            base_path: None,
            ipfs_gateway: "http://localhost:8080/ipfs/".to_string(),
            posts_per_page: 10,
            enable_rss: true,
            theme: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageContext {
    pub site: SiteConfig,
    pub page_title: String,
    pub content: String,
}
