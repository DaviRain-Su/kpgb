# KPGB - IPFS-Based Decentralized Personal Blog System

A fully decentralized personal blog system where all content is stored on IPFS.

## Quick Start

### Development Server (Local)
```bash
# Use development configuration (no base_path)
./serve-dev.sh
# Or manually:
cargo run -- serve --port 9000 --config site.dev.toml
```

### Production Build (GitHub Pages)
```bash
# Use production configuration (with base_path)
cargo run -- generate --output ./public
./scripts/deploy.sh
```

## Configuration Files

- `site.dev.toml` - Development configuration (no base_path, localhost URLs)
- `site.toml` - Default configuration 
- `site.prod.toml` - Production configuration (GitHub Pages with base_path)

## Features

- **Decentralized Storage**: All content stored on IPFS with unique CIDs
- **Multiple Storage Backends**: Support for IPFS, GitHub, and local storage
- **Content Deduplication**: SHA256-based duplicate detection
- **Fast Search**: SQLite FTS5 full-text search
- **Static Site Generation**: Generate static HTML sites
- **Dynamic Web Interface**: Real-time web UI with search
- **RSS Feed**: Automatic RSS feed generation
- **CLI Management**: Complete command-line interface
- **GitHub Pages Deployment**: Automated deployment with GitHub Actions
- **External Website Integration**: Embeddable widget and API
- **Multiple Themes**: Choose from hacker, minimal, dark, cyberpunk, or default themes

## Installation

```bash
# Clone the repository
git clone <your-repo-url>
cd kpgb

# Build the project
cargo build --release

# Install IPFS (optional but recommended)
# macOS
brew install ipfs
# Start IPFS daemon
ipfs daemon
```

## Configuration

1. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your settings
```

2. Initialize site configuration:
```bash
cargo run -- init
```

## Basic Usage

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

# Start development server
./serve-dev.sh
```

## Web Interface

### Local Development Server
Access at `http://localhost:9000` (NO `/kpgb` prefix):
- Home: `http://localhost:9000/`
- Archive: `http://localhost:9000/archive`
- Search: `http://localhost:9000/search`
- RSS Feed: `http://localhost:9000/feed.xml`
- API: `http://localhost:9000/api/posts`

### GitHub Pages (Production)
Access at `https://username.github.io/kpgb` (WITH `/kpgb` prefix):
- Home: `https://username.github.io/kpgb/`
- Archive: `https://username.github.io/kpgb/archive`
- RSS Feed: `https://username.github.io/kpgb/feed.xml`

## Storage Backends

### IPFS
- Set `IPFS_API_URL` environment variable
- Content is permanently stored with unique CIDs
- Accessible via IPFS gateways

### GitHub
- Set `GITHUB_TOKEN`, `GITHUB_OWNER`, and `GITHUB_REPO`
- Content stored as GitHub repository files

### Local
- Default fallback storage
- Files stored in `./storage/local`

## Deployment

### GitHub Pages
```bash
# Generate static site
cargo run -- generate

# Deploy to GitHub Pages
./scripts/deploy.sh
```

### Self-Hosted Server
```bash
# Run production server
cargo run --release -- serve --port 80
```

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
public/          # Generated static site
migrations/      # Database migrations
scripts/         # Deployment scripts
.github/         # GitHub Actions workflows
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

## Documentation

See [CLAUDE.md](CLAUDE.md) for detailed documentation, architecture, and roadmap.

## License

[Your License]