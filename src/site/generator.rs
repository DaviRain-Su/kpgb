use anyhow::Result;
use chrono::Datelike;
use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use super::SiteConfig;
use crate::blog::BlogManager;
use crate::models::BlogPost;

pub struct SiteGenerator {
    blog_manager: BlogManager,
    config: SiteConfig,
    tera: Tera,
    output_dir: PathBuf,
}

impl SiteGenerator {
    pub async fn new(
        blog_manager: BlogManager,
        config: SiteConfig,
        output_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        let output_dir = output_dir.as_ref().to_path_buf();

        // Create output directory
        fs::create_dir_all(&output_dir)?;

        // Load templates
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("base.html", include_str!("../../templates/base.html")),
            ("index.html", include_str!("../../templates/index.html")),
            ("post.html", include_str!("../../templates/post.html")),
            ("archive.html", include_str!("../../templates/archive.html")),
            ("tags.html", include_str!("../../templates/tags.html")),
            (
                "tag_posts.html",
                include_str!("../../templates/tag_posts.html"),
            ),
            ("docs.html", include_str!("../../templates/docs.html")),
        ])?;

        // Add custom filter for URL-safe tags
        tera.register_filter(
            "url_safe_tag",
            |value: &tera::Value, _: &std::collections::HashMap<String, tera::Value>| match value
                .as_str()
            {
                Some(tag) => Ok(tera::Value::String(sanitize_tag_for_url(tag))),
                None => Err(tera::Error::msg("url_safe_tag filter expects a string")),
            },
        );

        Ok(Self {
            blog_manager,
            config,
            tera,
            output_dir,
        })
    }

    pub async fn generate(&self) -> Result<()> {
        println!("🚀 Generating static site...");

        // Copy static assets
        self.copy_static_assets()?;

        // Get all published posts
        let mut posts = self.blog_manager.list_posts(true).await?;

        // Deduplicate posts by slug, keeping the one with more tags or newer
        let mut seen_slugs: std::collections::HashMap<String, (String, BlogPost)> =
            std::collections::HashMap::new();
        for (id, post) in posts.iter() {
            let entry = seen_slugs
                .entry(post.slug.clone())
                .or_insert_with(|| (id.clone(), post.clone()));
            // Replace if current post has more tags or is newer
            if post.tags.len() > entry.1.tags.len()
                || (post.tags.len() == entry.1.tags.len() && post.created_at > entry.1.created_at)
            {
                *entry = (id.clone(), post.clone());
            }
        }
        // Convert back to vector
        posts = seen_slugs.into_iter().map(|(_, v)| v).collect();
        // Sort by created_at desc to maintain order
        posts.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        // Generate index page
        self.generate_index(&posts).await?;

        // Generate individual post pages
        for (storage_id, post) in &posts {
            self.generate_post_page(storage_id, post).await?;
        }

        // Generate archive page
        self.generate_archive(&posts).await?;

        // Generate RSS feed
        if self.config.enable_rss {
            self.generate_rss(&posts).await?;
        }

        // Generate tag pages
        self.generate_tag_pages(&posts).await?;

        // Generate docs page
        self.generate_docs_page().await?;

        println!(
            "✅ Site generated successfully in: {}",
            self.output_dir.display()
        );
        Ok(())
    }

    async fn generate_index(&self, posts: &[(String, BlogPost)]) -> Result<()> {
        let posts_per_page = self.config.posts_per_page;
        let total_pages = posts.len().div_ceil(posts_per_page);

        // Generate each page
        for page in 1..=total_pages.max(1) {
            let mut context = Context::new();
            context.insert("site", &self.config);
            let page_title = if page == 1 {
                "Home".to_string()
            } else {
                format!("Page {}", page)
            };
            context.insert("page_title", &page_title);

            // Calculate post range for this page
            let start = (page - 1) * posts_per_page;
            let end = (start + posts_per_page).min(posts.len());

            let page_posts: Vec<_> = posts[start..end]
                .iter()
                .map(|(id, post)| {
                    let mut post_context = serde_json::to_value(post).unwrap();
                    let base_path = self.config.base_path.as_deref().unwrap_or("");
                    post_context["url"] = serde_json::Value::String(format!(
                        "{}/posts/{}.html",
                        base_path,
                        sanitize_slug(&post.slug)
                    ));
                    post_context["content_html"] =
                        serde_json::Value::String(markdown_to_html(&post.content));
                    post_context["storage_id"] = serde_json::Value::String(id.clone());

                    // Generate excerpt HTML if not provided
                    if post.excerpt.is_none() {
                        let excerpt_text =
                            crate::utils::generate_formatted_excerpt(&post.content, 300);
                        post_context["excerpt_html"] =
                            serde_json::Value::String(markdown_to_html(&excerpt_text));
                    } else {
                        post_context["excerpt_html"] = serde_json::Value::String(markdown_to_html(
                            post.excerpt.as_ref().unwrap(),
                        ));
                    }

                    post_context
                })
                .collect();

            context.insert("posts", &page_posts);

            // Pagination context
            context.insert("current_page", &page);
            context.insert("total_pages", &total_pages);
            context.insert("has_prev", &(page > 1));
            context.insert("has_next", &(page < total_pages));

            let rendered = self.tera.render("index.html", &context)?;

            // Determine output path
            let output_path = if page == 1 {
                self.output_dir.join("index.html")
            } else {
                let page_dir = self.output_dir.join("page").join(page.to_string());
                fs::create_dir_all(&page_dir)?;
                page_dir.join("index.html")
            };

            fs::write(output_path, rendered)?;
        }

        Ok(())
    }

    async fn generate_post_page(&self, storage_id: &str, post: &BlogPost) -> Result<()> {
        let mut context = Context::new();
        context.insert("site", &self.config);
        context.insert("page_title", &post.title);
        context.insert("post", post);
        context.insert("content_html", &markdown_to_html(&post.content));
        context.insert("storage_id", storage_id);

        // Add IPFS link if it's an IPFS CID
        if storage_id.starts_with("Qm") {
            context.insert(
                "ipfs_link",
                &format!("{}{}", self.config.ipfs_gateway, storage_id),
            );
        }

        let rendered = self.tera.render("post.html", &context)?;

        // Create posts directory
        let posts_dir = self.output_dir.join("posts");
        fs::create_dir_all(&posts_dir)?;

        // Sanitize slug for file name (remove non-ASCII characters)
        let safe_slug = sanitize_slug(&post.slug);
        let output_path = posts_dir.join(format!("{}.html", safe_slug));
        fs::write(output_path, rendered)?;

        Ok(())
    }

    async fn generate_archive(&self, posts: &[(String, BlogPost)]) -> Result<()> {
        let mut context = Context::new();
        context.insert("site", &self.config);
        context.insert("page_title", "Archive");

        // Group posts by year
        let mut posts_by_year: std::collections::HashMap<i32, Vec<_>> =
            std::collections::HashMap::new();

        for (id, post) in posts {
            let year = post.created_at.year();
            let mut post_context = serde_json::to_value(post).unwrap();
            let base_path = self.config.base_path.as_deref().unwrap_or("");
            post_context["url"] = serde_json::Value::String(format!(
                "{}/posts/{}.html",
                base_path,
                sanitize_slug(&post.slug)
            ));
            post_context["storage_id"] = serde_json::Value::String(id.clone());

            posts_by_year
                .entry(year)
                .or_insert_with(Vec::new)
                .push(post_context);
        }

        let mut years: Vec<_> = posts_by_year.into_iter().collect();
        years.sort_by(|a, b| b.0.cmp(&a.0)); // Sort years descending

        context.insert("years", &years);

        let rendered = self.tera.render("archive.html", &context)?;

        // Create archive directory for clean URLs
        let archive_dir = self.output_dir.join("archive");
        fs::create_dir_all(&archive_dir)?;
        let output_path = archive_dir.join("index.html");
        fs::write(&output_path, &rendered)?;

        // Also create archive.html for backward compatibility
        let archive_html_path = self.output_dir.join("archive.html");
        fs::write(archive_html_path, rendered)?;

        Ok(())
    }

    async fn generate_rss(&self, posts: &[(String, BlogPost)]) -> Result<()> {
        use rss::{ChannelBuilder, ItemBuilder};

        let mut items = Vec::new();

        for (storage_id, post) in posts.iter().take(20) {
            // Latest 20 posts
            let link = format!("{}/posts/{}.html", self.config.base_url, post.slug);
            let content_html = markdown_to_html(&post.content);

            let item = ItemBuilder::default()
                .title(Some(post.title.clone()))
                .link(Some(link))
                .description(Some(content_html))
                .author(Some(post.author.clone()))
                .pub_date(Some(post.created_at.to_rfc2822()))
                .guid(Some(rss::Guid {
                    value: storage_id.clone(),
                    permalink: false,
                }))
                .build();

            items.push(item);
        }

        let channel = ChannelBuilder::default()
            .title(&self.config.title)
            .link(&self.config.base_url)
            .description(&self.config.description)
            .items(items)
            .build();

        let output_path = self.output_dir.join("feed.xml");
        fs::write(output_path, channel.to_string())?;

        Ok(())
    }

    fn copy_static_assets(&self) -> Result<()> {
        // Create CSS
        let css_dir = self.output_dir.join("css");
        fs::create_dir_all(&css_dir)?;

        // Load theme-specific CSS
        let css_content = match self.config.theme.as_str() {
            "hacker" => include_str!("../../templates/themes/hacker.css"),
            "minimal" => include_str!("../../templates/themes/minimal.css"),
            "dark" => include_str!("../../templates/themes/dark.css"),
            "cyberpunk" => include_str!("../../templates/themes/cyberpunk.css"),
            _ => include_str!("../../templates/style.css"), // default theme
        };

        fs::write(css_dir.join("style.css"), css_content)?;

        Ok(())
    }

    async fn generate_tag_pages(&self, posts: &[(String, BlogPost)]) -> Result<()> {
        // Get all tags
        let all_tags = self.blog_manager.get_all_tags().await?;

        // Generate main tags page
        let mut context = Context::new();
        context.insert("site", &self.config);
        context.insert("title", "Tags");

        let tag_data: Vec<_> = all_tags
            .iter()
            .map(|(name, count)| {
                let mut tag_map = serde_json::Map::new();
                tag_map.insert("name".to_string(), serde_json::Value::String(name.clone()));
                tag_map.insert(
                    "count".to_string(),
                    serde_json::Value::Number((*count).into()),
                );
                let base_path = self.config.base_path.as_deref().unwrap_or("");
                tag_map.insert(
                    "url".to_string(),
                    serde_json::Value::String(format!("{}/tags/{}", base_path, name)),
                );
                serde_json::Value::Object(tag_map)
            })
            .collect();

        context.insert("tags", &tag_data);

        // Create tags directory with index.html for clean URLs
        let tags_dir = self.output_dir.join("tags");
        fs::create_dir_all(&tags_dir)?;

        let rendered = self.tera.render("tags.html", &context)?;
        let output_path = tags_dir.join("index.html");
        fs::write(output_path, rendered)?;

        // Generate individual tag pages
        for (tag_name, _count) in all_tags {
            println!("Generating tag page for: {}", tag_name);
            self.generate_tag_page(&tag_name, posts).await?;
        }

        Ok(())
    }

    async fn generate_tag_page(&self, tag: &str, all_posts: &[(String, BlogPost)]) -> Result<()> {
        // Filter posts by tag
        let tag_posts: Vec<_> = all_posts
            .iter()
            .filter(|(_, post)| post.tags.contains(&tag.to_string()))
            .collect();

        let posts_per_page = self.config.posts_per_page;
        let total_pages = tag_posts.len().div_ceil(posts_per_page);

        // Generate each page for this tag
        for page in 1..=total_pages.max(1) {
            // Calculate post range for this page
            let start = (page - 1) * posts_per_page;
            let end = (start + posts_per_page).min(tag_posts.len());

            let posts_data: Vec<_> = tag_posts[start..end]
                .iter()
                .map(|(id, post)| {
                    let mut post_context = serde_json::to_value(post).unwrap();
                    let base_path = self.config.base_path.as_deref().unwrap_or("");
                    post_context["url"] = serde_json::Value::String(format!(
                        "{}/posts/{}.html",
                        base_path,
                        sanitize_slug(&post.slug)
                    ));
                    post_context["content_html"] =
                        serde_json::Value::String(markdown_to_html(&post.content));
                    post_context["storage_id"] = serde_json::Value::String(id.clone());

                    // Generate excerpt HTML if not provided
                    if post.excerpt.is_none() {
                        let excerpt_text =
                            crate::utils::generate_formatted_excerpt(&post.content, 300);
                        post_context["excerpt_html"] =
                            serde_json::Value::String(markdown_to_html(&excerpt_text));
                    } else {
                        post_context["excerpt_html"] = serde_json::Value::String(markdown_to_html(
                            post.excerpt.as_ref().unwrap(),
                        ));
                    }

                    post_context
                })
                .collect();

            let mut context = Context::new();
            context.insert("site", &self.config);
            context.insert("posts", &posts_data);
            context.insert("tag", tag);
            context.insert("title", &format!("Posts tagged '{}'", tag));

            // Pagination context
            context.insert("current_page", &page);
            context.insert("total_pages", &total_pages);
            context.insert("has_prev", &(page > 1));
            context.insert("has_next", &(page < total_pages));

            // Create output directory
            let tag_url_safe = sanitize_tag_for_url(tag);
            let output_path = if page == 1 {
                let tag_dir = self.output_dir.join("tags").join(&tag_url_safe);
                fs::create_dir_all(&tag_dir)?;
                tag_dir.join("index.html")
            } else {
                let page_dir = self
                    .output_dir
                    .join("tags")
                    .join(&tag_url_safe)
                    .join("page")
                    .join(page.to_string());
                fs::create_dir_all(&page_dir)?;
                page_dir.join("index.html")
            };

            let rendered = self.tera.render("tag_posts.html", &context)?;
            fs::write(output_path, rendered)?;
        }

        Ok(())
    }
}

fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

/// Sanitize slug to only contain ASCII characters for file names
fn sanitize_slug(slug: &str) -> String {
    let sanitized = slug
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // If empty after sanitization, use a default
    if sanitized.is_empty() {
        format!("post-{}", chrono::Utc::now().timestamp())
    } else {
        sanitized
    }
}

fn sanitize_tag_for_url(tag: &str) -> String {
    // For tags, we'll use a simple approach: convert to lowercase and replace spaces with hyphens
    // Chinese characters and other non-ASCII will be preserved
    tag.to_lowercase().replace(' ', "-")
}

impl SiteGenerator {
    async fn generate_docs_page(&self) -> Result<()> {
        let mut context = Context::new();
        context.insert("site", &self.config);
        context.insert("page_title", "技术文档中心");

        let html = self.tera.render("docs.html", &context)?;

        let docs_path = self.output_dir.join("docs");
        fs::create_dir_all(&docs_path)?;
        fs::write(docs_path.join("index.html"), html)?;

        // Also create docs.html for compatibility
        let html = self.tera.render("docs.html", &context)?;
        fs::write(self.output_dir.join("docs.html"), html)?;

        Ok(())
    }
}
