use regex::Regex;
use std::collections::HashMap;
use tera::{Result, Value};

/// Highlight search terms in text
pub fn highlight_search(value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
    let text = match value.as_str() {
        Some(s) => s,
        None => return Ok(value.clone()),
    };

    let query = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) if !q.is_empty() => q,
        _ => return Ok(value.clone()),
    };

    // Split query into words
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.is_empty() {
        return Ok(value.clone());
    }

    let mut highlighted = text.to_string();

    // Highlight each word
    for word in words {
        // Escape special regex characters
        let escaped_word = regex::escape(word);
        let pattern = format!(r"(?i)\b{}\b", escaped_word);

        if let Ok(re) = Regex::new(&pattern) {
            highlighted = re
                .replace_all(&highlighted, |caps: &regex::Captures| {
                    format!("<mark>{}</mark>", &caps[0])
                })
                .to_string();
        }
    }

    Ok(Value::String(highlighted))
}

/// Convert tag to URL-safe format
pub fn url_safe_tag(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    let tag = match value.as_str() {
        Some(s) => s,
        None => return Ok(value.clone()),
    };

    // Only keep ASCII alphanumeric and hyphen
    let safe_tag: String = tag
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    Ok(Value::String(safe_tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_search() {
        let mut args = HashMap::new();
        args.insert(
            "query".to_string(),
            Value::String("rust programming".to_string()),
        );

        let text = Value::String("Learn Rust programming with this guide".to_string());
        let result = highlight_search(&text, &args).unwrap();

        assert_eq!(
            result.as_str().unwrap(),
            "Learn <mark>Rust</mark> <mark>programming</mark> with this guide"
        );
    }

    #[test]
    fn test_url_safe_tag() {
        let tag = Value::String("Rust & Web Development!".to_string());
        let result = url_safe_tag(&tag, &HashMap::new()).unwrap();

        assert_eq!(result.as_str().unwrap(), "rust-web-development");
    }
}
