use anyhow::Result;
use sqlx::{query, query_as, SqlitePool};
use super::{DocSection, DocCategory};

pub struct DocsDatabase {
    pool: SqlitePool,
}

impl DocsDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init_tables(&self) -> Result<()> {
        // 创建文档分类表
        query(
            r#"
            CREATE TABLE IF NOT EXISTS doc_categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                description TEXT,
                order_index INTEGER DEFAULT 0,
                icon TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建文档章节表
        query(
            r#"
            CREATE TABLE IF NOT EXISTS doc_sections (
                id TEXT PRIMARY KEY,
                category_id TEXT NOT NULL,
                title TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                content TEXT NOT NULL,
                parent_id TEXT,
                order_index INTEGER DEFAULT 0,
                source_url TEXT,
                is_translated BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (category_id) REFERENCES doc_categories(id),
                FOREIGN KEY (parent_id) REFERENCES doc_sections(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        query("CREATE INDEX IF NOT EXISTS idx_doc_sections_category ON doc_sections(category_id)")
            .execute(&self.pool)
            .await?;

        query("CREATE INDEX IF NOT EXISTS idx_doc_sections_slug ON doc_sections(slug)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn create_category(&self, category: &DocCategory) -> Result<()> {
        query(
            r#"
            INSERT INTO doc_categories (id, name, slug, description, order_index, icon)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&category.id)
        .bind(&category.name)
        .bind(&category.slug)
        .bind(&category.description)
        .bind(category.order)
        .bind(&category.icon)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_section(&self, section: &DocSection) -> Result<()> {
        query(
            r#"
            INSERT INTO doc_sections (
                id, category_id, title, slug, content, parent_id, 
                order_index, source_url, is_translated, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(&section.id)
        .bind(&section.parent_id.as_ref().unwrap_or(&section.id)) // category_id
        .bind(&section.title)
        .bind(&section.slug)
        .bind(&section.content)
        .bind(&section.parent_id)
        .bind(section.order)
        .bind(&section.source_url)
        .bind(section.is_translated)
        .bind(section.created_at)
        .bind(section.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all_categories(&self) -> Result<Vec<DocCategory>> {
        let categories = query_as!(
            DocCategory,
            r#"
            SELECT 
                id, name, slug, description, 
                order_index as "order: i32", 
                icon
            FROM doc_categories
            ORDER BY order_index, name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    pub async fn get_sections_by_category(&self, category_id: &str) -> Result<Vec<DocSection>> {
        let sections = sqlx::query!(
            r#"
            SELECT 
                id, title, slug, content, parent_id, 
                order_index, source_url, is_translated,
                created_at, updated_at
            FROM doc_sections
            WHERE category_id = ?1 OR parent_id = ?1
            ORDER BY order_index, title
            "#,
            category_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| DocSection {
            id: row.id,
            title: row.title,
            slug: row.slug,
            content: row.content,
            parent_id: row.parent_id,
            order: row.order_index,
            created_at: row.created_at,
            updated_at: row.updated_at,
            source_url: row.source_url,
            is_translated: row.is_translated,
        })
        .collect();

        Ok(sections)
    }

    pub async fn get_section_by_slug(&self, slug: &str) -> Result<Option<DocSection>> {
        let section = sqlx::query!(
            r#"
            SELECT 
                id, title, slug, content, parent_id, 
                order_index, source_url, is_translated,
                created_at, updated_at
            FROM doc_sections
            WHERE slug = ?1
            "#,
            slug
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| DocSection {
            id: row.id,
            title: row.title,
            slug: row.slug,
            content: row.content,
            parent_id: row.parent_id,
            order: row.order_index,
            created_at: row.created_at,
            updated_at: row.updated_at,
            source_url: row.source_url,
            is_translated: row.is_translated,
        });

        Ok(section)
    }

    pub async fn update_section_content(&self, id: &str, content: &str) -> Result<()> {
        query(
            r#"
            UPDATE doc_sections 
            SET content = ?1, is_translated = TRUE, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?2
            "#,
        )
        .bind(content)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}