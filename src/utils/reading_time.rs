use pulldown_cmark::{Event, Parser, Tag, TagEnd};

/// Reading speed constants
pub const WORDS_PER_MINUTE: usize = 200; // Average adult reading speed
pub const WORDS_PER_MINUTE_TECHNICAL: usize = 150; // Technical content is slower
pub const CHINESE_CHARS_PER_MINUTE: usize = 300; // Chinese reading speed

/// Calculate estimated reading time for content
pub fn calculate_reading_time(content: &str, is_technical: bool) -> ReadingTime {
    let stats = analyze_content(content);

    // Choose reading speed based on content type
    let wpm = if is_technical {
        WORDS_PER_MINUTE_TECHNICAL
    } else {
        WORDS_PER_MINUTE
    };

    // Calculate time for English words
    let english_minutes = stats.word_count as f64 / wpm as f64;

    // Calculate time for Chinese characters
    let chinese_minutes = stats.chinese_chars as f64 / CHINESE_CHARS_PER_MINUTE as f64;

    // Add time for code blocks (slower reading)
    let code_minutes = stats.code_blocks as f64 * 0.5; // Assume 30 seconds per code block

    // Total time
    let total_minutes = (english_minutes + chinese_minutes + code_minutes).max(1.0);

    ReadingTime {
        minutes: total_minutes.ceil() as usize,
        words: stats.word_count,
        chinese_chars: stats.chinese_chars,
        code_blocks: stats.code_blocks,
    }
}

#[derive(Debug, Clone)]
pub struct ReadingTime {
    pub minutes: usize,
    pub words: usize,
    pub chinese_chars: usize,
    pub code_blocks: usize,
}

impl ReadingTime {
    /// Get a human-readable string
    pub fn to_string(&self) -> String {
        if self.minutes == 1 {
            "1 min read".to_string()
        } else {
            format!("{} min read", self.minutes)
        }
    }

    /// Get detailed statistics
    pub fn details(&self) -> String {
        let mut parts = Vec::new();

        if self.words > 0 {
            parts.push(format!("{} words", self.words));
        }

        if self.chinese_chars > 0 {
            parts.push(format!("{} Chinese chars", self.chinese_chars));
        }

        if self.code_blocks > 0 {
            parts.push(format!("{} code blocks", self.code_blocks));
        }

        format!("{} ({})", self.to_string(), parts.join(", "))
    }
}

#[derive(Debug, Default)]
struct ContentStats {
    word_count: usize,
    chinese_chars: usize,
    code_blocks: usize,
}

/// Analyze content to get reading statistics
fn analyze_content(content: &str) -> ContentStats {
    let mut stats = ContentStats::default();
    let parser = Parser::new(content);
    let mut in_code_block = false;
    let mut current_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
                stats.code_blocks += 1;
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
            }
            Event::Text(text) => {
                if !in_code_block {
                    current_text.push_str(&text);
                }
            }
            Event::Code(code) => {
                if !in_code_block {
                    current_text.push_str(&code);
                }
            }
            _ => {}
        }
    }

    // Count words and Chinese characters
    for ch in current_text.chars() {
        if is_chinese_char(ch) {
            stats.chinese_chars += 1;
        }
    }

    // Count English words
    stats.word_count = current_text
        .split_whitespace()
        .filter(|word| word.chars().any(|c| c.is_ascii_alphabetic()))
        .count();

    stats
}

/// Check if a character is Chinese
fn is_chinese_char(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' | // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' | // CJK Extension A
        '\u{F900}'..='\u{FAFF}'   // CJK Compatibility Ideographs
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_content() {
        let content =
            "This is a test post with some English content. It should take about 1 minute.";
        let time = calculate_reading_time(content, false);
        assert_eq!(time.minutes, 1);
        assert!(time.words > 0);
        assert_eq!(time.chinese_chars, 0);
    }

    #[test]
    fn test_chinese_content() {
        let content = "这是一篇测试文章，包含一些中文内容。应该需要大约1分钟阅读时间。";
        let time = calculate_reading_time(content, false);
        assert_eq!(time.minutes, 1);
        assert!(time.chinese_chars > 0);
    }

    #[test]
    fn test_mixed_content() {
        let content = r#"
# Mixed Content Test

This is English text. 这是中文文本。

```rust
fn main() {
    println!("Hello, world!");
}
```

More content here.
        "#;
        let time = calculate_reading_time(content, false);
        assert!(time.words > 0);
        assert!(time.chinese_chars > 0);
        assert_eq!(time.code_blocks, 1);
    }
}
