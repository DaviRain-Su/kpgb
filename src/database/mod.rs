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

    async fn load_tags_for_post(&self, post_id: &str) -> Result<Vec<String>> {
        let tags: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT t.name
            FROM tags t
            JOIN post_tags pt ON t.id = pt.tag_id
            WHERE pt.post_id = ?1
            "#,
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
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
            post.tags = self.load_tags_for_post(&post.id).await?;
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

    pub async fn update_post(&self, post: &BlogPost) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Update post content and metadata
        sqlx::query(
            r#"
            UPDATE posts 
            SET title = ?1, content = ?2, excerpt = ?3, author = ?4, 
                category = ?5, updated_at = ?6, content_hash = ?7
            WHERE id = ?8
            "#,
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(&post.excerpt)
        .bind(&post.author)
        .bind(&post.category)
        .bind(chrono::Utc::now())
        .bind(BlogPost::calculate_hash(&post.content))
        .bind(&post.id)
        .execute(&mut *tx)
        .await?;

        // Delete existing tags
        sqlx::query("DELETE FROM post_tags WHERE post_id = ?1")
            .bind(&post.id)
            .execute(&mut *tx)
            .await?;

        // Insert new tags
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

    pub async fn delete_post(&self, post_id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete post tags first (foreign key constraint)
        sqlx::query("DELETE FROM post_tags WHERE post_id = ?1")
            .bind(post_id)
            .execute(&mut *tx)
            .await?;

        // Delete the post
        sqlx::query("DELETE FROM posts WHERE id = ?1")
            .bind(post_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
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
            post.tags = self.load_tags_for_post(&post.id).await?;
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
            post.tags = self.load_tags_for_post(&post.id).await?;
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

    pub async fn get_posts_by_tag(
        &self,
        tag: &str,
        published_only: bool,
    ) -> Result<Vec<(String, BlogPost)>> {
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

    pub async fn get_related_posts(
        &self,
        post_id: &str,
        tags: &[String],
        category: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, BlogPost)>> {
        // Build query to find related posts based on:
        // 1. Shared tags (weighted by number of shared tags)
        // 2. Same category
        // 3. Recent posts

        // Simpler query that works with SQLite
        let query = r#"
            SELECT DISTINCT 
                p.id,
                p.title,
                p.slug,
                p.content,
                p.excerpt,
                p.author,
                p.created_at,
                p.updated_at,
                p.published,
                p.category,
                p.storage_id,
                p.content_hash,
                (
                    -- Count shared tags
                    SELECT COUNT(DISTINCT t2.name)
                    FROM tags t1
                    JOIN post_tags pt1 ON t1.id = pt1.tag_id
                    JOIN post_tags pt2 ON pt2.tag_id = t1.id
                    JOIN tags t2 ON t2.id = pt2.tag_id
                    WHERE pt1.post_id = ?1 AND pt2.post_id = p.id
                ) * 2 +
                -- Same category bonus
                CASE WHEN p.category = ?2 AND p.category IS NOT NULL THEN 1 ELSE 0 END AS relevance_score
            FROM posts p
            WHERE p.id != ?1 AND p.published = 1
            GROUP BY p.id
            HAVING relevance_score > 0
            ORDER BY relevance_score DESC, p.created_at DESC
            LIMIT ?3
        "#;

        let rows = sqlx::query(query)
            .bind(post_id)
            .bind(category)
            .bind(limit as i64)
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

        // If we don't have enough related posts, fill with recent posts
        if results.len() < limit {
            let additional_query = r#"
                SELECT id, title, slug, content, excerpt, author,
                       created_at, updated_at, published, category, storage_id, content_hash
                FROM posts
                WHERE id != ?1 AND published = 1
                  AND id NOT IN (
                      SELECT p2.id
                      FROM posts p2
                      WHERE p2.id != ?1 AND p2.published = 1
                      AND (
                          EXISTS (
                              SELECT 1 FROM post_tags pt1
                              JOIN post_tags pt2 ON pt1.tag_id = pt2.tag_id
                              WHERE pt1.post_id = ?1 AND pt2.post_id = p2.id
                          )
                          OR (p2.category = ?2 AND p2.category IS NOT NULL)
                      )
                  )
                ORDER BY created_at DESC
                LIMIT ?3
            "#;

            let additional_rows = sqlx::query(additional_query)
                .bind(post_id)
                .bind(category)
                .bind((limit - results.len()) as i64)
                .fetch_all(&self.pool)
                .await?;

            for row in additional_rows {
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
                post.tags = self.load_tags_for_post(&post.id).await?;
                results.push((storage_id, post));
            }
        }

        Ok(results)
    }
}
