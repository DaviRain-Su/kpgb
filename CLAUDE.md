# KPGB - Kaspa-Powered Genesis Blog

## Project Overview
A decentralized personal blog system where all content is stored on IPFS (InterPlanetary File System). The system supports local content indexing for enhanced functionality while maintaining full decentralization. Features both static site generation and dynamic web interface.

## Current Implementation Status

### ✅ Completed Features (100%)

#### 1. **Pure IPFS Decentralized Blog System**
- Full IPFS integration using HTTP API
- Content storage with automatic pinning
- Content retrieval by CID
- Immutable content guarantee
- Verified working with CIDs like: QmQmWyC1JXi269pT6J6Jnip9mP5aWHcquDfd7DFCjAYFo2

#### 2. **Multiple Storage Backends**
- **IPFS**: Primary decentralized storage (requires running IPFS node)
- **Local**: File system storage for testing/fallback
- **GitHub**: Store blog posts in GitHub repository
- Pluggable storage trait system for easy extension

#### 3. **SQLite Database Index with FTS5**
- Fast local indexing with SQLite
- Full-text search using FTS5
- Tag management system
- Content deduplication by SHA256 hash
- Efficient querying and filtering
- Complete database migrations

#### 4. **Content Management**
- Create, publish, and search blog posts
- Markdown content support with pulldown-cmark
- Automatic slug generation
- Tag and category support
- Excerpt generation
- Draft/published status management

#### 5. **Static Site Generation**
- Tera template engine with inheritance
- Responsive CSS design
- Generate complete static websites
- RSS 2.0 feed generation
- Archive pages grouped by year
- Mobile-friendly design

#### 6. **Dynamic Web Interface**
- Axum web server with async support
- Real-time search functionality
- RESTful API endpoints
- Dynamic routing for posts
- CORS support for API access

#### 7. **Complete CLI Interface**
```bash
# Create new post
cargo run -- new --title "Title" --author "Author" [--content file.md]

# List posts
cargo run -- list [--published]

# Publish post
cargo run -- publish <storage-id>

# Read post
cargo run -- read <storage-id>

# Search posts
cargo run -- search "query"

# Test storage
cargo run -- test-storage --backend [ipfs|local|github]

# Initialize site config
cargo run -- init

# Generate static site
cargo run -- generate [--output ./public]

# Start web server
cargo run -- serve [--port 9000]
```

## Technical Architecture

### Storage Trait System
```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn store(&self, content: &[u8], metadata: HashMap<String, String>) -> Result<StorageResult>;
    async fn retrieve(&self, id: &str) -> Result<Vec<u8>>;
    async fn exists(&self, id: &str) -> Result<bool>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn list(&self, prefix: Option<&str>) -> Result<Vec<StorageMetadata>>;
    fn storage_type(&self) -> &'static str;
}
```

### Database Schema
```sql
-- Posts table with full metadata
CREATE TABLE posts (
    id TEXT PRIMARY KEY,
    storage_id TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    author TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    published BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    published_at TIMESTAMP
);

-- FTS5 virtual table for search
CREATE VIRTUAL TABLE posts_fts USING fts5(
    title, content, author, tags,
    content=posts,
    content_rowid=rowid
);

-- Tags system
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE post_tags (
    post_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (post_id, tag_id)
);
```

### Web Routes
```
# Web UI Routes
GET  /                    # Home page with paginated posts
GET  /posts/:slug         # Individual post page
GET  /archive             # Archive page grouped by year
GET  /search?q=query      # Search results page
GET  /feed.xml            # RSS feed
GET  /css/style.css       # Stylesheet

# API Routes
GET  /api/posts           # JSON API: List posts
GET  /api/posts/:id       # JSON API: Get post
POST /api/search          # JSON API: Search posts
```

### Content Flow
1. User creates post via CLI
2. Content hash calculated (SHA256)
3. Check for duplicate content in database
4. If new: Store in configured backend (IPFS/Local/GitHub)
5. Save metadata to SQLite index
6. Return storage ID (CID for IPFS)

## Configuration

### Environment Variables (.env)
```bash
# Database
DATABASE_URL=sqlite:./kpgb.db

# IPFS Configuration
IPFS_API_URL=http://localhost:5001

# GitHub Configuration (optional)
GITHUB_OWNER=your-username
GITHUB_REPO=your-repo
GITHUB_BRANCH=main
GITHUB_TOKEN=your-token
```

### Site Configuration (site.toml)
```toml
title = "My IPFS Blog"
description = "A decentralized blog powered by IPFS"
author = "Your Name"
base_url = "http://localhost:8080"
posts_per_page = 10
ipfs_gateway = "https://ipfs.io/ipfs/"
```

## Getting Started

### 1. Install IPFS (for decentralized storage)
```bash
# macOS
brew install ipfs

# Linux
wget https://dist.ipfs.io/go-ipfs/v0.12.0/go-ipfs_v0.12.0_linux-amd64.tar.gz
tar -xvzf go-ipfs_v0.12.0_linux-amd64.tar.gz
cd go-ipfs
sudo bash install.sh

# Initialize IPFS
ipfs init

# Start IPFS daemon
ipfs daemon
```

### 2. Set up the project
```bash
# Clone and setup
git clone <repo>
cd kpgb
cp .env.example .env

# Build
cargo build --release
```

### 3. Create your first post
```bash
# Interactive mode
cargo run -- new --title "My First Post" --author "Alice"

# From file
echo "# Hello IPFS World" > first-post.md
cargo run -- new --title "My First Post" --author "Alice" --content first-post.md

# Publish it
cargo run -- publish <storage-id>

# Start web server
cargo run -- serve --port 9000
# Visit http://localhost:9000
```

## Important Implementation Notes

### IPFS Integration
- Uses HTTP API instead of native library due to async/Send trait compatibility
- Implements `.no_proxy()` on reqwest client to avoid proxy interference
- Supports both local IPFS nodes and remote IPFS gateways
- Content verification through CID

### Port Conflicts
Common ports that may be in use:
- 3000: Often used by development servers
- 4001: IPFS Swarm port
- 5000: AirPlay on macOS
- 8080: Common development port
- **Recommended**: Use ports 9000+ to avoid conflicts

### HTTP Proxy Issues
If experiencing connection issues:
```bash
unset http_proxy
unset https_proxy
# Or use --noproxy flag
curl --noproxy '*' http://localhost:9000
```

### Template System
- Tera templates with inheritance support
- Templates compiled into binary for easy distribution
- Responsive design with mobile support
- Dark mode ready CSS structure

### Performance Optimizations
- Content deduplication saves storage and bandwidth
- Local caching reduces IPFS queries
- Paginated post listings
- Lazy loading of content
- SQLite FTS5 for fast searches

## Project Structure
```
kpgb/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── models/          # Data models
│   ├── storage/         # Storage backends
│   │   ├── mod.rs       # Storage trait
│   │   ├── ipfs.rs      # IPFS implementation
│   │   ├── github.rs    # GitHub implementation
│   │   └── local.rs     # Local storage
│   ├── blog/            # Blog management
│   ├── database/        # SQLite operations
│   ├── site/            # Static site generator
│   │   ├── mod.rs       # Site config
│   │   └── generator.rs # HTML generation
│   └── web/             # Web server
│       ├── mod.rs       # Router setup
│       ├── server.rs    # Axum server
│       ├── handlers.rs  # Web handlers
│       └── api.rs       # API endpoints
├── templates/           # Tera templates
│   ├── base.html        # Base template
│   ├── index.html       # Home page
│   ├── post.html        # Post page
│   ├── archive.html     # Archive page
│   ├── search.html      # Search page
│   └── style.css        # Styles
├── migrations/          # Database migrations
├── Cargo.toml           # Dependencies
└── CLAUDE.md            # This file
```

## Security Considerations
- All content on IPFS is public and immutable
- Use environment variables for sensitive data
- GitHub storage requires access token
- SQLite database is local only
- No authentication in current version
- Consider encryption for private content (future)

## Testing

### Storage Backend Testing
```bash
# Test IPFS
cargo run -- test-storage --backend ipfs

# Test local storage
cargo run -- test-storage --backend local

# Test GitHub
cargo run -- test-storage --backend github
```

### Content Verification
```bash
# Verify IPFS content
ipfs cat <CID>

# Via gateway
curl https://ipfs.io/ipfs/<CID>
```

## Development Guidelines

### Adding New Storage Backends
1. Implement the `Storage` trait in `src/storage/`
2. Add backend to `StorageBackend` enum
3. Initialize in `main.rs`
4. Add environment configuration

### Database Migrations
- Place SQL files in `migrations/` directory
- Use numbered prefixes (e.g., `001_create_posts.sql`)
- Migrations run automatically on startup

### Content Deduplication
- Content is hashed before storage
- Duplicate content reuses existing storage ID
- Metadata can differ for same content

## Future Enhancements (Roadmap)

### Phase 1: Core Improvements
- [ ] IPNS support for mutable references
- [ ] Content encryption for private posts
- [ ] Media management (images, videos)
- [ ] Theme system with multiple themes
- [ ] Plugin architecture

### Phase 2: Kaspa Integration
- [ ] Wallet-based authentication
- [ ] Token-gated content
- [ ] KAS payment for premium posts
- [ ] Author tips/donations
- [ ] Content monetization

### Phase 3: Advanced Features
- [ ] P2P comment system
- [ ] Distributed search across nodes
- [ ] Multi-author support
- [ ] Content versioning
- [ ] Mobile app
- [ ] Analytics without tracking
- [ ] Federation support

## Dependencies

- **tokio**: Async runtime
- **axum**: Web framework
- **sqlx**: Database with compile-time checking
- **tera**: Template engine
- **reqwest**: HTTP client for APIs
- **clap**: CLI parsing
- **serde**: Serialization
- **sha2**: Content hashing
- **chrono**: Date/time handling
- **uuid**: Unique identifiers
- **rss**: RSS feed generation
- **pulldown-cmark**: Markdown parsing

## Important Reminders

- Always check for duplicate content before storage
- Maintain backward compatibility with existing posts
- Test with local storage before using IPFS
- Keep SQLite index in sync with storage
- Handle IPFS daemon connection errors gracefully
- Use appropriate ports to avoid conflicts
- Remember to set environment variables
- Backup your SQLite database regularly