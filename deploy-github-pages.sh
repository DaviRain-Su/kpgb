#!/bin/bash

# Deploy KPGB static site to GitHub Pages
# Usage: ./deploy-github-pages.sh [github-username]

set -e

echo "🚀 KPGB GitHub Pages Deployment Script"
echo "======================================"

# Check if gh-pages branch exists
if ! git show-ref --verify --quiet refs/heads/gh-pages; then
    echo "Creating gh-pages branch..."
    git checkout --orphan gh-pages
    git rm -rf .
    git commit --allow-empty -m "Initial gh-pages commit"
    git checkout master
fi

# Get GitHub username
GITHUB_USERNAME=${1:-""}
if [ -z "$GITHUB_USERNAME" ]; then
    echo "Please provide your GitHub username as an argument"
    echo "Usage: ./deploy-github-pages.sh YOUR_USERNAME"
    exit 1
fi

# Update production config with GitHub username
echo "Updating production configuration..."
sed -i.bak "s/YOUR_USERNAME/$GITHUB_USERNAME/g" site.production.toml
rm -f site.production.toml.bak

# Build the site with production config
echo "Building static site with production configuration..."
# First, backup current config
cp site.toml site.toml.backup
# Use production config
cp site.production.toml site.toml

# Run the static site generator
cargo run -- generate

# Restore original config
mv site.toml.backup site.toml

# Create .nojekyll file to prevent Jekyll processing
touch public/.nojekyll

# Add CNAME file if custom domain is desired (commented out by default)
# echo "your-custom-domain.com" > public/CNAME

# Copy to temporary directory
TEMP_DIR=$(mktemp -d)
cp -r public/* "$TEMP_DIR/"

# Switch to gh-pages branch
echo "Switching to gh-pages branch..."
git checkout gh-pages

# Clear existing content
rm -rf *

# Copy new content
cp -r "$TEMP_DIR"/* .

# Add all files
git add .

# Commit changes
COMMIT_MSG="Deploy static site - $(date '+%Y-%m-%d %H:%M:%S')"
git commit -m "$COMMIT_MSG" || echo "No changes to commit"

# Push to GitHub
echo "Pushing to GitHub Pages..."
git push origin gh-pages

# Switch back to master branch
git checkout master

# Clean up
rm -rf "$TEMP_DIR"

echo "✅ Deployment complete!"
echo "Your site will be available at: https://$GITHUB_USERNAME.github.io/kpgb"
echo "Note: It may take a few minutes for GitHub Pages to update."