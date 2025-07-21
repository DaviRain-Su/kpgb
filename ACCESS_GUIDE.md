# Access Guide for KPGB

## Development Server (Local)

When running the development server with `./serve-dev.sh` or `cargo run -- serve`:

**Base URL**: `http://localhost:3000` (or your chosen port)

**Routes**:
- Home: `http://localhost:3000/`
- Archive: `http://localhost:3000/archive`
- RSS Feed: `http://localhost:3000/feed.xml`
- Search: `http://localhost:3000/search`
- Posts: `http://localhost:3000/posts/[slug]`
- API: `http://localhost:3000/api/posts`

**NO `/kpgb` prefix needed for local development!**

## Static Site (GitHub Pages)

When deployed to GitHub Pages:

**Base URL**: `https://username.github.io/kpgb`

**Routes**:
- Home: `https://username.github.io/kpgb/`
- Archive: `https://username.github.io/kpgb/archive`
- RSS Feed: `https://username.github.io/kpgb/feed.xml`
- Posts: `https://username.github.io/kpgb/posts/[slug].html`

**The `/kpgb` prefix is automatically added by GitHub Pages**

## Configuration Files

### For Local Development (`site.dev.toml`)
```toml
base_url = "http://localhost:9000"
# NO base_path - leave it empty for local development
```

### For GitHub Pages (`site.toml` or `site.prod.toml`)
```toml
base_url = "https://username.github.io/kpgb"
base_path = "/kpgb"
```

## Common Mistakes

❌ Wrong: `http://localhost:3000/kpgb/feed.xml`
✅ Correct: `http://localhost:3000/feed.xml`

❌ Wrong: `https://username.github.io/archive.html`
✅ Correct: `https://username.github.io/kpgb/archive`

## Quick Test Commands

### Test Local Server
```bash
# Start server
./serve-dev.sh

# Test routes
curl http://localhost:9000/
curl http://localhost:9000/archive
curl http://localhost:9000/feed.xml
```

### Test Static Site Locally
```bash
# Generate site
cargo run -- generate

# Serve with Python
cd public
python -m http.server 8000

# Access at: http://localhost:8000/
# Note: Some links may not work correctly without the /kpgb prefix
```