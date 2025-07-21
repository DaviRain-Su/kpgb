#!/bin/bash
# Deploy script for GitHub Pages

set -e

echo "🚀 Deploying to GitHub Pages..."

# Check if site.prod.toml exists
if [ ! -f site.prod.toml ]; then
    echo "❌ Error: site.prod.toml not found!"
    echo "Please create site.prod.toml with your production configuration."
    exit 1
fi

echo "📦 Building project..."
cargo build --release

echo "🎨 Generating static site with production config..."
echo "Theme: $(grep '^theme' site.prod.toml | cut -d'"' -f2)"
cargo run -- generate --config site.prod.toml

# Create .nojekyll to prevent Jekyll processing
touch public/.nojekyll

# Initialize git in public directory
cd public

# Check if gh-pages branch exists
if git ls-remote --heads origin gh-pages | grep -q gh-pages; then
    echo "📝 Using existing gh-pages branch"
else
    echo "📝 Creating gh-pages branch"
fi

git init
git add -A
git commit -m "Deploy to GitHub Pages - $(date)"

# Get the repository URL
REPO_URL=$(git -C .. remote get-url origin)

# Push to gh-pages branch
git push -f "$REPO_URL" HEAD:gh-pages

echo "✅ Deployment complete!"
echo "🌐 Your site will be available at: https://$(git -C .. remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/' | tr '/' '.').github.io/$(basename $(git -C .. remote get-url origin) .git)/"

cd ..