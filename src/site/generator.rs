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

        // Add custom filters
        tera.register_filter("url_safe_tag", crate::site::filters::url_safe_tag);
        tera.register_filter("highlight_search", crate::site::filters::highlight_search);
        tera.register_filter("escape", crate::site::filters::escape_html);

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

        // Generate SEO files
        self.generate_sitemap(&posts).await?;
        self.generate_robots_txt()?;
        self.generate_webmanifest()?;
        self.create_default_images()?;

        // Optimize images
        self.optimize_all_images().await?;

        // Minify HTML/CSS/JS
        self.minify_resources().await?;

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

                    // Add reading time
                    let reading_time = crate::utils::calculate_reading_time(&post.content, false);
                    post_context["reading_time"] =
                        serde_json::Value::String(reading_time.to_string());
                    post_context["reading_minutes"] =
                        serde_json::Value::Number(reading_time.minutes.into());

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
        // Get related posts
        let related_posts = self
            .blog_manager
            .get_related_posts(&post.id, &post.tags, post.category.as_deref(), 5)
            .await?;

        let related_posts_data: Vec<_> = related_posts
            .iter()
            .map(|(_, related_post)| {
                let mut post_context = serde_json::to_value(related_post).unwrap();
                let base_path = self.config.base_path.as_deref().unwrap_or("");
                post_context["url"] = serde_json::Value::String(format!(
                    "{}/posts/{}.html",
                    base_path,
                    sanitize_slug(&related_post.slug)
                ));

                // Add reading time
                let reading_time =
                    crate::utils::calculate_reading_time(&related_post.content, false);
                post_context["reading_time"] = serde_json::Value::String(reading_time.to_string());

                post_context
            })
            .collect();

        let mut context = Context::new();
        context.insert("site", &self.config);
        context.insert("page_title", &post.title);
        context.insert("post", post);

        // Generate HTML content with heading IDs
        let content_html = markdown_to_html(&post.content);
        context.insert("content_html", &content_html);
        context.insert("storage_id", storage_id);
        context.insert("related_posts", &related_posts_data);

        // Generate table of contents
        let toc = crate::utils::generate_toc(&post.content);
        let toc_html = crate::utils::toc::generate_toc_html(&toc);
        context.insert("toc", &toc);
        context.insert("toc_html", &toc_html);
        context.insert("has_toc", &!toc.is_empty());

        // Add reading time
        let reading_time = crate::utils::calculate_reading_time(&post.content, false);
        context.insert("reading_time", &reading_time.to_string());
        context.insert("reading_minutes", &reading_time.minutes);

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

        // Create images directory
        let images_dir = self.output_dir.join("images");
        fs::create_dir_all(&images_dir)?;

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

                    // Add reading time
                    let reading_time = crate::utils::calculate_reading_time(&post.content, false);
                    post_context["reading_time"] =
                        serde_json::Value::String(reading_time.to_string());
                    post_context["reading_minutes"] =
                        serde_json::Value::Number(reading_time.minutes.into());

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

    // Post-process HTML to add IDs to headings
    add_heading_ids_to_html(&html_output)
}

/// Add IDs to headings in HTML for TOC navigation
fn add_heading_ids_to_html(html: &str) -> String {
    use regex::Regex;

    let heading_re = Regex::new(r"<h([1-6])>(.*?)</h[1-6]>").unwrap();

    heading_re
        .replace_all(html, |caps: &regex::Captures| {
            let level = &caps[1];
            let content = &caps[2];

            // Extract text content without HTML tags for ID generation
            let text_only = Regex::new(r"<[^>]+>").unwrap().replace_all(content, "");
            let id = crate::utils::toc::generate_heading_id(&text_only);

            format!(r#"<h{} id="{}">{}</h{}>"#, level, id, content, level)
        })
        .to_string()
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

    async fn generate_sitemap(&self, posts: &[(String, BlogPost)]) -> Result<()> {
        use chrono::SecondsFormat;

        let mut sitemap = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        sitemap.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

        // Add homepage
        sitemap.push_str(&format!(
            "  <url>\n    <loc>{}/</loc>\n    <changefreq>daily</changefreq>\n    <priority>1.0</priority>\n  </url>\n",
            self.config.base_url
        ));

        // Add archive page
        sitemap.push_str(&format!(
            "  <url>\n    <loc>{}/archive/</loc>\n    <changefreq>weekly</changefreq>\n    <priority>0.8</priority>\n  </url>\n",
            self.config.base_url
        ));

        // Add tags page
        sitemap.push_str(&format!(
            "  <url>\n    <loc>{}/tags/</loc>\n    <changefreq>weekly</changefreq>\n    <priority>0.7</priority>\n  </url>\n",
            self.config.base_url
        ));

        // Add docs page if exists
        sitemap.push_str(&format!(
            "  <url>\n    <loc>{}/docs/</loc>\n    <changefreq>weekly</changefreq>\n    <priority>0.8</priority>\n  </url>\n",
            self.config.base_url
        ));

        // Add all posts
        for (_, post) in posts {
            let safe_slug = sanitize_slug(&post.slug);
            sitemap.push_str(&format!(
                "  <url>\n    <loc>{}/posts/{}.html</loc>\n    <lastmod>{}</lastmod>\n    <changefreq>monthly</changefreq>\n    <priority>0.6</priority>\n  </url>\n",
                self.config.base_url,
                safe_slug,
                post.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true)
            ));
        }

        // Add tag pages
        let all_tags = self.blog_manager.get_all_tags().await?;
        for (tag_name, _) in all_tags {
            let tag_url_safe = sanitize_tag_for_url(&tag_name);
            sitemap.push_str(&format!(
                "  <url>\n    <loc>{}/tags/{}/</loc>\n    <changefreq>weekly</changefreq>\n    <priority>0.5</priority>\n  </url>\n",
                self.config.base_url,
                tag_url_safe
            ));
        }

        // Add pagination pages
        let total_pages = posts.len().div_ceil(self.config.posts_per_page);
        for page in 2..=total_pages {
            sitemap.push_str(&format!(
                "  <url>\n    <loc>{}/page/{}/</loc>\n    <changefreq>weekly</changefreq>\n    <priority>0.4</priority>\n  </url>\n",
                self.config.base_url,
                page
            ));
        }

        sitemap.push_str("</urlset>");

        let output_path = self.output_dir.join("sitemap.xml");
        fs::write(output_path, sitemap)?;

        println!("📄 Generated sitemap.xml");
        Ok(())
    }

    fn generate_robots_txt(&self) -> Result<()> {
        let mut robots = String::from("# Robots.txt for KPGB Blog\n\n");

        // Allow all crawlers by default
        robots.push_str("User-agent: *\n");
        robots.push_str("Allow: /\n");
        robots.push_str("Crawl-delay: 1\n\n");

        // Block specific paths if needed
        robots.push_str("# Block draft pages\n");
        robots.push_str("Disallow: /drafts/\n");
        robots.push_str("Disallow: /admin/\n\n");

        // Add sitemap location
        robots.push_str(&format!("Sitemap: {}/sitemap.xml\n", self.config.base_url));

        // Add host directive
        robots.push_str(&format!("\nHost: {}\n", self.config.base_url));

        let output_path = self.output_dir.join("robots.txt");
        fs::write(output_path, robots)?;

        println!("🤖 Generated robots.txt");
        Ok(())
    }

    fn generate_webmanifest(&self) -> Result<()> {
        let manifest = serde_json::json!({
            "name": self.config.title,
            "short_name": self.config.title,
            "description": self.config.description,
            "start_url": "/",
            "display": "standalone",
            "background_color": "#ffffff",
            "theme_color": "#3498db",
            "icons": [
                {
                    "src": "/favicon-16x16.png",
                    "sizes": "16x16",
                    "type": "image/png"
                },
                {
                    "src": "/favicon-32x32.png",
                    "sizes": "32x32",
                    "type": "image/png"
                },
                {
                    "src": "/android-chrome-192x192.png",
                    "sizes": "192x192",
                    "type": "image/png"
                },
                {
                    "src": "/android-chrome-512x512.png",
                    "sizes": "512x512",
                    "type": "image/png"
                }
            ]
        });

        let output_path = self.output_dir.join("site.webmanifest");
        fs::write(output_path, serde_json::to_string_pretty(&manifest)?)?;

        println!("📱 Generated site.webmanifest");
        Ok(())
    }

    fn create_default_images(&self) -> Result<()> {
        // Create images directory
        let images_dir = self.output_dir.join("images");
        fs::create_dir_all(&images_dir)?;

        // Generate simple SVG placeholders for now
        // In production, you would use actual images

        // Open Graph default image (1200x630)
        let og_svg = r##"<svg width="1200" height="630" xmlns="http://www.w3.org/2000/svg">
            <rect width="1200" height="630" fill="#3498db"/>
            <text x="600" y="315" font-family="Arial, sans-serif" font-size="72" fill="white" text-anchor="middle" dominant-baseline="middle">KPGB Blog</text>
            <text x="600" y="400" font-family="Arial, sans-serif" font-size="36" fill="white" text-anchor="middle" opacity="0.8">Decentralized IPFS Blog</text>
        </svg>"##;
        fs::write(images_dir.join("og-default.svg"), og_svg)?;

        // Twitter card image (1200x600)
        let twitter_svg = r##"<svg width="1200" height="600" xmlns="http://www.w3.org/2000/svg">
            <rect width="1200" height="600" fill="#2c3e50"/>
            <text x="600" y="300" font-family="Arial, sans-serif" font-size="72" fill="white" text-anchor="middle" dominant-baseline="middle">KPGB Blog</text>
            <text x="600" y="380" font-family="Arial, sans-serif" font-size="36" fill="white" text-anchor="middle" opacity="0.8">Powered by IPFS</text>
        </svg>"##;
        fs::write(images_dir.join("twitter-card.svg"), twitter_svg)?;

        // Logo placeholder
        let logo_svg = r##"<svg width="512" height="512" xmlns="http://www.w3.org/2000/svg">
            <rect width="512" height="512" fill="#3498db"/>
            <text x="256" y="256" font-family="Arial, sans-serif" font-size="200" fill="white" text-anchor="middle" dominant-baseline="middle">K</text>
        </svg>"##;
        fs::write(images_dir.join("logo.svg"), logo_svg)?;

        // Create favicon placeholders
        let favicon_svg = r##"<svg width="32" height="32" xmlns="http://www.w3.org/2000/svg">
            <rect width="32" height="32" fill="#3498db" rx="4"/>
            <text x="16" y="16" font-family="Arial, sans-serif" font-size="20" fill="white" text-anchor="middle" dominant-baseline="middle">K</text>
        </svg>"##;

        fs::write(self.output_dir.join("favicon.svg"), favicon_svg)?;

        // Note: In production, you would convert these SVGs to PNGs
        // For now, we'll use the SVGs as placeholders

        println!("🖼️  Created default images");
        Ok(())
    }

    async fn optimize_all_images(&self) -> Result<()> {
        println!("🖼️  Optimizing images...");

        // Create image optimization config
        let config = crate::utils::ImageOptimizationConfig::default();

        // Optimize images in the output directory
        let stats = crate::utils::optimize_images_in_directory(&self.output_dir, &config).await?;

        println!("✨ {}", stats.summary());
        Ok(())
    }

    async fn minify_resources(&self) -> Result<()> {
        println!("📦 Minifying HTML/CSS/JS...");

        // Create minification config
        let config = crate::utils::MinifyConfig::default();

        // Minify all resources in the output directory
        let stats = crate::utils::minify_directory(&self.output_dir, &config).await?;

        println!("🗜️  {}", stats.summary());
        Ok(())
    }
}
