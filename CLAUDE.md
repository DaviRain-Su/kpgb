# KPGB - IPFS-Based Decentralized Personal Blog System

## Project Overview
A fully decentralized personal blog system where all content is stored on IPFS (InterPlanetary File System). The system supports local content indexing for enhanced functionality while maintaining full decentralization. Features both static site generation for GitHub Pages deployment and dynamic web server with API endpoints.

### Is KPGB Ready for Blog Writing?
**Yes!** KPGB now provides a complete blog writing experience:
- ✅ **Core Publishing**: Create, publish, and manage posts with markdown
- ✅ **Beautiful Output**: 5 themes, responsive design, RSS feed
- ✅ **Decentralized Storage**: Your content on IPFS forever
- ✅ **Post Management**: Edit and delete commands
- ✅ **Automatic Image Upload**: Local images auto-uploaded to IPFS
- ✅ **Comments**: Giscus integration for reader engagement
- ✅ **Statistics**: Blog analytics and random post discovery
- 📝 **Active Development**: See [Blog Writing Features](#using-kpgb-as-a-blog-writing-tool) section for details

## Recent Updates (2025-07-23)
- ✅ **Image Auto-Upload**: Local images automatically uploaded to IPFS
- ✅ **Comment System**: Giscus integration for reader discussions
- ✅ **Post Management**: Edit and delete commands for easy content updates
- ✅ **Blog Statistics**: Analytics with word count, author stats, and posting trends
- ✅ **Random Post**: Discover random articles with tag filtering
- ✅ **Better Reading**: Enhanced terminal reading with pager, formatting, and export
- ✅ **Batch Import**: Import existing Jekyll/Markdown posts

## Previous Updates (2025-07-22)
- ✅ **Tag System**: Complete tag management with cloud view and filtering
- ✅ **Pagination**: Clean URL pagination for better navigation
- ✅ **Auto Excerpts**: Automatic excerpt generation from content
- ✅ **URL Translation**: English to Chinese translation using Claude API
- ✅ **ASCII-only Slugs**: Fixed GitHub Pages compatibility issues
- ✅ **Documentation Center**: Added technical documentation section with NautilusTrader docs

## Current Implementation Status

### ✅ Completed Features

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
- Automatic slug generation (ASCII-only for GitHub Pages compatibility)
- Tag and category support
- **Automatic excerpt generation** from content
- Draft/published status management
- **Edit and delete** existing posts
- **Batch import** from Jekyll/Markdown files

#### 5. **Automatic Image Upload to IPFS**
- Detects local images in markdown content
- Automatically uploads images to IPFS during post creation/editing
- Replaces local paths with permanent IPFS URLs
- Supports PNG, JPEG, GIF, WebP, SVG formats
- Preserves image alt text
- Skips already uploaded images (HTTP/IPFS URLs)
- Works with relative and absolute paths

#### 6. **Static Site Generation**
- Tera template engine with inheritance
- Responsive CSS design
- Generate complete static websites
- RSS 2.0 feed generation
- Archive pages grouped by year
- Mobile-friendly design
- GitHub Pages support with base_path configuration
- Production-ready deployment scripts
- **Multiple themes**: default, hacker, minimal, dark, cyberpunk

#### 7. **Dynamic Web Interface**
- Axum web server with async support
- Real-time search functionality
- RESTful API endpoints
- Dynamic routing for posts
- CORS support for API access
- Embeddable widget support

#### 8. **GitHub Pages Deployment**
- Automated deployment scripts
- GitHub Actions CI/CD workflows
- Base path configuration for subdirectory deployment
- Static asset optimization
- SEO-friendly URLs

#### 9. **External Website Integration**
- Embeddable JavaScript widget
- CORS-enabled API endpoints
- Client-side search functionality
- Customizable themes
- Demo page for testing integration

#### 10. **Tag System**
- Complete tag management with SQLite backend
- Tag cloud page showing all tags with post counts
- Tag-filtered post listings
- API endpoints for tag queries (`/api/tags`, `/api/tags/:tag`)
- Static generation of tag pages
- Clean URL structure for tag navigation

#### 11. **Pagination System**
- Paginated post listings (configurable posts per page)
- Clean URL structure (`/page/2/`, `/page/3/`)
- Pagination navigation UI with page numbers
- Support for both dynamic and static site generation
- Efficient database queries with LIMIT/OFFSET

#### 12. **URL Content Translation**
- Fetch and translate web content from English to Chinese
- Uses Claude API for high-quality translations
- HTML to text conversion
- Option to save translations as blog posts
- Preserves both original and translated content
- Custom title and author support

#### 13. **Comment System (Giscus)**
- GitHub Discussions-based comments
- Zero-cost, ad-free solution
- Supports reactions and threading
- Configurable themes and languages
- Easy setup with GitHub repo
- Respects user privacy

#### 14. **Blog Management Tools**
- **Edit Command**: Modify posts with metadata updates
- **Delete Command**: Remove posts with confirmation
- **Statistics**: Word count, author analytics, posting trends
- **Random Discovery**: Find random posts with tag filtering
- **Enhanced Reading**: Terminal pager, formatting options, export
- **Batch Import**: Import existing Jekyll/Markdown posts

#### 15. **Reading Time Estimation**
- Automatic calculation based on word count and content type
- Support for both English and Chinese content
- Code block consideration for technical articles
- Display in post lists and article headers
- Accurate estimation: 200 WPM for English, 300 CPM for Chinese

### 🚧 Pending Features

#### Core Blog Writing Features (High Priority)

1. **Advanced Media Management**
   - Automatic image optimization before upload
   - Media library browser UI
   - Drag-and-drop support in web interface
   - Video and audio file support

2. **Enhanced SEO & Metadata**
   - Meta descriptions support
   - Open Graph tags for social sharing
   - XML sitemap generation
   - Canonical URLs

3. **Live Preview & Writing Tools**
   - Real-time markdown preview
   - Side-by-side editing mode
   - Word count and reading time
   - Auto-save functionality

#### Enhanced Features (Medium Priority)

4. **Site Search Page Optimization** (功能4)
   - Enhanced search UI/UX
   - Search result highlighting
   - Advanced search filters


5. **Related Articles Recommendation** (功能6)
   - Content-based article recommendations
   - Tag-based similarity matching
   - Display related posts at the end of articles

6. **Navigation & Organization**
   - Previous/Next post links
   - Post series/collections
   - Custom page support (About, Contact)
   - Better category management

#### Advanced Features (Low Priority)

7. **Advanced Analytics & Insights**
    - View statistics tracking
    - Popular content analytics
    - Reader engagement metrics
    - Content performance dashboard
    - Export analytics data

#### 16. **Complete CLI Interface**
```bash
# Create new post (with automatic image upload to IPFS)
cargo run -- new --title "Title" --author "Author" [--content file.md]

# List posts
cargo run -- list [--published]

# Publish post
cargo run -- publish <storage-id>

# Read post (with enhanced terminal display)
cargo run -- read <storage-id> [--pager] [--format plain|markdown|html] [--width 80] [--export file.txt]

# Edit post (with automatic image upload)
cargo run -- edit <storage-id> [--title "New Title"] [--author "New Author"] [--content file.md] [--tags "tag1,tag2"] [--category "Tech"] [--editor]

# Delete post
cargo run -- delete <storage-id> [--force]

# Blog statistics
cargo run -- stats [--detailed] [--json]

# Random post discovery
cargo run -- random [--published] [--tag "tech"]

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

# Translate URL content (English to Chinese)
cargo run -- translate <url> [--save] [--title "Custom Title"] [--author "Name"]

# Deploy to GitHub Pages
./scripts/deploy.sh
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
GET  /archive.html        # Archive page grouped by year
GET  /search?q=query      # Search results page
GET  /feed.xml            # RSS feed
GET  /css/style.css       # Stylesheet

# API Routes
GET  /api/posts           # JSON API: List posts
GET  /api/posts/:id       # JSON API: Get post
POST /api/search          # JSON API: Search posts
```

### Deployment Architecture

#### Static Deployment (GitHub Pages)
- Generate static HTML files
- Deploy to GitHub Pages at `/username/kpgb/`
- Base path configuration for subdirectory
- No server required
- Ideal for personal blogs

#### Dynamic Deployment (Server)
- Run Axum web server
- Real-time content updates
- API access for external sites
- Search functionality
- Ideal for multi-user scenarios

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

# Claude API Configuration (for translate command)
ANTHROPIC_API_KEY=your-claude-api-key
```

### Site Configuration (site.toml)
```toml
title = "My IPFS Blog"
description = "A decentralized blog powered by IPFS"
author = "Your Name"
base_url = "https://username.github.io/kpgb"
base_path = "/kpgb"  # For GitHub Pages subdirectory
ipfs_gateway = "https://ipfs.io/ipfs/"
posts_per_page = 10
enable_rss = true
theme = "default"
```

### Production Configuration (site.prod.toml)
```toml
title = "My IPFS Blog"
description = "A decentralized blog powered by IPFS"
author = "Your Name"
base_url = "https://username.github.io/kpgb"
base_path = "/kpgb"
ipfs_gateway = "https://ipfs.io/ipfs/"
posts_per_page = 10
enable_rss = true
theme = "default"
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

# Generate static site
cargo run -- generate

# Or start web server
cargo run -- serve --port 9000
# Visit http://localhost:9000
```

### 3.1 Translate web content to Chinese
```bash
# Set API key
export ANTHROPIC_API_KEY="your-api-key"

# Translate and display in console
cargo run -- translate https://example.com/article

# Translate and save as blog post
cargo run -- translate https://example.com/article --save --title "翻译文章标题"
```

### 4. Deploy to GitHub Pages
```bash
# Configure git
git config user.name "Your Name"
git config user.email "your-email@example.com"

# Option 1: Using Personal Access Token
export GITHUB_TOKEN="your-token-here"
./scripts/deploy.sh

# Option 2: Using SSH
git remote set-url origin git@github.com:username/kpgb.git
./scripts/deploy.sh
```

## GitHub Actions CI/CD

The project includes two GitHub Actions workflows:

### 1. CI Workflow (.github/workflows/ci.yml)
- Runs on every push and pull request
- Runs `cargo clippy` with strict warnings
- Runs `cargo test`
- Ensures code quality

### 2. Deploy Workflow (.github/workflows/deploy.yml)
- Manually triggered or on push to main
- Generates static site with production config
- Deploys to GitHub Pages
- Accessible at `https://username.github.io/kpgb/`

## External Website Integration

### Embedding the Blog Widget
```html
<!-- Add to your website -->
<div id="kpgb-widget"></div>
<script src="https://username.github.io/kpgb/widget.js"></script>
<script>
  KPGBWidget.init({
    container: '#kpgb-widget',
    apiUrl: 'https://your-server.com',
    postsPerPage: 5,
    theme: 'light',
    showSearch: true,
    showTags: true
  });
</script>
```

### Using the API
```javascript
// Fetch posts
fetch('https://your-server.com/api/posts')
  .then(res => res.json())
  .then(posts => console.log(posts));

// Search posts
fetch('https://your-server.com/api/search', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ query: 'IPFS' })
})
  .then(res => res.json())
  .then(results => console.log(results));
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
- Base path support for subdirectory deployment

### Performance Optimizations
- Content deduplication saves storage and bandwidth
- Local caching reduces IPFS queries
- Paginated post listings
- Lazy loading of content
- SQLite FTS5 for fast searches
- Client-side search for static deployments

### URL Path Handling
- Static sites use `base_path` from configuration for GitHub Pages
- Dynamic web server automatically removes `base_path` from all templates
- This ensures local development doesn't include `/kpgb` prefix in links
- Both systems use same URL patterns: `/archive`, `/feed.xml`, etc.

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
│   ├── style.css        # Styles
│   ├── widget.js        # Embeddable widget
│   └── widget-demo.html # Widget demo page
├── scripts/
│   └── deploy.sh        # GitHub Pages deployment
├── .github/workflows/
│   ├── ci.yml           # CI workflow
│   └── deploy.yml       # Deploy workflow
├── migrations/          # Database migrations
├── public/              # Generated static site
├── Cargo.toml           # Dependencies
├── site.toml            # Development config
├── site.prod.toml       # Production config
└── CLAUDE.md            # This file
```

## Code Quality
- All clippy warnings fixed
- Proper error handling throughout
- Async/await patterns correctly implemented
- No unsafe code
- Comprehensive type safety

## Security Considerations
- All content on IPFS is public and immutable
- Use environment variables for sensitive data
- GitHub storage requires access token
- SQLite database is local only
- No authentication in current version
- CORS configured for API access
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

## Architecture Documents

### ARCHITECTURE.md
Comprehensive system architecture covering:
- Dual deployment modes (static vs dynamic)
- Component interaction diagrams
- Data flow documentation
- API specifications
- Security model

### ROADMAP.md
Detailed development roadmap with:
- Phase 1: Index Sharing Protocol
- Phase 2: Public Aggregation Platform
- Phase 3: Advanced Features
- Phase 4: Ecosystem Development

## Future Vision

This system is designed to be the foundation for a decentralized content aggregation network where:
- Each person maintains their own blog with data on IPFS
- Indexes can be shared between nodes
- Public aggregation sites can display trending content
- Content remains under individual control
- No central authority controls the network

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
- **base64**: Encoding for GitHub API
- **hex**: Hash encoding
- **dotenv**: Environment management
- **html2text**: HTML to text conversion for translate command

## Using KPGB as a Blog Writing Tool

### Current Blog Writing Capabilities

KPGB currently provides a solid foundation for blog writing with the following features:

#### ✅ Content Creation & Management
- **Markdown Support**: Full markdown syntax with frontmatter metadata
- **Rich Metadata**: Title, author, date, tags, categories, excerpts, custom slugs
- **Tag System**: Comprehensive tagging with browsable tag pages
- **Auto-Excerpts**: Intelligent excerpt generation for post previews
- **Multiple Authors**: Support for different authors per post
- **Draft/Published States**: Control post visibility

#### ✅ Publishing & Distribution
- **Multi-Storage Options**: IPFS (decentralized), GitHub, or local storage
- **Static Site Generation**: Generate HTML for GitHub Pages hosting
- **RSS Feed**: Automatic RSS 2.0 feed generation
- **SEO-Friendly URLs**: Clean, readable URL structure
- **Multiple Themes**: 5 beautiful themes to choose from

#### ✅ Reader Experience
- **Full-Text Search**: Fast search with SQLite FTS5
- **Responsive Design**: Mobile-optimized reading experience
- **Archive Pages**: Chronological post browsing
- **Tag Browsing**: Explore posts by topic
- **Pagination**: Navigate through post listings

### Blog Writing Workflow

#### Basic Publishing Flow
```bash
# 1. Create a new post
cargo run -- new --title "My Post Title" --author "Your Name"
# Type your content, press Ctrl+D when done

# 2. Or create from a markdown file
echo "# My Post" > post.md
cargo run -- new --title "My Post" --author "Your Name" --content post.md

# 3. List your posts to get the ID
cargo run -- list

# 4. Publish the post
cargo run -- publish <post-id>

# 5. Generate static site
cargo run -- generate

# 6. Deploy to GitHub Pages
./scripts/deploy.sh
```

#### Writing with Frontmatter
```markdown
---
title: "Advanced Post Example"
author: "Your Name"
date: 2025-07-22
tags: ["rust", "ipfs", "blog"]
category: "Technology"
excerpt: "Custom excerpt for better SEO"
slug: "custom-url-slug"
---

# Your Post Content Here

Write your post using standard markdown...
```

### 🚧 Limitations & Planned Improvements

#### High Priority Improvements Needed

1. **Post Editing** ⚠️
   - Currently no CLI command to edit existing posts
   - Planned: `cargo run -- edit <post-id>` command
   - Workaround: Edit database directly or republish

2. **Delete/Unpublish** ⚠️
   - No built-in delete functionality
   - Planned: `cargo run -- delete <post-id>` command
   - Workaround: Manual database operations

3. **Image Management** ⚠️
   - No built-in image upload/optimization
   - Planned: Automatic IPFS image storage
   - Workaround: Upload images separately to IPFS

4. **Live Preview** 📝
   - No real-time preview while writing
   - Planned: Watch mode with auto-refresh
   - Workaround: Use external markdown editor

5. **SEO Metadata** 🔍
   - Missing meta descriptions, Open Graph tags
   - Planned: Enhanced frontmatter support
   - Workaround: Manually edit templates

#### Medium Priority Enhancements

6. **Navigation Features**
   - Previous/Next post links
   - Related posts suggestions
   - Series/collection support

7. **Custom Pages**
   - About, Contact, Portfolio pages
   - Custom page templates
   - Menu management

8. **Writing Analytics**
   - Word count display
   - Reading time estimation
   - Post performance metrics

9. **Backup System**
   - Automated backups
   - Export/import tools
   - Multi-device sync

10. **Comment System**
    - Decentralized comments
    - Moderation tools
    - Anti-spam measures

### Recommended Workflow Enhancements

Until these features are built-in, here are some workflow recommendations:

#### 1. Create Helper Scripts
```bash
#!/bin/bash
# blog-edit.sh
sqlite3 kpgb.db "SELECT id, title FROM posts ORDER BY created_at DESC LIMIT 10"
echo "Enter post ID to edit:"
read post_id
# Your editing logic here
```

#### 2. Use External Tools
- **Editor**: VS Code with markdown preview
- **Images**: Optimize with ImageOptim before uploading
- **Backup**: Git repository for content versioning
- **Analytics**: Add Google Analytics to templates

#### 3. Template System
Create post templates in a `templates/` directory:
```markdown
<!-- templates/review.md -->
---
title: "Review: {PRODUCT}"
author: "Your Name"
tags: ["review", "{CATEGORY}"]
category: "Reviews"
---

## Overview
{INTRODUCTION}

## Pros
- 

## Cons
- 

## Verdict
{CONCLUSION}
```

### Future Vision for Blog Writers

KPGB aims to become a comprehensive blogging platform that combines:
- **Decentralization**: Your content, your control
- **Convenience**: Modern writing tools and workflows
- **Performance**: Fast, SEO-optimized output
- **Flexibility**: Multiple deployment options

The goal is to make KPGB as convenient as traditional platforms while maintaining the benefits of decentralization.

## Important Reminders

- Always check for duplicate content before storage
- Maintain backward compatibility with existing posts
- Test with local storage before using IPFS
- Keep SQLite index in sync with storage
- Handle IPFS daemon connection errors gracefully
- Use appropriate ports to avoid conflicts
- Remember to set environment variables
- Backup your SQLite database regularly
- Configure base_path for GitHub Pages deployment
- Run clippy before committing code
- Consider reader experience when writing posts
- Use meaningful tags for better organization
- Write descriptive excerpts for better SEO