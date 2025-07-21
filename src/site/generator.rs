use anyhow::Result;
use pulldown_cmark::{Parser, Options, html};
use tera::{Tera, Context};
use std::path::{Path, PathBuf};
use std::fs;
use chrono::Datelike;

use crate::blog::BlogManager;
use crate::models::BlogPost;
use super::SiteConfig;

pub struct SiteGenerator {
    blog_manager: BlogManager,
    config: SiteConfig,
    tera: Tera,
    output_dir: PathBuf,
}

impl SiteGenerator {
    pub async fn new(blog_manager: BlogManager, config: SiteConfig, output_dir: impl AsRef<Path>) -> Result<Self> {
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
        ])?;
        
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
        let posts = self.blog_manager.list_posts(true).await?;
        
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
        
        println!("✅ Site generated successfully in: {}", self.output_dir.display());
        Ok(())
    }
    
    async fn generate_index(&self, posts: &[(String, BlogPost)]) -> Result<()> {
        let mut context = Context::new();
        context.insert("site", &self.config);
        context.insert("page_title", "Home");
        
        // Take the latest posts for the index
        let latest_posts: Vec<_> = posts.iter()
            .take(self.config.posts_per_page)
            .map(|(id, post)| {
                let mut post_context = serde_json::to_value(post).unwrap();
                post_context["url"] = serde_json::Value::String(format!("posts/{}.html", post.slug));
                post_context["content_html"] = serde_json::Value::String(markdown_to_html(&post.content));
                post_context["storage_id"] = serde_json::Value::String(id.clone());
                post_context
            })
            .collect();
        
        context.insert("posts", &latest_posts);
        
        let rendered = self.tera.render("index.html", &context)?;
        let output_path = self.output_dir.join("index.html");
        fs::write(output_path, rendered)?;
        
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
            context.insert("ipfs_link", &format!("{}{}", self.config.ipfs_gateway, storage_id));
        }
        
        let rendered = self.tera.render("post.html", &context)?;
        
        // Create posts directory
        let posts_dir = self.output_dir.join("posts");
        fs::create_dir_all(&posts_dir)?;
        
        let output_path = posts_dir.join(format!("{}.html", post.slug));
        fs::write(output_path, rendered)?;
        
        Ok(())
    }
    
    async fn generate_archive(&self, posts: &[(String, BlogPost)]) -> Result<()> {
        let mut context = Context::new();
        context.insert("site", &self.config);
        context.insert("page_title", "Archive");
        
        // Group posts by year
        let mut posts_by_year: std::collections::HashMap<i32, Vec<_>> = std::collections::HashMap::new();
        
        for (id, post) in posts {
            let year = post.created_at.year();
            let mut post_context = serde_json::to_value(post).unwrap();
            post_context["url"] = serde_json::Value::String(format!("posts/{}.html", post.slug));
            post_context["storage_id"] = serde_json::Value::String(id.clone());
            
            posts_by_year.entry(year).or_insert_with(Vec::new).push(post_context);
        }
        
        let mut years: Vec<_> = posts_by_year.into_iter().collect();
        years.sort_by(|a, b| b.0.cmp(&a.0)); // Sort years descending
        
        context.insert("years", &years);
        
        let rendered = self.tera.render("archive.html", &context)?;
        let output_path = self.output_dir.join("archive.html");
        fs::write(output_path, rendered)?;
        
        Ok(())
    }
    
    async fn generate_rss(&self, posts: &[(String, BlogPost)]) -> Result<()> {
        use rss::{ChannelBuilder, ItemBuilder};
        
        let mut items = Vec::new();
        
        for (storage_id, post) in posts.iter().take(20) { // Latest 20 posts
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
        
        let css_content = include_str!("../../templates/style.css");
        fs::write(css_dir.join("style.css"), css_content)?;
        
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