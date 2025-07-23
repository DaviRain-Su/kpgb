#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::iter_kv_map)]
#![allow(unused_variables)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::while_let_on_iterator)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::unnecessary_to_owned)]
mod blog;
mod database;
mod docs;
mod frontmatter;
mod models;
mod site;
mod storage;
mod utils;
mod web;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use tracing::info;

use crate::blog::BlogManager;
use crate::models::BlogPost;
use crate::storage::{StorageBackend, StorageManager};

#[derive(Parser)]
#[command(author, version, about = "Decentralized Personal Blog System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new blog post
    New {
        /// Post title
        #[arg(short, long, default_value = "Untitled")]
        title: String,

        /// Author name
        #[arg(short, long, default_value = "Anonymous")]
        author: String,

        /// Content file path (markdown)
        #[arg(short, long)]
        content: Option<String>,
    },

    /// List all posts
    List {
        /// Show only published posts
        #[arg(short, long)]
        published: bool,
    },

    /// Publish a post
    Publish {
        /// Storage ID of the post
        id: String,
    },

    /// Read a specific post
    Read {
        /// Storage ID of the post
        id: String,

        /// Use pager (less/more) for reading
        #[arg(short, long)]
        pager: bool,

        /// Format output (plain, markdown, html)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Terminal width for wrapping
        #[arg(short, long)]
        width: Option<usize>,

        /// Export to file
        #[arg(short, long)]
        export: Option<String>,
    },

    /// Search posts
    Search {
        /// Search query
        query: String,
    },

    /// Test storage backends
    TestStorage {
        /// Backend type: ipfs, github, local
        #[arg(short, long, default_value = "local")]
        backend: String,
    },

    /// Generate static site
    Generate {
        /// Output directory
        #[arg(short, long, default_value = "./public")]
        output: String,

        /// Configuration file to use
        #[arg(short, long, default_value = "site.toml")]
        config: String,
    },

    /// Initialize site configuration
    Init,

    /// Start web server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Configuration file to use
        #[arg(short, long, default_value = "site.toml")]
        config: String,
    },

    /// Edit an existing post
    Edit {
        /// Storage ID of the post to edit
        id: String,

        /// New title (optional)
        #[arg(short, long)]
        title: Option<String>,

        /// New author (optional)
        #[arg(short, long)]
        author: Option<String>,

        /// Content file path for new content (optional)
        #[arg(short, long)]
        content: Option<String>,

        /// New tags (comma-separated, optional)
        #[arg(long)]
        tags: Option<String>,

        /// New category (optional)
        #[arg(long)]
        category: Option<String>,

        /// Open in editor
        #[arg(short, long)]
        editor: bool,
    },

    /// Delete a post
    Delete {
        /// Storage ID of the post to delete
        id: String,

        /// Force delete without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Show blog statistics
    Stats {
        /// Show detailed statistics
        #[arg(short, long)]
        detailed: bool,

        /// Export statistics as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Read a random post
    Random {
        /// Only from published posts
        #[arg(short, long)]
        published: bool,

        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
    },
}

fn wrap_text(text: &str, width: usize) -> String {
    // Simple text wrapping for now
    let mut result = String::new();
    for line in text.lines() {
        if line.starts_with("```") || line.starts_with("    ") || line.len() <= width {
            // Don't wrap code blocks or short lines
            result.push_str(line);
            result.push('\n');
        } else {
            // Wrap long lines
            let words = line.split_whitespace();
            let mut current_line = String::new();
            for word in words {
                if current_line.len() + word.len() + 1 > width && !current_line.is_empty() {
                    result.push_str(&current_line);
                    result.push('\n');
                    current_line.clear();
                }
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            }
            if !current_line.is_empty() {
                result.push_str(&current_line);
                result.push('\n');
            }
        }
    }
    result
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    // Default to IPFS if available, otherwise use local
    let default_backend = if std::env::var("IPFS_API_URL").is_ok() {
        StorageBackend::Ipfs
    } else {
        StorageBackend::Local
    };

    let mut storage_manager = StorageManager::new(default_backend.clone());

    // Always add local storage as fallback
    let local_storage = storage::local::LocalStorage::new("./storage/local")?;
    storage_manager.add_backend(StorageBackend::Local, Box::new(local_storage));

    // Add IPFS if configured
    if let Ok(ipfs_storage) = storage::ipfs::IpfsStorage::from_env() {
        info!("IPFS storage backend configured");
        storage_manager.add_backend(StorageBackend::Ipfs, Box::new(ipfs_storage));
    }

    // Add GitHub if configured
    if let Ok(github_storage) = storage::github::GitHubStorage::from_env() {
        info!("GitHub storage backend configured");
        storage_manager.add_backend(StorageBackend::GitHub, Box::new(github_storage));
    }

    // Database URL
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./kpgb.db".to_string());

    let mut blog_manager = BlogManager::new(storage_manager.clone(), &database_url).await?;

    match cli.command {
        Commands::New {
            title,
            author,
            content,
        } => {
            let (content_text, content_dir) = match &content {
                Some(path) => {
                    let content_path = std::path::Path::new(path);
                    let content_dir = content_path.parent();
                    (std::fs::read_to_string(path)?, content_dir)
                }
                None => {
                    println!("Enter content (press Ctrl+D when done):");
                    use std::io::Read;
                    let mut buffer = String::new();
                    std::io::stdin().read_to_string(&mut buffer)?;
                    (buffer, None)
                }
            };

            // Parse frontmatter if present
            let (frontmatter, mut clean_content) = frontmatter::parse_frontmatter(&content_text)?;

            // Process images in content
            println!("🖼️  Processing images...");
            let (processed_content, image_map) = crate::utils::process_images_in_markdown(
                &clean_content,
                content_dir,
                &storage_manager,
            )
            .await?;

            if !image_map.is_empty() {
                println!("✅ Uploaded {} images to IPFS:", image_map.len());
                for (local_path, ipfs_url) in &image_map {
                    println!("   {} -> {}", local_path, ipfs_url);
                }
            }

            clean_content = processed_content;

            let post = if let Some(fm) = frontmatter {
                // Use frontmatter data, CLI args override frontmatter
                let final_title = if title != "Untitled" {
                    title.clone()
                } else {
                    fm.title
                };
                let final_author = if author != "Anonymous" {
                    author
                } else {
                    fm.author
                };

                let mut post = BlogPost::new(
                    final_title.clone(),
                    clean_content.clone(),
                    final_author.clone(),
                );
                // Use slug from frontmatter if provided
                if let Some(slug) = fm.slug {
                    post.slug = slug;
                }
                post.tags = fm.tags;
                post.category = fm.category;
                post.excerpt = fm
                    .excerpt
                    .or_else(|| Some(crate::utils::generate_excerpt(&clean_content, 50)));

                post
            } else {
                // No frontmatter, use CLI args and full content
                let mut post = BlogPost::new(title.clone(), content_text.clone(), author);
                post.excerpt = Some(crate::utils::generate_excerpt(&content_text, 50));
                post
            };

            let storage_id = blog_manager.create_post(post).await?;

            println!("✅ Post created successfully!");
            println!("Storage ID: {storage_id}");
        }

        Commands::List { published } => {
            let posts = blog_manager.list_posts(published).await?;

            if posts.is_empty() {
                println!("No posts found.");
            } else {
                println!("📝 Blog Posts:");
                println!("{:-<80}", "");
                for (storage_id, post) in posts {
                    let reading_time = crate::utils::calculate_reading_time(&post.content, false);
                    println!("ID: {storage_id}");
                    println!("Title: {}", post.title);
                    println!("Author: {}", post.author);
                    println!("Created: {}", post.created_at.format("%Y-%m-%d %H:%M"));
                    println!("Published: {}", if post.published { "Yes" } else { "No" });
                    println!("Reading: {}", reading_time.to_string());
                    if !post.tags.is_empty() {
                        println!("Tags: {}", post.tags.join(", "));
                    }
                    println!("{:-<80}", "");
                }
            }
        }

        Commands::Publish { id } => {
            blog_manager.publish_post(&id).await?;
            println!("✅ Post published successfully!");
        }

        Commands::Read {
            id,
            pager,
            format,
            width,
            export,
        } => {
            let post = blog_manager.get_post(&id).await?;

            // Calculate reading time
            let is_technical = post.tags.iter().any(|tag| {
                tag.to_lowercase().contains("tech")
                    || tag.to_lowercase().contains("programming")
                    || tag.to_lowercase().contains("rust")
                    || tag.to_lowercase().contains("code")
            });
            let reading_time = crate::utils::calculate_reading_time(&post.content, is_technical);

            // Prepare formatted content
            let mut output = String::new();

            // Header with better formatting
            output.push_str(&format!("{}\n", "═".repeat(80)));
            output.push_str(&format!("📖 {}\n", post.title));
            output.push_str(&format!("{}\n", "─".repeat(80)));
            output.push_str(&format!("✍️  Author: {}\n", post.author));
            output.push_str(&format!(
                "📅 Date: {}\n",
                post.created_at.format("%Y-%m-%d %H:%M")
            ));
            output.push_str(&format!("⏱️  Reading time: {}\n", reading_time.details()));

            if !post.tags.is_empty() {
                output.push_str(&format!("🏷️  Tags: {}\n", post.tags.join(", ")));
            }
            if let Some(cat) = &post.category {
                output.push_str(&format!("📁 Category: {}\n", cat));
            }
            output.push_str(&format!("{}\n\n", "═".repeat(80)));

            // Format content based on format option
            let formatted_content = match format.as_str() {
                "plain" => {
                    // Simple plain text - just strip markdown formatting
                    post.content
                        .lines()
                        .map(|line| {
                            // Remove markdown formatting
                            line.trim_start_matches('#')
                                .trim_start_matches('>')
                                .trim_start_matches('-')
                                .trim_start_matches('*')
                                .trim_start_matches('`')
                                .trim()
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                "html" => {
                    use pulldown_cmark::{html, Options, Parser};
                    let mut options = Options::empty();
                    options.insert(Options::ENABLE_STRIKETHROUGH);
                    options.insert(Options::ENABLE_TABLES);
                    let parser = Parser::new_ext(&post.content, options);
                    let mut html_output = String::new();
                    html::push_html(&mut html_output, parser);
                    html_output
                }
                _ => {
                    // Markdown with syntax highlighting hints
                    let mut formatted = String::new();
                    for line in post.content.lines() {
                        if line.starts_with('#') {
                            // Headers - make them stand out
                            formatted.push_str(&format!("\n{}\n", line));
                        } else if line.starts_with("```") {
                            // Code blocks
                            formatted.push_str(&format!("{}\n", line));
                        } else if let Some(stripped) = line.strip_prefix('>') {
                            // Quotes
                            formatted.push_str(&format!("│ {}\n", stripped.trim()));
                        } else if line.starts_with("- ") || line.starts_with("* ") {
                            // Lists
                            formatted.push_str(&format!("  • {}\n", &line[2..].trim()));
                        } else {
                            formatted.push_str(&format!("{}\n", line));
                        }
                    }
                    formatted
                }
            };

            // Apply text wrapping if width is specified
            let final_content = if let Some(w) = width {
                wrap_text(&formatted_content, w)
            } else if let Ok(term_width) = std::env::var("COLUMNS") {
                if let Ok(w) = term_width.parse::<usize>() {
                    wrap_text(&formatted_content, w.saturating_sub(4))
                } else {
                    formatted_content
                }
            } else {
                formatted_content
            };

            output.push_str(&final_content);

            // Footer
            output.push_str(&format!("\n{}\n", "═".repeat(80)));
            output.push_str(&format!("📌 Post ID: {}\n", id));

            // Export to file if requested
            if let Some(export_path) = export {
                std::fs::write(&export_path, &output)?;
                println!("✅ Post exported to: {}", export_path);
                return Ok(());
            }

            // Use pager if requested
            if pager {
                use std::io::Write;
                use std::process::{Command, Stdio};

                let pager_cmd = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());

                if let Ok(mut pager_process) = Command::new(&pager_cmd)
                    .arg("-R") // Enable color support in less
                    .stdin(Stdio::piped())
                    .spawn()
                {
                    if let Some(stdin) = pager_process.stdin.take() {
                        let mut writer = std::io::BufWriter::new(stdin);
                        let _ = writer.write_all(output.as_bytes());
                        let _ = writer.flush();
                    }
                    let _ = pager_process.wait();
                } else {
                    // Fallback to regular print if pager fails
                    print!("{}", output);
                }
            } else {
                print!("{}", output);
            }
        }

        Commands::Search { query } => {
            let results = blog_manager.search_posts(&query).await?;

            if results.is_empty() {
                println!("No posts found matching '{query}'");
            } else {
                println!("🔍 Search results for '{query}':");
                println!("{:-<80}", "");
                for (storage_id, post) in results {
                    println!("ID: {storage_id}");
                    println!("Title: {}", post.title);
                    println!("Author: {}", post.author);
                    println!("{:-<80}", "");
                }
            }
        }

        Commands::TestStorage { backend } => {
            let test_content = b"Hello, decentralized world!";
            let mut metadata = HashMap::new();
            metadata.insert("content_type".to_string(), "text/plain".to_string());
            metadata.insert("test".to_string(), "true".to_string());

            let storage_backend = match backend.as_str() {
                "ipfs" => StorageBackend::Ipfs,
                "github" => StorageBackend::GitHub,
                _ => StorageBackend::Local,
            };

            let storage = storage_manager
                .get_backend(&storage_backend)
                .ok_or_else(|| anyhow::anyhow!("Storage backend '{}' not configured", backend))?;

            info!("Testing {} storage...", storage.storage_type());

            let result = storage.store(test_content, metadata).await?;
            println!("✅ Stored successfully!");
            println!("ID: {}", result.id);
            if let Some(url) = result.url {
                println!("URL: {url}");
            }

            let retrieved = storage.retrieve(&result.id).await?;
            println!("✅ Retrieved successfully!");
            println!("Content: {}", String::from_utf8_lossy(&retrieved));

            let exists = storage.exists(&result.id).await?;
            println!("✅ Exists check: {exists}");
        }

        Commands::Generate { output, config } => {
            let site_config = site::SiteConfig::load_from(&config).unwrap_or_default();
            let generator =
                site::generator::SiteGenerator::new(blog_manager, site_config, &output).await?;

            generator.generate().await?;
        }

        Commands::Init => {
            let config = site::SiteConfig::default();
            config.save()?;
            println!("✅ Site configuration initialized: site.toml");
            println!("📝 Edit site.toml to customize your blog settings");
        }

        Commands::Serve { port, config } => {
            let site_config = site::SiteConfig::load_from(&config).unwrap_or_default();
            let server = web::server::WebServer::new(blog_manager, site_config, port);

            println!("🌐 Starting web server...");
            println!("🔗 Visit http://localhost:{port}");
            println!("🔍 Search: http://localhost:{port}/search");
            println!("📚 Archive: http://localhost:{port}/archive");
            println!("📡 API: http://localhost:{port}/api/posts");
            println!("❌ Press Ctrl+C to stop");

            server.run().await?;
        }

        Commands::Edit {
            id,
            title,
            author,
            content,
            tags,
            category,
            editor,
        } => {
            // First, get the current post
            let posts = blog_manager.list_posts(false).await?;
            let post_data = posts
                .iter()
                .find(|(storage_id, _)| storage_id == &id || storage_id.starts_with(&id))
                .ok_or_else(|| anyhow::anyhow!("Post not found with ID: {}", id))?;

            let (storage_id, mut post) = post_data.clone();

            println!("📝 Editing post: {}", post.title);
            println!("   ID: {}", storage_id);

            // Update fields if provided
            let mut updated = false;

            if let Some(new_title) = title {
                post.title = new_title;
                updated = true;
            }

            if let Some(new_author) = author {
                post.author = new_author;
                updated = true;
            }

            if let Some(new_category) = category {
                post.category = Some(new_category);
                updated = true;
            }

            if let Some(tags_str) = tags {
                post.tags = tags_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                updated = true;
            }

            // Handle content update
            if let Some(content_path) = content {
                let content_path = std::path::Path::new(&content_path);
                let content_dir = content_path.parent();
                let new_content = std::fs::read_to_string(content_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read content file: {}", e))?;

                // Process images in new content
                println!("🖼️  Processing images...");
                let (processed_content, image_map) = crate::utils::process_images_in_markdown(
                    &new_content,
                    content_dir,
                    &storage_manager,
                )
                .await?;

                if !image_map.is_empty() {
                    println!("✅ Uploaded {} images to IPFS:", image_map.len());
                    for (local_path, ipfs_url) in &image_map {
                        println!("   {} -> {}", local_path, ipfs_url);
                    }
                }

                post.content = processed_content;
                updated = true;
            } else if editor {
                // Open in default editor
                let temp_file = format!("/tmp/kpgb-edit-{}.md", post.id);
                std::fs::write(&temp_file, &post.content)?;

                let editor_cmd = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                std::process::Command::new(&editor_cmd)
                    .arg(&temp_file)
                    .status()?;

                let edited_content = std::fs::read_to_string(&temp_file)?;
                if edited_content != post.content {
                    post.content = edited_content;
                    updated = true;
                }

                // Clean up temp file
                std::fs::remove_file(&temp_file).ok();
            }

            if !updated && !editor {
                println!("ℹ️  No changes specified. Use --title, --author, --content, --tags, --category, or --editor");
                return Ok(());
            }

            // Update the post in database
            blog_manager.update_post(&post).await?;

            println!("✅ Post updated successfully!");
            println!("   Title: {}", post.title);
            println!("   Author: {}", post.author);
            if !post.tags.is_empty() {
                println!("   Tags: {}", post.tags.join(", "));
            }
            if let Some(cat) = &post.category {
                println!("   Category: {}", cat);
            }
        }

        Commands::Delete { id, force } => {
            // First, get the post to delete
            let posts = blog_manager.list_posts(false).await?;
            let post_data = posts
                .iter()
                .find(|(storage_id, _)| storage_id == &id || storage_id.starts_with(&id))
                .ok_or_else(|| anyhow::anyhow!("Post not found with ID: {}", id))?;

            let (storage_id, post) = post_data;

            println!("🗑️  Post to delete:");
            println!("   Title: {}", post.title);
            println!("   Author: {}", post.author);
            println!("   Created: {}", post.created_at.format("%Y-%m-%d %H:%M"));
            println!("   ID: {}", storage_id);

            // Confirm deletion unless --force is used
            if !force {
                print!("\n⚠️  Are you sure you want to delete this post? (y/N): ");
                use std::io::{self, Write};
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();

                if input != "y" && input != "yes" {
                    println!("❌ Deletion cancelled");
                    return Ok(());
                }
            }

            // Delete from database
            blog_manager.delete_post(&post.id).await?;

            println!("✅ Post deleted successfully!");
            println!("   Note: The content may still exist in IPFS if pinned elsewhere");
        }

        Commands::Stats { detailed, json } => {
            let posts = blog_manager.list_posts(false).await?;
            let tags = blog_manager.get_all_tags().await?;

            // Calculate basic statistics
            let total_posts = posts.len();
            let published_posts = posts.iter().filter(|(_, p)| p.published).count();
            let draft_posts = total_posts - published_posts;

            // Calculate word statistics
            let mut total_words = 0;
            let mut total_chars = 0;
            let mut author_stats: std::collections::HashMap<String, (usize, usize)> =
                std::collections::HashMap::new();
            let mut posts_by_month: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            let mut category_stats: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for (_, post) in &posts {
                // Count words (split by whitespace)
                let words = post.content.split_whitespace().count();
                let chars = post.content.chars().count();
                total_words += words;
                total_chars += chars;

                // Author statistics
                let entry = author_stats.entry(post.author.clone()).or_insert((0, 0));
                entry.0 += 1; // post count
                entry.1 += words; // word count

                // Monthly statistics
                let month = post.created_at.format("%Y-%m").to_string();
                *posts_by_month.entry(month).or_insert(0) += 1;

                // Category statistics
                if let Some(cat) = &post.category {
                    *category_stats.entry(cat.clone()).or_insert(0) += 1;
                }
            }

            let avg_words = if total_posts > 0 {
                total_words / total_posts
            } else {
                0
            };
            let avg_chars = if total_posts > 0 {
                total_chars / total_posts
            } else {
                0
            };

            if json {
                // Export as JSON
                let stats = serde_json::json!({
                    "total_posts": total_posts,
                    "published_posts": published_posts,
                    "draft_posts": draft_posts,
                    "total_words": total_words,
                    "total_characters": total_chars,
                    "average_words_per_post": avg_words,
                    "average_chars_per_post": avg_chars,
                    "total_tags": tags.len(),
                    "authors": author_stats,
                    "posts_by_month": posts_by_month,
                    "categories": category_stats,
                    "tags": tags,
                });
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                // Display formatted statistics
                println!("📊 Blog Statistics");
                println!("==================");
                println!();
                println!("📝 Posts:");
                println!("   Total:     {}", total_posts);
                println!(
                    "   Published: {} ({}%)",
                    published_posts,
                    if total_posts > 0 {
                        published_posts * 100 / total_posts
                    } else {
                        0
                    }
                );
                println!(
                    "   Drafts:    {} ({}%)",
                    draft_posts,
                    if total_posts > 0 {
                        draft_posts * 100 / total_posts
                    } else {
                        0
                    }
                );
                println!();
                println!("📖 Content:");
                println!("   Total words:      {:>8}", total_words.to_string());
                println!("   Total characters: {:>8}", total_chars.to_string());
                println!("   Avg words/post:   {:>8}", avg_words.to_string());
                println!("   Avg chars/post:   {:>8}", avg_chars.to_string());
                println!();
                println!("🏷️  Tags: {} unique tags", tags.len());

                if detailed {
                    // Show top 5 tags
                    println!("\n   Top Tags:");
                    for (i, (tag, count)) in tags.iter().take(5).enumerate() {
                        println!("   {}. {} ({})", i + 1, tag, count);
                    }

                    // Show author statistics
                    println!("\n✍️  Authors:");
                    let mut authors: Vec<_> = author_stats.iter().collect();
                    authors.sort_by(|a, b| b.1 .0.cmp(&a.1 .0)); // Sort by post count
                    for (author, (posts, words)) in authors.iter().take(5) {
                        println!("   {} - {} posts, {} words", author, posts, words);
                    }

                    // Show category statistics
                    if !category_stats.is_empty() {
                        println!("\n📁 Categories:");
                        let mut categories: Vec<_> = category_stats.iter().collect();
                        categories.sort_by(|a, b| b.1.cmp(a.1));
                        for (cat, count) in categories.iter().take(5) {
                            println!("   {} - {} posts", cat, count);
                        }
                    }

                    // Show posting frequency
                    println!("\n📅 Recent Activity:");
                    let mut months: Vec<_> = posts_by_month.iter().collect();
                    months.sort_by(|a, b| b.0.cmp(a.0)); // Sort by month descending
                    for (month, count) in months.iter().take(6) {
                        let bar = "█".repeat((**count).min(20));
                        println!("   {} [{:>2}] {}", month, count, bar);
                    }
                }

                // Fun stats
                println!("\n🎉 Fun Facts:");
                if total_posts > 0 {
                    // Find longest post
                    let longest = posts
                        .iter()
                        .max_by_key(|(_, p)| p.content.len())
                        .map(|(_, p)| (&p.title, p.content.split_whitespace().count()));

                    if let Some((title, words)) = longest {
                        println!("   Longest post: \"{}\" ({} words)", title, words);
                    }

                    // Most used tag
                    if let Some((tag, count)) = tags.first() {
                        println!("   Most popular tag: \"{}\" (used {} times)", tag, count);
                    }

                    // Estimate reading time for all posts
                    let reading_minutes = total_words / 200; // Average reading speed
                    println!(
                        "   Time to read all posts: ~{} minutes ({} hours)",
                        reading_minutes,
                        reading_minutes / 60
                    );
                }
            }
        }

        Commands::Random { published, tag } => {
            use rand::seq::SliceRandom;

            let posts = if let Some(ref tag_filter) = tag {
                blog_manager.get_posts_by_tag(tag_filter, published).await?
            } else {
                blog_manager.list_posts(published).await?
            };

            if posts.is_empty() {
                println!("❌ No posts found");
                if tag.is_some() {
                    println!("   Try without tag filter or use a different tag");
                }
                return Ok(());
            }

            // Pick a random post
            let mut rng = rand::thread_rng();
            let (storage_id, post) = posts.choose(&mut rng).unwrap();

            // Display the post
            println!("🎲 Random post selected!\n");
            println!("═══════════════════════════════════════════════════════════════");
            println!("Title:    {}", post.title);
            println!("Author:   {}", post.author);
            println!("Date:     {}", post.created_at.format("%Y-%m-%d %H:%M"));
            if !post.tags.is_empty() {
                println!("Tags:     {}", post.tags.join(", "));
            }
            if let Some(cat) = &post.category {
                println!("Category: {}", cat);
            }
            println!("ID:       {}", storage_id);
            println!("═══════════════════════════════════════════════════════════════");
            println!();

            // Show excerpt or beginning of content
            if let Some(excerpt) = &post.excerpt {
                println!("{}\n", excerpt);
                println!("[...continue reading...]");
            } else {
                // Show first 500 characters
                let preview: String = post.content.chars().take(500).collect();
                println!("{}", preview);
                if post.content.len() > 500 {
                    println!("\n[...continue reading...]");
                }
            }

            println!("\n💡 To read the full post, run:");
            println!("   cargo run -- read {}", storage_id);
        }
    }

    Ok(())
}
