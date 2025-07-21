use crate::models::BlogPost;
use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::time::Duration;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn insert_post(&self, post: &BlogPost, storage_id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Insert post
        sqlx::query(
            r#"
            INSERT INTO posts (id, storage_id, title, slug, content, excerpt, author, 
                             content_hash, created_at, updated_at, published, category)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
        )
        .bind(&post.id)
        .bind(storage_id)
        .bind(&post.title)
        .bind(&post.slug)
        .bind(&post.content)
        .bind(&post.excerpt)
        .bind(&post.author)
        .bind(&post.content_hash)
        .bind(post.created_at)
        .bind(post.updated_at)
        .bind(post.published)
        .bind(&post.category)
        .execute(&mut *tx)
        .await?;

        // Insert tags
        for tag in &post.tags {
            // Insert tag if not exists
            sqlx::query("INSERT OR IGNORE INTO tags (name) VALUES (?1)")
                .bind(tag)
                .execute(&mut *tx)
                .await?;

            // Get tag id
            let tag_id: i64 = sqlx::query_scalar("SELECT id FROM tags WHERE name = ?1")
                .bind(tag)
                .fetch_one(&mut *tx)
                .await?;

            // Link post and tag
            sqlx::query("INSERT INTO post_tags (post_id, tag_id) VALUES (?1, ?2)")
                .bind(&post.id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_post_by_storage_id(&self, storage_id: &str) -> Result<Option<BlogPost>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, slug, content, excerpt, author,
                   created_at, updated_at, published, category, storage_id, content_hash
            FROM posts
            WHERE storage_id = ?1
            "#,
        )
        .bind(storage_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let mut post = BlogPost {
                id: row.get("id"),
                title: row.get("title"),
                slug: row.get("slug"),
                content: row.get("content"),
                excerpt: row.get("excerpt"),
                author: row.get("author"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                published: row.get("published"),
                tags: Vec::new(),
                category: row.get("category"),
                storage_id: row.get("storage_id"),
                content_hash: row.get("content_hash"),
            };

            // Load tags
            let tags: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT t.name
                FROM tags t
                JOIN post_tags pt ON t.id = pt.tag_id
                WHERE pt.post_id = ?1
                "#,
            )
            .bind(&post.id)
            .fetch_all(&self.pool)
            .await?;

            post.tags = tags;
            Ok(Some(post))
        } else {
            Ok(None)
        }
    }

    pub async fn get_post_by_content_hash(&self, content_hash: &str) -> Result<Option<String>> {
        let storage_id = sqlx::query_scalar("SELECT storage_id FROM posts WHERE content_hash = ?1")
            .bind(content_hash)
            .fetch_optional(&self.pool)
            .await?;

        Ok(storage_id)
    }

    pub async fn update_post_published(&self, storage_id: &str, published: bool) -> Result<()> {
        sqlx::query("UPDATE posts SET published = ?1, updated_at = ?2 WHERE storage_id = ?3")
            .bind(published)
            .bind(chrono::Utc::now())
            .bind(storage_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list_posts(&self, published_only: bool) -> Result<Vec<(String, BlogPost)>> {
        let query = if published_only {
            r#"
            SELECT id, title, slug, content, excerpt, author,
                   created_at, updated_at, published, category, storage_id, content_hash
            FROM posts
            WHERE published = 1
            ORDER BY created_at DESC
            "#
        } else {
            r#"
            SELECT id, title, slug, content, excerpt, author,
                   created_at, updated_at, published, category, storage_id, content_hash
            FROM posts
            ORDER BY created_at DESC
            "#
        };

        let rows = sqlx::query(query).fetch_all(&self.pool).await?;

        let mut results = Vec::new();
        for row in rows {
            let mut post = BlogPost {
                id: row.get("id"),
                title: row.get("title"),
                slug: row.get("slug"),
                content: row.get("content"),
                excerpt: row.get("excerpt"),
                author: row.get("author"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                published: row.get("published"),
                tags: Vec::new(),
                category: row.get("category"),
                storage_id: Some(row.get("storage_id")),
                content_hash: row.get("content_hash"),
            };

            let storage_id = post.storage_id.clone().unwrap_or_default();

            // Load tags
            let tags: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT t.name
                FROM tags t
                JOIN post_tags pt ON t.id = pt.tag_id
                WHERE pt.post_id = ?1
                "#,
            )
            .bind(&post.id)
            .fetch_all(&self.pool)
            .await?;

            post.tags = tags;
            results.push((storage_id, post));
        }

        Ok(results)
    }

    pub async fn search_posts(&self, query: &str) -> Result<Vec<(String, BlogPost)>> {
        // Use FTS5 for full-text search
        let rows = sqlx::query(
            r#"
            SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.author,
                   p.created_at, p.updated_at, p.published, p.category, p.storage_id, p.content_hash
            FROM posts p
            JOIN posts_fts ON p.rowid = posts_fts.rowid
            WHERE posts_fts MATCH ?1
            ORDER BY rank
            "#,
        )
        .bind(query)
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let mut post = BlogPost {
                id: row.get("id"),
                title: row.get("title"),
                slug: row.get("slug"),
                content: row.get("content"),
                excerpt: row.get("excerpt"),
                author: row.get("author"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                published: row.get("published"),
                tags: Vec::new(),
                category: row.get("category"),
                storage_id: Some(row.get("storage_id")),
                content_hash: row.get("content_hash"),
            };

            let storage_id = post.storage_id.clone().unwrap_or_default();

            // Load tags
            let tags: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT t.name
                FROM tags t
                JOIN post_tags pt ON t.id = pt.tag_id
                WHERE pt.post_id = ?1
                "#,
            )
            .bind(&post.id)
            .fetch_all(&self.pool)
            .await?;

            post.tags = tags;
            results.push((storage_id, post));
        }

        Ok(results)
    }

    pub async fn get_all_tags(&self) -> Result<Vec<(String, i64)>> {
        let rows = sqlx::query(
            r#"
            SELECT t.name, COUNT(pt.post_id) as post_count
            FROM tags t
            LEFT JOIN post_tags pt ON t.id = pt.tag_id
            LEFT JOIN posts p ON pt.post_id = p.id
            WHERE p.published = 1 OR p.published IS NULL
            GROUP BY t.id, t.name
            ORDER BY post_count DESC, t.name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let name: String = row.get("name");
            let count: i64 = row.get("post_count");
            results.push((name, count));
        }

        Ok(results)
    }

    pub async fn get_posts_by_tag(&self, tag: &str, published_only: bool) -> Result<Vec<(String, BlogPost)>> {
        let query = if published_only {
            r#"
            SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.author,
                   p.created_at, p.updated_at, p.published, p.category, p.storage_id, p.content_hash
            FROM posts p
            JOIN post_tags pt ON p.id = pt.post_id
            JOIN tags t ON pt.tag_id = t.id
            WHERE t.name = ?1 AND p.published = 1
            ORDER BY p.created_at DESC
            "#
        } else {
            r#"
            SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.author,
                   p.created_at, p.updated_at, p.published, p.category, p.storage_id, p.content_hash
            FROM posts p
            JOIN post_tags pt ON p.id = pt.post_id
            JOIN tags t ON pt.tag_id = t.id
            WHERE t.name = ?1
            ORDER BY p.created_at DESC
            "#
        };

        let rows = sqlx::query(query).bind(tag).fetch_all(&self.pool).await?;

        let mut results = Vec::new();
        for row in rows {
            let mut post = BlogPost {
                id: row.get("id"),
                title: row.get("title"),
                slug: row.get("slug"),
                content: row.get("content"),
                excerpt: row.get("excerpt"),
                author: row.get("author"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                published: row.get("published"),
                tags: Vec::new(),
                category: row.get("category"),
                storage_id: Some(row.get("storage_id")),
                content_hash: row.get("content_hash"),
            };

            let storage_id = post.storage_id.clone().unwrap_or_default();

            // Load all tags for this post
            let tags: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT t.name
                FROM tags t
                JOIN post_tags pt ON t.id = pt.tag_id
                WHERE pt.post_id = ?1
                "#,
            )
            .bind(&post.id)
            .fetch_all(&self.pool)
            .await?;

            post.tags = tags;
            results.push((storage_id, post));
        }

        Ok(results)
    }
}
