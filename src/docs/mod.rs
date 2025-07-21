use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSection {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source_url: Option<String>,
    pub is_translated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocCategory {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub order: i32,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    pub project_name: String,
    pub project_url: String,
    pub version: String,
    pub categories: Vec<DocCategory>,
    pub sections: HashMap<String, Vec<DocSection>>,
}

impl Documentation {
    pub fn new(project_name: String, project_url: String, version: String) -> Self {
        Self {
            project_name,
            project_url,
            version,
            categories: Vec::new(),
            sections: HashMap::new(),
        }
    }

    pub fn add_category(&mut self, category: DocCategory) {
        self.categories.push(category);
    }

    pub fn add_section(&mut self, category_id: String, section: DocSection) {
        self.sections.entry(category_id).or_default().push(section);
    }

    pub fn get_sections_by_category(&self, category_id: &str) -> Vec<&DocSection> {
        self.sections
            .get(category_id)
            .map(|sections| sections.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_section_by_slug(&self, slug: &str) -> Option<&DocSection> {
        for sections in self.sections.values() {
            if let Some(section) = sections.iter().find(|s| s.slug == slug) {
                return Some(section);
            }
        }
        None
    }
}

pub fn generate_doc_slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
