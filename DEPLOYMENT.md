# KPGB Deployment Guide

This guide explains how to deploy your KPGB static site to GitHub Pages and make it accessible from external websites.

## Prerequisites

- Git repository for your blog
- GitHub account
- KPGB installed and configured

## Deployment Steps

### 1. Prepare Production Configuration

Edit `site.production.toml` and replace `YOUR_USERNAME` with your GitHub username:

```toml
base_url = "https://YOUR_USERNAME.github.io/kpgb"
```

### 2. Deploy to GitHub Pages

Run the deployment script with your GitHub username:

```bash
./deploy-github-pages.sh YOUR_USERNAME
```

This script will:
- Update the production configuration
- Generate the static site with production URLs
- Create/update the `gh-pages` branch
- Push the static files to GitHub

### 3. Enable GitHub Pages

1. Go to your repository settings on GitHub
2. Navigate to "Pages" section
3. Under "Source", select "Deploy from a branch"
4. Choose `gh-pages` branch and `/ (root)` folder
5. Click "Save"

Your site will be available at: `https://YOUR_USERNAME.github.io/kpgb`

## External Access Features

### CORS Headers

The site includes CORS headers that allow external websites to access:
- Blog posts JSON (`/search-index.json`)
- RSS feed (`/feed.xml`)
- All static resources

### Embeddable Widget

External websites can embed your blog posts using the JavaScript widget:

```html
<!-- Basic widget with 5 recent posts -->
<div id="kpgb-widget" data-base-url="https://YOUR_USERNAME.github.io/kpgb"></div>
<script src="https://YOUR_USERNAME.github.io/kpgb/js/widget.js"></script>
```

#### Widget Options

| Attribute | Description | Default |
|-----------|-------------|---------|
| `data-base-url` | Your blog's base URL | Current origin |
| `data-count` | Number of posts to display | 5 |
| `data-theme` | Widget theme ("light" or "dark") | "light" |
| `data-search` | Enable search ("true" or "false") | "false" |
| `data-tags` | Show tags ("true" or "false") | "true" |

#### Widget Examples

```html
<!-- Widget with search enabled -->
<div class="kpgb-widget" 
     data-base-url="https://YOUR_USERNAME.github.io/kpgb" 
     data-search="true"
     data-count="10">
</div>

<!-- Dark theme widget without tags -->
<div class="kpgb-widget" 
     data-base-url="https://YOUR_USERNAME.github.io/kpgb" 
     data-theme="dark"
     data-tags="false">
</div>
```

### RSS Feed

Your blog's RSS feed is available at:
```
https://YOUR_USERNAME.github.io/kpgb/feed.xml
```

### Search API

The search index is available as JSON at:
```
https://YOUR_USERNAME.github.io/kpgb/search-index.json
```

This can be used by external applications to search and display your blog content.

## Testing External Access

Open `widget-demo.html` in a browser to test the widget locally:

```bash
open widget-demo.html
```

Update the `data-base-url` attributes to your production URL for testing.

## Updating Your Blog

To update your deployed blog:

1. Create/update posts using KPGB
2. Run the deployment script again:
   ```bash
   ./deploy-github-pages.sh YOUR_USERNAME
   ```

The script will automatically:
- Generate fresh static files
- Commit changes to `gh-pages` branch
- Push updates to GitHub

## Custom Domain (Optional)

To use a custom domain:

1. Create a `CNAME` file in the `public` directory with your domain
2. Configure DNS settings with your domain provider
3. Update `base_url` in `site.production.toml`

## Troubleshooting

### Widget Not Loading

- Check browser console for CORS errors
- Ensure the base URL is correct
- Verify GitHub Pages is enabled and deployed

### Search Not Working

- Confirm `enable_search = true` in configuration
- Check that `search-index.json` exists in deployed site
- Verify JavaScript is enabled in browser

### RSS Feed Issues

- Ensure `enable_rss = true` in configuration
- Check feed URL: `https://YOUR_USERNAME.github.io/kpgb/feed.xml`
- Validate feed using online RSS validators