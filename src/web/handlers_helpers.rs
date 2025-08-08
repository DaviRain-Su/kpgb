use crate::models::BlogPost;
use crate::site::SiteConfig;
use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;
use tera::Context;

pub fn create_post_context(
    storage_id: &str,
    post: &BlogPost,
    include_content_html: bool,
) -> Value {
    let mut post_context = serde_json::to_value(post).unwrap();
    post_context["url"] = Value::String(format!("/posts/{}", post.slug));
    post_context["storage_id"] = Value::String(storage_id.to_string());
    
    if include_content_html {
        post_context["content_html"] = Value::String(markdown_to_html(&post.content));
    }
    
    // Generate excerpt HTML
    if post.excerpt.is_none() {
        let excerpt_text = crate::utils::generate_formatted_excerpt(&post.content, 300);
        post_context["excerpt_html"] = Value::String(markdown_to_html(&excerpt_text));
    } else {
        post_context["excerpt_html"] = 
            Value::String(markdown_to_html(post.excerpt.as_ref().unwrap()));
    }
    
    // Add reading time
    let reading_time = crate::utils::calculate_reading_time(&post.content, false);
    post_context["reading_time"] = Value::String(reading_time.to_string());
    
    post_context
}

pub fn create_base_context(site_config: &SiteConfig, page_title: &str) -> Context {
    let mut context = Context::new();
    
    // For web server, always use empty base_path
    let mut config_copy = site_config.clone();
    config_copy.base_path = None;
    
    context.insert("site", &config_copy);
    context.insert("page_title", page_title);
    context
}

pub fn markdown_to_html(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    
    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    
    // Post-process HTML to add IDs to headings
    let html_with_ids = add_heading_ids_to_html(&html_output);
    
    // Post-process HTML to add copy buttons to code blocks
    add_copy_buttons_to_code_blocks(&html_with_ids)
}

fn add_heading_ids_to_html(html: &str) -> String {
    use regex::Regex;
    
    let heading_re = Regex::new(r"<h([1-6])>(.*?)</h[1-6]>").unwrap();
    
    heading_re
        .replace_all(html, |caps: &regex::Captures| {
            let level = &caps[1];
            let content = &caps[2];
            
            // Extract text content without HTML tags for ID generation
            let text_only = Regex::new(r"<[^>]+>").unwrap().replace_all(content, "");
            let id = crate::utils::toc::generate_heading_id(&text_only);
            
            format!(r#"<h{} id="{}">{}</h{}>"#, level, id, content, level)
        })
        .to_string()
}

fn add_copy_buttons_to_code_blocks(html: &str) -> String {
    use regex::Regex;
    
    // Match <pre><code> blocks
    let code_block_re = Regex::new(r"<pre><code([^>]*)>([\s\S]*?)</code></pre>").unwrap();
    
    let mut block_id = 0;
    code_block_re.replace_all(html, |caps: &regex::Captures| {
        block_id += 1;
        let attrs = &caps[1];
        let code_content = &caps[2];
        
        // Extract language class if present
        let lang_re = Regex::new(r#"class="language-([^"]+)""#).unwrap();
        let language = lang_re.captures(attrs)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .unwrap_or("");
        
        // Ensure language class is properly set for Prism.js
        let code_attrs = if !language.is_empty() {
            attrs.to_string()
        } else if attrs.contains("class=") {
            attrs.to_string()
        } else {
            r#" class="language-plaintext""#.to_string()
        };
        
        format!(
            r#"<div class="code-block-wrapper">
                <div class="code-header">
                    <span class="code-language">{}</span>
                    <button class="copy-button" data-code-id="code-{}" onclick="copyCode('code-{}')">
                        <svg class="copy-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                        </svg>
                        <span class="copy-text">Copy</span>
                    </button>
                </div>
                <pre class="line-numbers"><code{} id="code-{}">{}</code></pre>
            </div>"#,
            language,
            block_id,
            block_id,
            code_attrs,
            block_id,
            code_content
        )
    }).to_string()
}

pub fn render_template(
    template_name: &str,
    context: &Context,
) -> Result<String, axum::http::StatusCode> {
    let mut tera = tera::Tera::default();
    
    // Add templates
    tera.add_raw_templates(vec![
        ("base.html", include_str!("../../templates/base.html")),
        ("index.html", include_str!("../../templates/index.html")),
        ("post.html", include_str!("../../templates/post.html")),
        ("archive.html", include_str!("../../templates/archive.html")),
        ("search.html", include_str!("../../templates/search.html")),
        ("tags.html", include_str!("../../templates/tags.html")),
        ("tag_posts.html", include_str!("../../templates/tag_posts.html")),
        ("docs.html", include_str!("../../templates/docs.html")),
    ])
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Register filters
    tera.register_filter("highlight_search", crate::site::filters::highlight_search);
    tera.register_filter("escape", crate::site::filters::escape_html);
    tera.register_filter("url_safe_tag", crate::site::filters::url_safe_tag);
    
    tera.render(template_name, context)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}