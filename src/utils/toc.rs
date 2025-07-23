use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    pub level: u8,
    pub text: String,
    pub id: String,
    pub children: Vec<TocEntry>,
}

/// Generate a table of contents from markdown content
pub fn generate_toc(markdown: &str) -> Vec<TocEntry> {
    let parser = Parser::new(markdown);
    let mut toc_entries = Vec::new();
    let mut current_heading = String::new();
    let mut in_heading = false;
    let mut heading_level = 1;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                current_heading.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if in_heading && !current_heading.is_empty() {
                    let id = generate_heading_id(&current_heading);
                    toc_entries.push(TocEntry {
                        level: heading_level,
                        text: current_heading.clone(),
                        id,
                        children: Vec::new(),
                    });
                }
                in_heading = false;
            }
            Event::Text(text) if in_heading => {
                current_heading.push_str(&text);
            }
            Event::Code(code) if in_heading => {
                current_heading.push_str(&format!("`{}`", code));
            }
            _ => {}
        }
    }

    // Build hierarchical structure
    build_toc_tree(toc_entries)
}

/// Generate a URL-safe ID from heading text
pub fn generate_heading_id(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Build a hierarchical tree structure from flat TOC entries
fn build_toc_tree(entries: Vec<TocEntry>) -> Vec<TocEntry> {
    let mut result = Vec::new();
    let mut stack: Vec<TocEntry> = Vec::new();

    for entry in entries {
        // Skip H1 headers as they're usually the post title
        if entry.level == 1 {
            continue;
        }

        // Find the right parent
        while let Some(last) = stack.last() {
            if last.level < entry.level {
                break;
            }
            if let Some(completed) = stack.pop() {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(completed);
                } else {
                    result.push(completed);
                }
            }
        }

        stack.push(entry);
    }

    // Clean up remaining items in stack
    while let Some(completed) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(completed);
        } else {
            result.push(completed);
        }
    }

    result
}

/// Generate HTML for the table of contents
pub fn generate_toc_html(toc: &[TocEntry]) -> String {
    if toc.is_empty() {
        return String::new();
    }

    let mut html = String::from(
        r##"<nav class="toc" id="toc">
    <h2 class="toc-title">Table of Contents</h2>
    <ul class="toc-list">"##,
    );

    generate_toc_list_html(toc, &mut html);

    html.push_str("</ul></nav>");
    html
}

fn generate_toc_list_html(entries: &[TocEntry], html: &mut String) {
    for entry in entries {
        html.push_str(&format!(
            r##"<li class="toc-item toc-level-{}">
                <a href="#{}" class="toc-link">{}</a>"##,
            entry.level,
            entry.id,
            escape_html(&entry.text)
        ));

        if !entry.children.is_empty() {
            html.push_str(r##"<ul class="toc-sublist">"##);
            generate_toc_list_html(&entry.children, html);
            html.push_str("</ul>");
        }

        html.push_str("</li>");
    }
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_heading_id() {
        assert_eq!(generate_heading_id("Hello World"), "hello-world");
        assert_eq!(generate_heading_id("Rust & Go"), "rust-_-go");
        assert_eq!(
            generate_heading_id("  Multiple   Spaces  "),
            "multiple-spaces"
        );
        assert_eq!(generate_heading_id("中文标题"), "_____");
    }

    #[test]
    fn test_generate_toc() {
        let markdown = r#"
# Main Title

## Section 1
Some content here.

### Subsection 1.1
More content.

### Subsection 1.2
Even more content.

## Section 2
Another section.
"#;

        let toc = generate_toc(markdown);
        assert_eq!(toc.len(), 2); // H1 is skipped
        assert_eq!(toc[0].text, "Section 1");
        assert_eq!(toc[0].level, 2);
        assert_eq!(toc[0].children.len(), 2);
        assert_eq!(toc[0].children[0].text, "Subsection 1.1");
        assert_eq!(toc[1].text, "Section 2");
    }
}
