// Common content types
pub const CONTENT_TYPE_OCTET_STREAM: &str = "application/octet-stream";
pub const CONTENT_TYPE_MARKDOWN: &str = "text/markdown";
pub const CONTENT_TYPE_HTML: &str = "text/html";

// Storage backend names
pub const STORAGE_IPFS: &str = "ipfs";
pub const STORAGE_LOCAL: &str = "local";
pub const STORAGE_GITHUB: &str = "github";

// Default values
pub const DEFAULT_IPFS_API_URL: &str = "http://localhost:5001";
pub const DEFAULT_POSTS_PER_PAGE: usize = 10;
pub const DEFAULT_EXCERPT_LENGTH: usize = 300;

// Error messages
pub const ERROR_STORAGE_NOT_CONFIGURED: &str = "Storage backend '{}' not configured";
pub const ERROR_IPFS_IMMUTABLE: &str = "IPFS content is immutable and cannot be deleted";

// Metadata keys
pub const METADATA_CONTENT_TYPE: &str = "content_type";
pub const METADATA_TITLE: &str = "title";
pub const METADATA_AUTHOR: &str = "author";

// Template names
pub const TEMPLATE_BASE: &str = "base.html";
pub const TEMPLATE_INDEX: &str = "index.html";
pub const TEMPLATE_POST: &str = "post.html";
pub const TEMPLATE_ARCHIVE: &str = "archive.html";
pub const TEMPLATE_SEARCH: &str = "search.html";
pub const TEMPLATE_TAGS: &str = "tags.html";
pub const TEMPLATE_TAG_POSTS: &str = "tag_posts.html";
pub const TEMPLATE_DOCS: &str = "docs.html";