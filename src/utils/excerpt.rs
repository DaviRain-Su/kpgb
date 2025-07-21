use pulldown_cmark::{Event, Parser, Tag, TagEnd};

/// Generate an excerpt from markdown content
/// 
/// This function:
/// 1. Strips markdown formatting
/// 2. Extracts first paragraph or up to word limit
/// 3. Adds ellipsis if truncated
pub fn generate_excerpt(markdown: &str, word_limit: usize) -> String {
    let mut plain_text = String::new();
    let mut word_count = 0;
    let mut in_code_block = false;
    let mut in_heading = false;
    
    let parser = Parser::new(markdown);
    
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::CodeBlock(_) => in_code_block = true,
                Tag::Heading { .. } => in_heading = true,
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::CodeBlock => in_code_block = false,
                TagEnd::Heading(_) => {
                    in_heading = false;
                    // Add space after heading
                    if !plain_text.is_empty() && !plain_text.ends_with(' ') {
                        plain_text.push(' ');
                    }
                }
                TagEnd::Paragraph => {
                    // If we have enough content after first paragraph, stop
                    if word_count >= word_limit / 2 {
                        break;
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if !in_code_block && !in_heading {
                    let words: Vec<&str> = text.split_whitespace().collect();
                    for word in words {
                        if word_count >= word_limit {
                            plain_text.push_str("...");
                            return clean_excerpt(&plain_text);
                        }
                        if !plain_text.is_empty() && !plain_text.ends_with(' ') {
                            plain_text.push(' ');
                        }
                        plain_text.push_str(word);
                        word_count += 1;
                    }
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if !in_code_block && !plain_text.ends_with(' ') {
                    plain_text.push(' ');
                }
            }
            _ => {}
        }
    }
    
    clean_excerpt(&plain_text)
}

/// Clean up the excerpt text
fn clean_excerpt(text: &str) -> String {
    let mut result = text.trim().to_string();
    
    // Remove multiple consecutive spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    
    // Ensure it ends with proper punctuation or ellipsis
    if !result.is_empty() && !result.ends_with('.') && !result.ends_with('!') 
        && !result.ends_with('?') && !result.ends_with("...") {
        result.push_str("...");
    }
    
    result
}

/// Generate excerpt preserving some markdown formatting for better display
pub fn generate_formatted_excerpt(markdown: &str, char_limit: usize) -> String {
    let mut result = String::new();
    let mut char_count = 0;
    let mut in_code_block = false;
    let mut list_level = 0;
    
    let parser = Parser::new(markdown);
    
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    if !result.is_empty() {
                        result.push_str("\n\n");
                        char_count += 2;
                    }
                }
                Tag::Heading { .. } => {
                    if !result.is_empty() {
                        result.push_str("\n\n");
                        char_count += 2;
                    }
                    // Skip heading in excerpt
                    in_code_block = true;
                }
                Tag::List(_) => {
                    list_level += 1;
                    if !result.is_empty() {
                        result.push('\n');
                        char_count += 1;
                    }
                }
                Tag::Item => {
                    result.push_str("- ");
                    char_count += 2;
                }
                Tag::CodeBlock(_) => {
                    in_code_block = true;
                }
                Tag::Emphasis => {
                    result.push('*');
                    char_count += 1;
                }
                Tag::Strong => {
                    result.push_str("**");
                    char_count += 2;
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => {
                    in_code_block = false;
                }
                TagEnd::List(_) => {
                    list_level -= 1;
                }
                TagEnd::Item => {
                    result.push('\n');
                    char_count += 1;
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                }
                TagEnd::Emphasis => {
                    result.push('*');
                    char_count += 1;
                }
                TagEnd::Strong => {
                    result.push_str("**");
                    char_count += 2;
                }
                TagEnd::Paragraph => {
                    // Stop after first paragraph if we have enough content
                    if char_count >= char_limit / 2 {
                        break;
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if !in_code_block {
                    let remaining = char_limit.saturating_sub(char_count);
                    if remaining == 0 {
                        result.push_str("...");
                        break;
                    }
                    
                    if text.len() <= remaining {
                        result.push_str(&text);
                        char_count += text.len();
                    } else {
                        // Truncate at word boundary
                        let truncated = truncate_at_word_boundary(&text, remaining);
                        result.push_str(&truncated);
                        result.push_str("...");
                        break;
                    }
                }
            }
            Event::Code(code) => {
                if !in_code_block {
                    let code_with_backticks = format!("`{}`", code);
                    let remaining = char_limit.saturating_sub(char_count);
                    if code_with_backticks.len() <= remaining {
                        result.push_str(&code_with_backticks);
                        char_count += code_with_backticks.len();
                    }
                }
            }
            _ => {}
        }
        
        if char_count >= char_limit {
            if !result.ends_with("...") {
                result.push_str("...");
            }
            break;
        }
    }
    
    result.trim().to_string()
}

/// Truncate text at word boundary
fn truncate_at_word_boundary(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }
    
    // Work with char indices to avoid UTF-8 boundary issues
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_len {
        return text.to_string();
    }
    
    // Find last space before limit
    let mut boundary = max_len.min(chars.len());
    while boundary > 0 && !chars[boundary - 1].is_whitespace() {
        boundary -= 1;
    }
    
    // If no space found, just truncate at character boundary
    if boundary == 0 {
        boundary = max_len.min(chars.len());
    }
    
    chars[..boundary].iter().collect::<String>().trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_excerpt() {
        let markdown = "# Test Post\n\nThis is the first paragraph with some **bold** text.\n\nThis is the second paragraph.";
        let excerpt = generate_excerpt(markdown, 10);
        assert!(excerpt.starts_with("This is the first paragraph"));
        assert!(excerpt.ends_with("..."));
    }

    #[test]
    fn test_generate_formatted_excerpt() {
        let markdown = "# Test Post\n\nThis is a paragraph with *emphasis* and **bold** text.\n\n- List item 1\n- List item 2";
        let excerpt = generate_formatted_excerpt(markdown, 100);
        assert!(excerpt.contains("*emphasis*"));
        assert!(excerpt.contains("**bold**"));
        assert!(excerpt.contains("- List item"));
    }

    #[test]
    fn test_code_block_skipping() {
        let markdown = "Some text\n\n```rust\nfn main() {}\n```\n\nMore text here.";
        let excerpt = generate_excerpt(markdown, 10);
        assert!(!excerpt.contains("fn main"));
        assert!(excerpt.contains("More text"));
    }
}