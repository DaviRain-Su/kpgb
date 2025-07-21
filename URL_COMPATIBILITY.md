# URL Compatibility Between Static and Dynamic Sites

## Problem
The static site generator and dynamic web server were using different URL patterns:
- Static: `/archive.html`, `/posts/slug.html`
- Dynamic: `/archive`, `/posts/slug`

## Solution
Implemented clean URLs for both static and dynamic sites with backward compatibility.

### Static Site Generation
1. **Archive**: 
   - Primary: `/archive/index.html` (accessed as `/archive`)
   - Compatibility: `/archive.html` (duplicate file for old links)

2. **Posts**: 
   - Keep as `/posts/slug.html` (both systems support this)

3. **RSS Feed**: 
   - Keep as `/feed.xml` (standard location)

### Dynamic Web Server
1. **Routes**:
   - `/archive` - Main archive route
   - `/archive.html` - Redirects to `/archive` (301 permanent)
   - `/posts/:slug` - Post routes (works with or without .html)
   - `/feed.xml` - RSS feed

### Benefits
- **Consistent URLs**: Both static and dynamic sites use same paths
- **Clean URLs**: No `.html` extension in navigation
- **SEO Friendly**: Permanent redirects preserve link equity
- **Backward Compatible**: Old links continue to work

### Implementation Details

#### Static Generator (generator.rs)
```rust
// Create archive directory for clean URLs
let archive_dir = self.output_dir.join("archive");
fs::create_dir_all(&archive_dir)?;
let output_path = archive_dir.join("index.html");
fs::write(&output_path, &rendered)?;

// Also create archive.html for backward compatibility
let archive_html_path = self.output_dir.join("archive.html");
fs::write(archive_html_path, rendered)?;
```

#### Web Server (handlers.rs)
```rust
// Redirect handler for backward compatibility
pub async fn redirect_archive() -> impl IntoResponse {
    Redirect::permanent("/archive")
}
```

#### Templates (base.html)
```html
<li><a href="{{ site.base_path | default(value="") }}/archive">Archive</a></li>
```

### Testing
1. Static site: `cargo run -- generate && cd public && python -m http.server`
2. Dynamic site: `./serve-dev.sh`
3. Both should have working `/archive` and `/archive.html` URLs

### Future Considerations
- Could extend this pattern to posts: `/posts/slug/index.html`
- Could add more redirects as needed
- Consider implementing trailing slash handling