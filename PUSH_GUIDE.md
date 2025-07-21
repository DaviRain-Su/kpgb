# Push to GitHub Guide

## Current Status
All changes have been committed locally. You need to push to GitHub to trigger the deployment.

## Option 1: Using Personal Access Token
```bash
# Set your GitHub token
export GITHUB_TOKEN="your-personal-access-token"

# Push using token
git push https://${GITHUB_TOKEN}@github.com/DaviRain-Su/kpgb.git master
```

## Option 2: Using SSH
```bash
# Change remote to SSH
git remote set-url origin git@github.com:DaviRain-Su/kpgb.git

# Push
git push origin master
```

## Option 3: Using GitHub CLI
```bash
# Install GitHub CLI (if not installed)
brew install gh

# Authenticate
gh auth login

# Push
git push origin master
```

## What Will Happen After Push

1. GitHub Actions will automatically run the deploy workflow
2. It will use `site.prod.toml` with the dark theme
3. The site will be deployed to https://DaviRain-Su.github.io/kpgb/
4. You should see the dark theme applied in about 2-5 minutes

## Verify Deployment

After pushing, you can check:
1. GitHub Actions tab in your repository
2. Look for "Deploy to GitHub Pages" workflow
3. Wait for it to complete (green checkmark)
4. Visit https://DaviRain-Su.github.io/kpgb/

## Recent Changes Made

1. **Fixed site.prod.toml**:
   - Updated author to "DaviRain-Su"
   - Updated base_url to "https://DaviRain-Su.github.io/kpgb"
   - Theme is set to "dark"

2. **Updated GitHub Actions**:
   - Now uses `site.prod.toml` directly
   - No longer creates temporary config files
   - Preserves theme selection

3. **Added --config support**:
   - `cargo run -- generate --config site.prod.toml`
   - Allows using different configs without file copying