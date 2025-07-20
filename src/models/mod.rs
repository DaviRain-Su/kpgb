use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published: bool,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub storage_id: Option<String>,
    pub content_hash: String,
}

impl BlogPost {
    pub fn new(title: String, content: String, author: String) -> Self {
        let slug = Self::generate_slug(&title);
        let now = Utc::now();
        let content_hash = Self::calculate_hash(&content);
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            slug,
            content,
            excerpt: None,
            author,
            created_at: now,
            updated_at: now,
            published: false,
            tags: Vec::new(),
            category: None,
            storage_id: None,
            content_hash,
        }
    }
    
    pub fn generate_slug(title: &str) -> String {
        title.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
    
    pub fn calculate_hash(content: &str) -> String {
        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(content.as_bytes());
        hex::encode(hash)
    }
    
    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.content_hash = Self::calculate_hash(&self.content);
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogMetadata {
    pub total_posts: usize,
    pub published_posts: usize,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub authors: Vec<String>,
    pub last_updated: DateTime<Utc>,
}