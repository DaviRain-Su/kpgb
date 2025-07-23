use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::storage::StorageManager;

/// Process markdown content and upload local images to IPFS
pub async fn process_images_in_markdown(
    content: &str,
    base_path: Option<&Path>,
    storage_manager: &StorageManager,
) -> Result<(String, HashMap<String, String>)> {
    // Regex to find markdown images: ![alt](path)
    let img_regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)")?;

    let mut processed_content = content.to_string();
    let mut image_map = HashMap::new();
    let mut replacements = Vec::new();

    // Find all images in the content
    for cap in img_regex.captures_iter(content) {
        let alt_text = &cap[1];
        let img_path = &cap[2];

        // Skip if already an HTTP(S) URL or IPFS path
        if img_path.starts_with("http://")
            || img_path.starts_with("https://")
            || img_path.starts_with("ipfs://")
            || img_path.starts_with("/ipfs/")
        {
            continue;
        }

        // Resolve the image path
        let resolved_path = resolve_image_path(img_path, base_path)?;

        if !resolved_path.exists() {
            info!("Image not found, skipping: {:?}", resolved_path);
            continue;
        }

        // Read the image file
        let image_data = std::fs::read(&resolved_path)?;
        let file_name = resolved_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image");

        // Determine MIME type
        let mime_type = match resolved_path.extension().and_then(|e| e.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("webp") => "image/webp",
            _ => "application/octet-stream",
        };

        // Prepare metadata
        let mut metadata = HashMap::new();
        metadata.insert("filename".to_string(), file_name.to_string());
        metadata.insert("content_type".to_string(), mime_type.to_string());

        // Upload to storage (IPFS)
        info!("Uploading image to IPFS: {}", file_name);
        let storage_result = storage_manager.store(&image_data, metadata).await?;

        // Get the IPFS URL
        let ipfs_url = storage_result
            .url
            .unwrap_or_else(|| format!("/ipfs/{}", storage_result.id));

        info!("Image uploaded: {} -> {}", img_path, ipfs_url);

        // Store the mapping
        image_map.insert(img_path.to_string(), ipfs_url.clone());

        // Prepare replacement
        let new_markdown = format!("![{}]({})", alt_text, ipfs_url);
        replacements.push((cap[0].to_string(), new_markdown));
    }

    // Apply all replacements
    for (old, new) in replacements {
        processed_content = processed_content.replace(&old, &new);
    }

    Ok((processed_content, image_map))
}

/// Resolve image path relative to base path
fn resolve_image_path(img_path: &str, base_path: Option<&Path>) -> Result<PathBuf> {
    let path = Path::new(img_path);

    // If absolute path, return as is
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    // If base path is provided, resolve relative to it
    if let Some(base) = base_path {
        let resolved = base.join(path);
        if resolved.exists() {
            return Ok(resolved);
        }
    }

    // Try current directory
    let current_dir = std::env::current_dir()?;
    let resolved = current_dir.join(path);
    if resolved.exists() {
        return Ok(resolved);
    }

    // Return the original path if nothing works
    Ok(path.to_path_buf())
}

/// Extract all image URLs from markdown content
pub fn extract_image_urls(content: &str) -> Vec<String> {
    let img_regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();

    img_regex
        .captures_iter(content)
        .map(|cap| cap[2].to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_image_urls() {
        let content = r#"
# Test Post

Here's an image: ![alt text](image.png)

And another: ![](https://example.com/image.jpg)

And IPFS: ![ipfs image](/ipfs/QmXxx)
        "#;

        let urls = extract_image_urls(content);
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "image.png");
        assert_eq!(urls[1], "https://example.com/image.jpg");
        assert_eq!(urls[2], "/ipfs/QmXxx");
    }
}
