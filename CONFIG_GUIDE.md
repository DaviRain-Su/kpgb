# Configuration Guide

KPGB supports multiple configuration files for different environments.

## Configuration Files

### `site.toml` - Default Development Config
Used by default for local development and testing.

### `site.dev.toml` - Development Server Config
Used for running the development server locally.
```bash
./serve-dev.sh  # Uses site.dev.toml
# Or explicitly:
cargo run -- serve --config site.dev.toml
```

### `site.prod.toml` - Production Config
Used for GitHub Pages deployment and production builds.
```bash
# Generate with production config
cargo run -- generate --config site.prod.toml

# Deploy to GitHub Pages
./scripts/deploy.sh  # Automatically uses site.prod.toml
```

## Using Different Configurations

### For Static Site Generation
```bash
# Default (uses site.toml)
cargo run -- generate

# With specific config
cargo run -- generate --config site.prod.toml
cargo run -- generate --config my-custom-config.toml
```

### For Web Server
```bash
# Default (uses site.toml)
cargo run -- serve

# With specific config
cargo run -- serve --config site.dev.toml
cargo run -- serve --config site.prod.toml --port 8080
```

### For Preview
```bash
# Default (uses site.toml)
./preview.sh

# With specific config
./preview.sh site.prod.toml
./preview.sh my-theme-test.toml
```

## Configuration Priority

1. **Command line parameter** - Highest priority
   ```bash
   cargo run -- generate --config custom.toml
   ```

2. **Default file** - Used if no parameter specified
   - `generate` command: uses `site.toml`
   - `serve` command: uses `site.toml`

## GitHub Actions

GitHub Actions automatically uses `site.prod.toml` for deployment:
- Reads existing `site.prod.toml` from repository
- Updates `base_url` and `base_path` to match GitHub Pages URL
- Preserves all other settings including theme

## Example Configurations

### Development Config (site.dev.toml)
```toml
title = "My Blog (Dev)"
description = "Development environment"
author = "Developer"
base_url = "http://localhost:9000"
# No base_path for local development
ipfs_gateway = "http://localhost:8080/ipfs/"
posts_per_page = 10
enable_rss = true
theme = "hacker"  # Test different themes locally
```

### Production Config (site.prod.toml)
```toml
title = "My IPFS Blog"
description = "A decentralized blog powered by IPFS"
author = "Your Name"
base_url = "https://username.github.io/kpgb"
base_path = "/kpgb"
ipfs_gateway = "https://ipfs.io/ipfs/"
posts_per_page = 10
enable_rss = true
theme = "dark"  # Production theme
```

### Theme Testing Config
```toml
# Create test-cyberpunk.toml
title = "Theme Test"
description = "Testing cyberpunk theme"
author = "Tester"
base_url = "http://localhost:8888"
ipfs_gateway = "https://ipfs.io/ipfs/"
posts_per_page = 10
enable_rss = true
theme = "cyberpunk"
```

Then test with:
```bash
./preview.sh test-cyberpunk.toml
```

## Best Practices

1. **Keep configs in version control**
   - Commit `site.dev.toml` and `site.prod.toml`
   - Don't commit personal test configs

2. **Use meaningful names**
   - `site.staging.toml` for staging environment
   - `site.local.toml` for local testing
   - `site.theme-NAME.toml` for theme testing

3. **Environment-specific settings**
   - Development: localhost URLs, debug themes
   - Production: proper URLs, professional themes
   - Testing: experimental features

4. **Theme selection**
   - Test themes locally first
   - Use professional themes for production
   - Keep consistent theme across deployments