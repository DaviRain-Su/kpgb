# KPGB Theme Gallery

KPGB now includes multiple beautiful themes to customize your blog's appearance. Simply set the `theme` field in your configuration file.

## Available Themes

### 1. Default Theme
```toml
theme = "default"
```
- Classic blog design
- Clean and professional
- Blue accent colors
- Good readability

### 2. Hacker Theme (Terminal/Matrix Style)
```toml
theme = "hacker"
```
- Green text on black background
- Monospace fonts throughout
- Terminal-style cursor animations
- Scan line effects
- ASCII art inspired design
- Perfect for tech/security blogs

### 3. Minimal Theme
```toml
theme = "minimal"
```
- Ultra-clean design
- Focus on typography
- Lots of whitespace
- Subtle animations
- Perfect for writing-focused blogs
- Mobile-optimized

### 4. Dark Theme
```toml
theme = "dark"
```
- Modern dark mode design
- Purple accent colors
- Gradient effects
- Easy on the eyes
- Great for night reading
- Professional appearance

### 5. Cyberpunk Theme
```toml
theme = "cyberpunk"
```
- Neon colors (pink, blue, cyan, yellow)
- Glitch text effects
- Animated scan lines
- Futuristic design
- Bold and eye-catching
- Perfect for gaming/tech blogs

## How to Change Themes

### For Development Server
Edit `site.dev.toml`:
```toml
theme = "hacker"  # Your choice
```

### For Production (GitHub Pages)
Edit `site.toml` or `site.prod.toml`:
```toml
theme = "minimal"  # Your choice
```

### For Static Generation
```bash
# Generate with specific theme
echo 'theme = "dark"' >> site.toml
cargo run -- generate
```

## Theme Features

All themes include:
- Responsive design
- Mobile optimization
- IPFS badge styling
- Code syntax highlighting support
- Archive page styling
- Search page styling
- RSS feed support

## Creating Custom Themes

To create your own theme:

1. Create a new CSS file in `templates/themes/`
2. Use CSS variables for colors:
```css
:root {
    --bg-primary: #ffffff;
    --text-primary: #333333;
    --accent: #0066cc;
    /* etc... */
}
```

3. Add your theme to the handlers:
- Edit `src/web/handlers.rs`
- Edit `src/site/generator.rs`

4. Add your theme name to the match statements

## Theme Preview

To quickly preview different themes:

```bash
# Start server with hacker theme
./serve-dev.sh  # Uses site.dev.toml

# Generate static site with minimal theme
cargo run -- generate  # Uses site.toml

# Test specific theme
echo 'theme = "cyberpunk"' > test-theme.toml
cargo run -- serve --config test-theme.toml
```

## Recommended Theme Usage

- **Hacker**: Perfect for security blogs, CTF writeups, technical tutorials
- **Minimal**: Great for personal blogs, essays, long-form writing
- **Dark**: Ideal for developer blogs, code tutorials, technical content
- **Cyberpunk**: Best for gaming blogs, sci-fi content, creative writing
- **Default**: Good all-around choice for any type of content