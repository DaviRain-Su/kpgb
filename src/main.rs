mod models;
mod storage;
mod blog;
mod database;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber;
use std::collections::HashMap;

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
        #[arg(short, long)]
        title: String,
        
        /// Author name
        #[arg(short, long)]
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
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./kpgb.db".to_string());
    
    let mut blog_manager = BlogManager::new(storage_manager.clone(), &database_url).await?;
    
    match cli.command {
        Commands::New { title, author, content } => {
            let content_text = match content {
                Some(path) => std::fs::read_to_string(path)?,
                None => {
                    println!("Enter content (press Ctrl+D when done):");
                    use std::io::Read;
                    let mut buffer = String::new();
                    std::io::stdin().read_to_string(&mut buffer)?;
                    buffer
                }
            };
            
            let post = BlogPost::new(title.clone(), content_text, author);
            let storage_id = blog_manager.create_post(post).await?;
            
            println!("✅ Post created successfully!");
            println!("Storage ID: {}", storage_id);
            println!("Title: {}", title);
        }
        
        Commands::List { published } => {
            let posts = blog_manager.list_posts(published).await?;
            
            if posts.is_empty() {
                println!("No posts found.");
            } else {
                println!("📝 Blog Posts:");
                println!("{:-<80}", "");
                for (storage_id, post) in posts {
                    println!("ID: {}", storage_id);
                    println!("Title: {}", post.title);
                    println!("Author: {}", post.author);
                    println!("Created: {}", post.created_at.format("%Y-%m-%d %H:%M"));
                    println!("Published: {}", if post.published { "Yes" } else { "No" });
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
        
        Commands::Read { id } => {
            let post = blog_manager.get_post(&id).await?;
            println!("📖 {}", post.title);
            println!("By: {} | {}", post.author, post.created_at.format("%Y-%m-%d"));
            if !post.tags.is_empty() {
                println!("Tags: {}", post.tags.join(", "));
            }
            println!("\n{}", post.content);
        }
        
        Commands::Search { query } => {
            let results = blog_manager.search_posts(&query).await?;
            
            if results.is_empty() {
                println!("No posts found matching '{}'", query);
            } else {
                println!("🔍 Search results for '{}':", query);
                println!("{:-<80}", "");
                for (storage_id, post) in results {
                    println!("ID: {}", storage_id);
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
            
            let storage = storage_manager.get_backend(&storage_backend)
                .ok_or_else(|| anyhow::anyhow!("Storage backend '{}' not configured", backend))?;
            
            info!("Testing {} storage...", storage.storage_type());
            
            let result = storage.store(test_content, metadata).await?;
            println!("✅ Stored successfully!");
            println!("ID: {}", result.id);
            if let Some(url) = result.url {
                println!("URL: {}", url);
            }
            
            let retrieved = storage.retrieve(&result.id).await?;
            println!("✅ Retrieved successfully!");
            println!("Content: {}", String::from_utf8_lossy(&retrieved));
            
            let exists = storage.exists(&result.id).await?;
            println!("✅ Exists check: {}", exists);
        }
    }
    
    Ok(())
}