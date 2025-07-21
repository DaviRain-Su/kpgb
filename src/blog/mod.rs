use crate::database::Database;
use crate::models::BlogPost;
use crate::storage::StorageManager;
use anyhow::Result;
use std::collections::HashMap;

pub struct BlogManager {
    storage_manager: StorageManager,
    database: Database,
}

impl BlogManager {
    pub async fn new(storage_manager: StorageManager, database_url: &str) -> Result<Self> {
        let database = Database::new(database_url).await?;
        Ok(Self {
            storage_manager,
            database,
        })
    }

    pub async fn create_post(&mut self, post: BlogPost) -> Result<String> {
        // Check for duplicate content
        if let Some(existing_storage_id) = self
            .database
            .get_post_by_content_hash(&post.content_hash)
            .await?
        {
            return Ok(existing_storage_id);
        }

        // Store in default storage backend
        let storage = self.storage_manager.default_backend();

        let post_json = serde_json::to_string_pretty(&post)?;
        let mut metadata = HashMap::new();
        metadata.insert("content_type".to_string(), "application/json".to_string());
        metadata.insert("post_id".to_string(), post.id.clone());
        metadata.insert("slug".to_string(), post.slug.clone());

        let result = storage.store(post_json.as_bytes(), metadata).await?;

        // Save to database
        self.database.insert_post(&post, &result.id).await?;

        Ok(result.id)
    }

    pub async fn get_post(&self, storage_id: &str) -> Result<BlogPost> {
        // Try database first
        if let Some(post) = self.database.get_post_by_storage_id(storage_id).await? {
            return Ok(post);
        }

        // Fallback to storage
        let storage = self.storage_manager.default_backend();
        let content = storage.retrieve(storage_id).await?;
        let post: BlogPost = serde_json::from_slice(&content)?;
        Ok(post)
    }

    pub async fn update_post(&mut self, _storage_id: &str, mut post: BlogPost) -> Result<String> {
        post.updated_at = chrono::Utc::now();

        // Check for duplicate content
        if let Some(existing_storage_id) = self
            .database
            .get_post_by_content_hash(&post.content_hash)
            .await?
        {
            return Ok(existing_storage_id);
        }

        self.create_post(post).await
    }

    pub async fn publish_post(&mut self, storage_id: &str) -> Result<()> {
        self.database
            .update_post_published(storage_id, true)
            .await?;
        Ok(())
    }

    pub async fn list_posts(&self, published_only: bool) -> Result<Vec<(String, BlogPost)>> {
        self.database.list_posts(published_only).await
    }

    pub async fn search_posts(&self, query: &str) -> Result<Vec<(String, BlogPost)>> {
        self.database.search_posts(query).await
    }
}
