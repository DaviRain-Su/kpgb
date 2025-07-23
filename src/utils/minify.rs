use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// Minification configuration
#[derive(Debug, Clone)]
pub struct MinifyConfig {
    /// Enable HTML minification
    pub minify_html: bool,
    /// Enable CSS minification
    pub minify_css: bool,
    /// Enable JavaScript minification
    pub minify_js: bool,
    /// Preserve comments in minified output
    pub preserve_comments: bool,
}

impl Default for MinifyConfig {
    fn default() -> Self {
        Self {
            minify_html: true,
            minify_css: true,
            minify_js: true,
            preserve_comments: false,
        }
    }
}

/// Statistics from minification
#[derive(Debug, Default)]
pub struct MinifyStats {
    pub files_processed: usize,
    pub bytes_saved: i64,
    pub errors: usize,
}

impl MinifyStats {
    pub fn summary(&self) -> String {
        format!(
            "Minified {} files, saved {} bytes ({:.2} MB), {} errors",
            self.files_processed,
            self.bytes_saved,
            self.bytes_saved as f64 / 1_048_576.0,
            self.errors
        )
    }
}

/// Minify all HTML/CSS/JS files in a directory recursively
pub async fn minify_directory(dir: &Path, config: &MinifyConfig) -> Result<MinifyStats> {
    let mut stats = MinifyStats::default();
    minify_directory_recursive(dir, config, &mut stats)?;
    Ok(stats)
}

fn minify_directory_recursive(
    dir: &Path,
    config: &MinifyConfig,
    stats: &mut MinifyStats,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            minify_directory_recursive(&path, config, stats)?;
        } else {
            let should_minify = match path.extension().and_then(|s| s.to_str()) {
                Some("html") => config.minify_html,
                Some("css") => config.minify_css,
                Some("js") => config.minify_js,
                _ => false,
            };

            if should_minify {
                match minify_file(&path, config) {
                    Ok(saved) => {
                        stats.files_processed += 1;
                        stats.bytes_saved += saved;
                        if saved > 0 {
                            info!("Minified: {} (saved {} bytes)", path.display(), saved);
                        }
                    }
                    Err(e) => {
                        stats.errors += 1;
                        warn!("Failed to minify {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Minify a single file
fn minify_file(path: &Path, config: &MinifyConfig) -> Result<i64> {
    let content = fs::read_to_string(path)?;
    let original_size = content.len() as i64;

    let minified = match path.extension().and_then(|s| s.to_str()) {
        Some("html") => minify_html(&content, config)?,
        Some("css") => minify_css(&content, config)?,
        Some("js") => minify_js(&content, config)?,
        _ => return Ok(0),
    };

    let new_size = minified.len() as i64;
    let saved = original_size - new_size;

    if saved > 0 {
        fs::write(path, minified)?;
    }

    Ok(saved)
}

/// Minify HTML content
fn minify_html(html: &str, config: &MinifyConfig) -> Result<String> {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut last_char = ' ';
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                in_tag = true;
                result.push(ch);

                // Check for script or style tags
                let tag_preview: String = chars.clone().take(6).collect();
                if tag_preview.starts_with("script") {
                    in_script = true;
                } else if tag_preview.starts_with("style") {
                    in_style = true;
                } else if tag_preview.starts_with("/script") {
                    in_script = false;
                } else if tag_preview.starts_with("/style") {
                    in_style = false;
                }
            }
            '>' => {
                in_tag = false;
                result.push(ch);
            }
            ' ' | '\n' | '\r' | '\t' => {
                if in_tag || in_script || in_style {
                    result.push(ch);
                } else if !last_char.is_whitespace() {
                    result.push(' ');
                }
            }
            _ => {
                result.push(ch);
            }
        }
        last_char = ch;
    }

    // Remove HTML comments unless preserving
    if !config.preserve_comments {
        result = remove_html_comments(&result);
    }

    Ok(result)
}

/// Remove HTML comments from content
fn remove_html_comments(html: &str) -> String {
    let comment_regex = regex::Regex::new(r"<!--.*?-->").unwrap();
    comment_regex.replace_all(html, "").to_string()
}

/// Minify CSS content
fn minify_css(css: &str, config: &MinifyConfig) -> Result<String> {
    let mut result = String::with_capacity(css.len());
    let mut in_string = false;
    let mut string_char = '"';
    let mut last_char = ' ';
    let mut chars = css.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' | '\'' => {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char && last_char != '\\' {
                    in_string = false;
                }
                result.push(ch);
            }
            ' ' | '\n' | '\r' | '\t' => {
                if in_string {
                    result.push(ch);
                } else if !last_char.is_whitespace()
                    && !matches!(last_char, '{' | '}' | ';' | ':' | ',')
                    && !matches!(
                        chars.peek(),
                        Some('{') | Some('}') | Some(';') | Some(':') | Some(',')
                    )
                {
                    result.push(' ');
                }
            }
            _ => {
                result.push(ch);
            }
        }
        last_char = ch;
    }

    // Remove CSS comments unless preserving
    if !config.preserve_comments {
        result = remove_css_comments(&result);
    }

    Ok(result)
}

/// Remove CSS comments from content
fn remove_css_comments(css: &str) -> String {
    let comment_regex = regex::Regex::new(r"/\*.*?\*/").unwrap();
    comment_regex.replace_all(css, "").to_string()
}

/// Minify JavaScript content (basic minification)
fn minify_js(js: &str, config: &MinifyConfig) -> Result<String> {
    // Basic JS minification - remove unnecessary whitespace
    // For production, consider using a proper JS minifier
    let mut result = String::with_capacity(js.len());
    let mut in_string = false;
    let mut in_regex = false;
    let mut string_char = '"';
    let mut last_char = ' ';
    let mut chars = js.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' | '\'' | '`' => {
                if !in_string && !in_regex {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char && last_char != '\\' {
                    in_string = false;
                }
                result.push(ch);
            }
            '/' => {
                if !in_string {
                    if chars.peek() == Some(&'/') {
                        // Single-line comment - skip to end of line
                        if config.preserve_comments {
                            result.push(ch);
                            for c in chars.by_ref() {
                                result.push(c);
                                if c == '\n' {
                                    break;
                                }
                            }
                        } else {
                            while let Some(c) = chars.next() {
                                if c == '\n' {
                                    result.push('\n');
                                    break;
                                }
                            }
                        }
                        continue;
                    } else if chars.peek() == Some(&'*') {
                        // Multi-line comment
                        if config.preserve_comments {
                            result.push(ch);
                            result.push(chars.next().unwrap()); // consume '*'
                            let mut prev = ' ';
                            for c in chars.by_ref() {
                                result.push(c);
                                if prev == '*' && c == '/' {
                                    break;
                                }
                                prev = c;
                            }
                        } else {
                            chars.next(); // consume '*'
                            let mut prev = ' ';
                            for c in chars.by_ref() {
                                if prev == '*' && c == '/' {
                                    break;
                                }
                                prev = c;
                            }
                        }
                        continue;
                    } else if matches!(
                        last_char,
                        '=' | '('
                            | '['
                            | ','
                            | ':'
                            | ';'
                            | '!'
                            | '&'
                            | '|'
                            | '?'
                            | '+'
                            | '-'
                            | '*'
                            | '/'
                            | '%'
                            | '^'
                            | '~'
                    ) {
                        // Likely a regex
                        in_regex = true;
                    }
                }
                result.push(ch);
            }
            ' ' | '\n' | '\r' | '\t' => {
                if in_string || in_regex {
                    result.push(ch);
                } else if !last_char.is_whitespace()
                    && !is_js_operator(last_char)
                    && chars.peek().map_or(false, |&c| !is_js_operator(c))
                {
                    result.push(' ');
                }
            }
            _ => {
                if in_regex && ch == '/' && last_char != '\\' {
                    in_regex = false;
                }
                result.push(ch);
            }
        }
        last_char = ch;
    }

    Ok(result)
}

/// Check if character is a JavaScript operator or punctuation
fn is_js_operator(ch: char) -> bool {
    matches!(
        ch,
        '{' | '}'
            | '('
            | ')'
            | '['
            | ']'
            | ';'
            | ','
            | '.'
            | ':'
            | '='
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '!'
            | '?'
            | '<'
            | '>'
            | '&'
            | '|'
            | '^'
            | '~'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_html() {
        let config = MinifyConfig::default();
        let html = r#"
        <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <h1>Hello World</h1>
                <!-- This is a comment -->
                <p>This is   a   test</p>
            </body>
        </html>
        "#;

        let minified = minify_html(html, &config).unwrap();
        assert!(!minified.contains("<!--"));
        assert!(!minified.contains("   "));
        assert!(minified.contains("<h1>Hello World</h1>"));
    }

    #[test]
    fn test_minify_css() {
        let config = MinifyConfig::default();
        let css = r#"
        body {
            margin: 0;
            padding: 0;
        }
        
        /* This is a comment */
        .test {
            color: red;
        }
        "#;

        let minified = minify_css(css, &config).unwrap();
        assert!(!minified.contains("/*"));
        assert!(minified.contains("body{margin:0;padding:0;}"));
    }

    #[test]
    fn test_minify_js() {
        let config = MinifyConfig::default();
        let js = r#"
        function test() {
            // This is a comment
            var x = 1;
            var y = 2;
            return x + y;
        }
        "#;

        let minified = minify_js(js, &config).unwrap();
        assert!(!minified.contains("// This is a comment"));
        assert!(minified.contains("var x=1;"));
    }
}
