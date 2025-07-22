use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub author: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_tags")]
    pub tags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub published: Option<bool>,
}

// Custom deserializer for tags that handles both array and comma-separated string
fn deserialize_tags<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct TagsVisitor;

    impl<'de> Visitor<'de> for TagsVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence or comma-separated string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Vec<String>, E>
        where
            E: de::Error,
        {
            Ok(value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Vec<String>, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut tags = Vec::new();
            while let Some(tag) = seq.next_element::<String>()? {
                tags.push(tag);
            }
            Ok(tags)
        }
    }

    deserializer.deserialize_any(TagsVisitor)
}

pub fn parse_frontmatter(content: &str) -> Result<(Option<FrontMatter>, String)> {
    let content = content.trim_start();

    // Check if content starts with frontmatter delimiter
    if !content.starts_with("---") {
        return Ok((None, content.to_string()));
    }

    // Find the end of frontmatter
    let mut lines = content.lines();
    lines.next(); // Skip the first ---

    let mut frontmatter_lines = Vec::new();
    let mut content_lines = Vec::new();
    let mut in_frontmatter = true;

    for line in lines {
        if in_frontmatter && line.trim() == "---" {
            in_frontmatter = false;
            continue;
        }

        if in_frontmatter {
            frontmatter_lines.push(line);
        } else {
            content_lines.push(line);
        }
    }

    if in_frontmatter {
        // No closing --- found
        return Err(anyhow!("Unclosed frontmatter block"));
    }

    let frontmatter_str = frontmatter_lines.join("\n");
    let content_str = content_lines.join("\n").trim().to_string();

    // Parse YAML frontmatter
    let frontmatter: FrontMatter = serde_yaml::from_str(&frontmatter_str)
        .map_err(|e| anyhow!("Failed to parse frontmatter: {}", e))?;

    Ok((Some(frontmatter), content_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: Test Post
author: Test Author
tags: [rust, blog]
category: Technology
excerpt: This is a test post
---

# Test Content

This is the post content."#;

        let (frontmatter, content) = parse_frontmatter(content).unwrap();
        assert!(frontmatter.is_some());

        let fm = frontmatter.unwrap();
        assert_eq!(fm.title, "Test Post");
        assert_eq!(fm.author, "Test Author");
        assert_eq!(fm.tags, vec!["rust", "blog"]);
        assert_eq!(fm.category, Some("Technology".to_string()));
        assert_eq!(fm.excerpt, Some("This is a test post".to_string()));

        assert!(content.starts_with("# Test Content"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a regular markdown file\n\nNo frontmatter here.";
        let (frontmatter, returned_content) = parse_frontmatter(content).unwrap();
        assert!(frontmatter.is_none());
        assert_eq!(returned_content, content);
    }
}
