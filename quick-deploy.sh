#!/bin/bash

# Quick deployment script for KPGB static site
# This script generates and prepares the static site for deployment

set -e

echo "🚀 KPGB Quick Deploy"
echo "===================="

# Check if production config exists
if [ ! -f "site.production.toml" ]; then
    echo "Creating production configuration..."
    cat > site.production.toml << 'EOL'
title = "My IPFS Blog"
description = "A decentralized blog powered by IPFS"
author = "Your Name"
base_url = "https://YOUR_USERNAME.github.io/kpgb"
posts_per_page = 10
ipfs_gateway = "https://ipfs.io/ipfs/"
enable_rss = true
enable_search = true
EOL
    echo "✅ Created site.production.toml"
    echo "⚠️  Please edit site.production.toml and update YOUR_USERNAME"
    exit 1
fi

# Generate static site
echo "Generating static site..."
cargo run -- generate

# Ensure necessary files exist
echo "Adding deployment files..."

# Create .nojekyll to prevent Jekyll processing
touch public/.nojekyll

# Create index.html if it doesn't exist
if [ ! -f "public/index.html" ]; then
    echo "⚠️  Warning: index.html not found in public directory"
fi

# Show generated files
echo ""
echo "📁 Generated files:"
ls -la public/

echo ""
echo "✅ Static site generated successfully!"
echo ""
echo "🌐 Deployment Options:"
echo ""
echo "1. GitHub Pages:"
echo "   ./deploy-github-pages.sh YOUR_USERNAME"
echo ""
echo "2. Netlify Drop:"
echo "   - Visit https://app.netlify.com/drop"
echo "   - Drag and drop the 'public' folder"
echo ""
echo "3. Vercel:"
echo "   vercel public/"
echo ""
echo "4. Local Preview:"
echo "   cd public && python -m http.server 8000"
echo "   # Visit http://localhost:8000"
echo ""
echo "📦 The 'public' directory contains your complete static site!"