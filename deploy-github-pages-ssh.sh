#!/bin/bash

# Deploy KPGB static site to GitHub Pages using SSH
# This version uses SSH instead of HTTPS to avoid authentication issues
# Usage: ./deploy-github-pages-ssh.sh [github-username]

set -e

echo "🚀 KPGB GitHub Pages Deployment Script (SSH Version)"
echo "===================================================="

# Get GitHub username
GITHUB_USERNAME=${1:-""}
if [ -z "$GITHUB_USERNAME" ]; then
    echo "Please provide your GitHub username as an argument"
    echo "Usage: ./deploy-github-pages-ssh.sh YOUR_USERNAME"
    exit 1
fi

# Check if remote is using SSH
REMOTE_URL=$(git remote get-url origin 2>/dev/null || echo "")
if [[ ! "$REMOTE_URL" =~ ^git@github.com ]]; then
    echo "Converting remote to SSH..."
    git remote set-url origin git@github.com:${GITHUB_USERNAME}/kpgb.git
    echo "✅ Remote converted to SSH"
fi

# Test SSH connection
echo "Testing SSH connection to GitHub..."
ssh -T git@github.com 2>&1 || true

# Check if gh-pages branch exists
if ! git show-ref --verify --quiet refs/heads/gh-pages; then
    echo "Creating gh-pages branch..."
    git checkout --orphan gh-pages
    git rm -rf . || true
    git commit --allow-empty -m "Initial gh-pages commit"
    git checkout master || git checkout main
fi

# Update production config with GitHub username
echo "Updating production configuration..."
if [ -f "site.production.toml" ]; then
    sed -i.bak "s/YOUR_USERNAME/$GITHUB_USERNAME/g" site.production.toml
    rm -f site.production.toml.bak
else
    # Create production config if it doesn't exist
    cat > site.production.toml << EOL
title = "My IPFS Blog"
description = "A decentralized blog powered by IPFS"
author = "Your Name"
base_url = "https://${GITHUB_USERNAME}.github.io/kpgb"
posts_per_page = 10
ipfs_gateway = "https://ipfs.io/ipfs/"
enable_rss = true
enable_search = true
EOL
fi

# Build the site with production config
echo "Building static site with production configuration..."
# First, backup current config
cp site.toml site.toml.backup 2>/dev/null || true
# Use production config
cp site.production.toml site.toml

# Run the static site generator
cargo run -- generate

# Restore original config
mv site.toml.backup site.toml 2>/dev/null || true

# Create .nojekyll file to prevent Jekyll processing
touch public/.nojekyll

# Copy to temporary directory
TEMP_DIR=$(mktemp -d)
cp -r public/* "$TEMP_DIR/"

# Get current branch name
CURRENT_BRANCH=$(git branch --show-current)

# Switch to gh-pages branch
echo "Switching to gh-pages branch..."
git checkout gh-pages

# Clear existing content
rm -rf * .github .gitignore .env* || true

# Copy new content
cp -r "$TEMP_DIR"/* .
cp "$TEMP_DIR"/.nojekyll . 2>/dev/null || true

# Add all files
git add -A

# Commit changes
COMMIT_MSG="Deploy static site - $(date '+%Y-%m-%d %H:%M:%S')"
git commit -m "$COMMIT_MSG" || echo "No changes to commit"

# Push to GitHub
echo "Pushing to GitHub Pages..."
git push origin gh-pages

# Switch back to original branch
git checkout "$CURRENT_BRANCH"

# Clean up
rm -rf "$TEMP_DIR"

echo "✅ Deployment complete!"
echo "Your site will be available at: https://$GITHUB_USERNAME.github.io/kpgb"
echo ""
echo "⚠️  Important: Make sure to enable GitHub Pages in your repository settings:"
echo "1. Go to: https://github.com/$GITHUB_USERNAME/kpgb/settings/pages"
echo "2. Source: Deploy from a branch"
echo "3. Branch: gh-pages"
echo "4. Folder: / (root)"
echo ""
echo "Note: It may take a few minutes for GitHub Pages to update."