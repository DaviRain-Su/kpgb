# KPGB - Kaspa-Powered Genesis Blog

A decentralized personal blog system with content stored on IPFS and local indexing for performance.

## Features

- **Decentralized Storage**: All content stored on IPFS with unique CIDs
- **Multiple Storage Backends**: Support for IPFS, GitHub, and local storage
- **Content Deduplication**: SHA256-based duplicate detection
- **Fast Search**: SQLite FTS5 full-text search
- **Static Site Generation**: Generate static HTML sites
- **Dynamic Web Interface**: Real-time web UI with search
- **RSS Feed**: Automatic RSS feed generation
- **CLI Management**: Complete command-line interface

## Quick Start

### Installation

```bash
# Clone the repository
git clone <your-repo-url>
cd kpgb

# Build the project
cargo build --release
```

### Configuration

1. Set up IPFS (optional):
```bash
export IPFS_API_URL=http://localhost:5001
```

2. Initialize site configuration:
```bash
cargo run -- init
```

### Basic Usage

```bash
# Create a new post
cargo run -- new --title "My First Post" --author "Your Name"

# List all posts
cargo run -- list

# Publish a post
cargo run -- publish <storage-id>

# Search posts
cargo run -- search "keyword"

# Generate static site
cargo run -- generate

# Start web server
cargo run -- serve --port 3001
```

## Web Interface

Access the web interface at `http://localhost:3001`:
- Home: `/`
- Search: `/search`
- Archive: `/archive`
- API: `/api/posts`

## Storage Backends

### IPFS
- Set `IPFS_API_URL` environment variable
- Content is permanently stored with unique CIDs
- Accessible via IPFS gateways

### GitHub
- Set `GITHUB_TOKEN`, `GITHUB_OWNER`, and `GITHUB_REPO`
- Content stored as GitHub Gist or repository files

### Local
- Default fallback storage
- Files stored in `./storage/local`

## Architecture

```
src/
├── blog/        # Blog management logic
├── database/    # SQLite indexing
├── models/      # Data models
├── site/        # Static site generator
├── storage/     # Storage backends
└── web/         # Web server and API

templates/       # HTML templates
static/          # CSS and assets
migrations/      # Database migrations
```

## API Endpoints

- `GET /api/posts` - List all published posts
- `GET /api/posts/:id` - Get specific post
- `POST /api/search` - Search posts

## Development

```bash
# Run tests
cargo test

# Check code
cargo clippy

# Format code
cargo fmt

# Watch for changes
cargo watch -x run
```

## License

[Your License]