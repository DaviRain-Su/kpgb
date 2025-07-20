# KPGB - Kaspa-Powered Genesis Blog

## Current Implementation Status

### ✅ Completed Features

#### 1. **Pure IPFS Decentralized Blog System**
- Full IPFS integration using HTTP API
- Content storage with automatic pinning
- Content retrieval by CID
- Immutable content guarantee

#### 2. **Multiple Storage Backends**
- **IPFS**: Primary decentralized storage (requires running IPFS node)
- **Local**: File system storage for testing/fallback
- **GitHub**: Store blog posts in GitHub repository

#### 3. **SQLite Database Index**
- Fast local indexing with SQLite
- Full-text search using FTS5
- Tag management system
- Content deduplication by hash
- Efficient querying and filtering

#### 4. **Content Management**
- Create, publish, and search blog posts
- Markdown content support
- Automatic slug generation
- Tag and category support
- Excerpt support
- Content hash for deduplication

#### 5. **CLI Interface**
```bash
# Create new post
cargo run -- new --title "Title" --author "Author" --content file.md

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
```

### 🔧 Technical Architecture

#### Storage Trait System
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

#### Database Schema
- **posts**: Main blog post table with full metadata
- **tags**: Tag definitions
- **post_tags**: Many-to-many relationship
- **posts_fts**: Full-text search virtual table

#### Content Flow
1. User creates post via CLI
2. Content hash calculated (SHA256)
3. Check for duplicate content in database
4. If new: Store in configured backend (IPFS/Local/GitHub)
5. Save metadata to SQLite index
6. Return storage ID (CID for IPFS)

### 📝 Configuration

#### Environment Variables (.env)
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

### 🚀 Getting Started

1. **Install IPFS (for decentralized storage)**
   ```bash
   # Install IPFS
   # macOS: brew install ipfs
   # Linux: See https://docs.ipfs.io/install/
   
   # Initialize IPFS
   ipfs init
   
   # Start IPFS daemon
   ipfs daemon
   ```

2. **Set up the project**
   ```bash
   # Clone and setup
   git clone <repo>
   cd kpgb
   cp .env.example .env
   
   # Build
   cargo build --release
   ```

3. **Create your first post**
   ```bash
   # Create a markdown file
   echo "# Hello IPFS World" > first-post.md
   
   # Publish to IPFS
   cargo run -- new --title "My First Post" --author "Alice" --content first-post.md
   
   # List posts
   cargo run -- list
   ```

### 🔮 Future Enhancements (TODO)

1. **Static Site Generator**
   - Export blog as static HTML
   - Theme support
   - RSS feed generation

2. **Web UI**
   - Browser-based blog viewer
   - IPFS gateway integration
   - Search interface

3. **Kaspa Integration**
   - Wallet-based authentication
   - Token-gated content
   - KAS payment for premium posts
   - Author tips/donations

4. **Advanced Features**
   - IPNS for mutable references
   - P2P comment system
   - Multi-author support
   - Content versioning

### 🏗️ Development Guidelines

#### Adding New Storage Backends
1. Implement the `Storage` trait
2. Add backend to `StorageBackend` enum
3. Initialize in `main.rs`

#### Database Migrations
- Place SQL files in `migrations/` directory
- Use numbered prefixes (e.g., `001_create_posts.sql`)
- Migrations run automatically on startup

#### Content Deduplication
- Content is hashed before storage
- Duplicate content reuses existing storage ID
- Metadata can differ for same content

### 🔒 Security Considerations

- All content on IPFS is public
- Use environment variables for sensitive data
- GitHub storage requires access token
- SQLite database is local only
- Consider encryption for private content

### 📚 Dependencies

- **tokio**: Async runtime
- **sqlx**: Database access with compile-time checking
- **reqwest**: HTTP client for IPFS/GitHub APIs
- **clap**: CLI argument parsing
- **serde**: Serialization
- **sha2**: Content hashing
- **chrono**: Date/time handling
- **uuid**: Unique identifiers

---

## Important Instructions

- Always check for duplicate content before storage
- Maintain backward compatibility with existing posts
- Test with local storage before using IPFS
- Keep SQLite index in sync with storage
- Handle IPFS daemon connection errors gracefully