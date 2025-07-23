use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use std::sync::Arc;
use tera::Context;

use crate::web::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    author: Option<String>,
    category: Option<String>,
    tag: Option<String>,
    sort: Option<String>,
    page: Option<usize>,
}

#[derive(Deserialize)]
pub struct PageQuery {
    page: Option<usize>,
}

pub async fn index(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PageQuery>,
) -> Result<Html<String>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = state.site_config.posts_per_page;

    let all_posts = state
        .blog_manager
        .list_posts(true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_pages = all_posts.len().div_ceil(posts_per_page);
    let start = (page - 1) * posts_per_page;
    let end = (start + posts_per_page).min(all_posts.len());

    let posts: Vec<_> = all_posts[start..end]
        .iter()
        .map(|(id, post)| {
            let mut post_context = serde_json::to_value(post).unwrap();
            post_context["url"] = serde_json::Value::String(format!("/posts/{}", post.slug));
            post_context["content_html"] =
                serde_json::Value::String(markdown_to_html(&post.content));
            post_context["storage_id"] = serde_json::Value::String(id.clone());

            // Generate excerpt HTML if not provided
            if post.excerpt.is_none() {
                let excerpt_text = crate::utils::generate_formatted_excerpt(&post.content, 300);
                post_context["excerpt_html"] =
                    serde_json::Value::String(markdown_to_html(&excerpt_text));
            } else {
                post_context["excerpt_html"] =
                    serde_json::Value::String(markdown_to_html(post.excerpt.as_ref().unwrap()));
            }

            post_context
        })
        .collect();

    let mut context = Context::new();
    // For web server, always use empty base_path
    let mut site_config = state.site_config.clone();
    site_config.base_path = None;
    context.insert("site", &site_config);
    context.insert("page_title", "Home");
    context.insert("posts", &posts);
    context.insert("current_page", &page);
    context.insert("total_pages", &total_pages);
    context.insert("has_prev", &(page > 1));
    context.insert("has_next", &(page < total_pages));

    let rendered = render_template("index.html", &context)?;
    Ok(Html(rendered))
}

pub async fn post(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let posts = state
        .blog_manager
        .list_posts(true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Find post by slug, preferring ones with tags if there are duplicates
    let post_data = posts
        .iter()
        .filter(|(_, p)| p.slug == slug)
        .max_by_key(|(_, p)| (p.tags.len(), p.created_at))
        .ok_or(StatusCode::NOT_FOUND)?;

    let (storage_id, post) = post_data;

    // Get related posts
    let related_posts = state
        .blog_manager
        .get_related_posts(&post.id, &post.tags, post.category.as_deref(), 5)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let related_posts_data: Vec<_> = related_posts
        .iter()
        .map(|(_, related_post)| {
            let mut post_context = serde_json::to_value(related_post).unwrap();
            post_context["url"] =
                serde_json::Value::String(format!("/posts/{}", related_post.slug));

            // Add reading time
            let reading_time = crate::utils::calculate_reading_time(&related_post.content, false);
            post_context["reading_time"] = serde_json::Value::String(reading_time.to_string());

            post_context
        })
        .collect();

    let mut context = Context::new();
    // For web server, always use empty base_path
    let mut site_config = state.site_config.clone();
    site_config.base_path = None;
    context.insert("site", &site_config);
    context.insert("page_title", &post.title);
    context.insert("post", post);
    context.insert("content_html", &markdown_to_html(&post.content));
    context.insert("storage_id", storage_id);
    context.insert("related_posts", &related_posts_data);

    if storage_id.starts_with("Qm") {
        context.insert(
            "ipfs_link",
            &format!("{}{}", state.site_config.ipfs_gateway, storage_id),
        );
    }

    let rendered = render_template("post.html", &context)?;
    Ok(Html(rendered))
}

pub async fn archive(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let posts = state
        .blog_manager
        .list_posts(true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut posts_by_year: std::collections::HashMap<i32, Vec<_>> =
        std::collections::HashMap::new();

    for (id, post) in posts {
        let year = post.created_at.year();
        let mut post_context = serde_json::to_value(&post).unwrap();
        post_context["url"] = serde_json::Value::String(format!("/posts/{}", post.slug));
        post_context["storage_id"] = serde_json::Value::String(id.clone());

        posts_by_year
            .entry(year)
            .or_insert_with(Vec::new)
            .push(post_context);
    }

    let mut years: Vec<_> = posts_by_year.into_iter().collect();
    years.sort_by(|a, b| b.0.cmp(&a.0));

    let mut context = Context::new();
    // For web server, always use empty base_path
    let mut site_config = state.site_config.clone();
    site_config.base_path = None;
    context.insert("site", &site_config);
    context.insert("page_title", "Archive");
    context.insert("years", &years);

    let rendered = render_template("archive.html", &context)?;
    Ok(Html(rendered))
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<Html<String>, StatusCode> {
    let query = params.q.unwrap_or_default();
    let author_filter = params.author.unwrap_or_default();
    let category_filter = params.category.unwrap_or_default();
    let tag_filter = params.tag.unwrap_or_default();
    let sort_order = params.sort.unwrap_or_else(|| "relevance".to_string());
    let page = params.page.unwrap_or(1);
    let posts_per_page = state.site_config.posts_per_page;

    // Get all posts
    let all_posts = state
        .blog_manager
        .list_posts(true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Apply filters and search
    let start_time = std::time::Instant::now();
    let mut filtered_posts: Vec<(String, crate::models::BlogPost, f32)> = Vec::new();

    for (id, post) in all_posts {
        // Apply filters
        if !author_filter.is_empty() && post.author != author_filter {
            continue;
        }
        if !category_filter.is_empty() && post.category.as_ref() != Some(&category_filter) {
            continue;
        }
        if !tag_filter.is_empty() && !post.tags.contains(&tag_filter) {
            continue;
        }

        // Calculate search score if query exists
        let score = if !query.is_empty() {
            calculate_search_score(&post, &query)
        } else {
            1.0 // Default score for filtered results without search query
        };

        if score > 0.0 || query.is_empty() {
            filtered_posts.push((id, post, score));
        }
    }

    // Sort results based on sort order
    match sort_order.as_str() {
        "date_desc" => filtered_posts.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at)),
        "date_asc" => filtered_posts.sort_by(|a, b| a.1.created_at.cmp(&b.1.created_at)),
        "title" => filtered_posts.sort_by(|a, b| a.1.title.cmp(&b.1.title)),
        _ => filtered_posts.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap()), // relevance
    }

    let search_time = start_time.elapsed().as_millis();
    let total_results = filtered_posts.len();
    let total_pages = total_results.div_ceil(posts_per_page);

    // Paginate results
    let start = (page - 1) * posts_per_page;
    let end = (start + posts_per_page).min(filtered_posts.len());
    let page_posts = &filtered_posts[start..end];

    // Prepare posts for template
    let posts: Vec<_> = page_posts
        .iter()
        .map(|(id, post, score)| {
            let mut post_context = serde_json::to_value(post).unwrap();
            post_context["url"] = serde_json::Value::String(format!("/posts/{}", post.slug));
            post_context["storage_id"] = serde_json::Value::String(id.clone());

            // Add search score as percentage
            if !query.is_empty() {
                post_context["search_score"] = serde_json::Value::Number(
                    serde_json::Number::from_f64((score * 100.0) as f64).unwrap(),
                );
            }

            // Add reading time
            let reading_time = crate::utils::calculate_reading_time(&post.content, false);
            post_context["reading_time"] = serde_json::Value::String(reading_time.to_string());

            // Generate excerpt with highlighting if query exists
            if !query.is_empty() {
                if let Some(excerpt) = generate_highlighted_excerpt(&post.content, &query, 200) {
                    post_context["excerpt_highlighted"] = serde_json::Value::String(excerpt);
                }
            }

            post_context
        })
        .collect();

    // Get all unique values for filters
    let all_posts_for_filters = state
        .blog_manager
        .list_posts(true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut authors = std::collections::HashSet::new();
    let mut categories = std::collections::HashSet::new();
    let mut tags = std::collections::HashSet::new();

    for (_, post) in &all_posts_for_filters {
        authors.insert(post.author.clone());
        if let Some(cat) = &post.category {
            categories.insert(cat.clone());
        }
        for tag in &post.tags {
            tags.insert(tag.clone());
        }
    }

    let mut authors_vec: Vec<_> = authors.into_iter().collect();
    let mut categories_vec: Vec<_> = categories.into_iter().collect();
    let mut tags_vec: Vec<_> = tags.into_iter().collect();
    authors_vec.sort();
    categories_vec.sort();
    tags_vec.sort();

    // Get total posts count before moving the collection
    let total_posts_count = all_posts_for_filters.len();

    // Get popular tags and recent posts for empty search
    let popular_tags = if query.is_empty()
        && author_filter.is_empty()
        && category_filter.is_empty()
        && tag_filter.is_empty()
    {
        state
            .blog_manager
            .get_all_tags()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .into_iter()
            .take(10)
            .map(|(tag, _)| tag)
            .collect()
    } else {
        Vec::new()
    };

    let recent_posts = if query.is_empty()
        && author_filter.is_empty()
        && category_filter.is_empty()
        && tag_filter.is_empty()
    {
        all_posts_for_filters
            .into_iter()
            .take(5)
            .map(|(_, post)| {
                let mut post_map = serde_json::Map::new();
                post_map.insert("title".to_string(), serde_json::Value::String(post.title));
                post_map.insert(
                    "url".to_string(),
                    serde_json::Value::String(format!("/posts/{}", post.slug)),
                );
                post_map.insert(
                    "created_at".to_string(),
                    serde_json::Value::String(post.created_at.to_rfc3339()),
                );
                serde_json::Value::Object(post_map)
            })
            .collect()
    } else {
        Vec::new()
    };

    let mut context = Context::new();
    // For web server, always use empty base_path
    let mut site_config = state.site_config.clone();
    site_config.base_path = None;
    context.insert("site", &site_config);
    context.insert("page_title", "Search");
    context.insert("query", &query);
    context.insert("posts", &posts);
    context.insert("count", &total_results);
    context.insert("search_time", &search_time);
    context.insert("total_posts", &total_posts_count);

    // Filter values
    context.insert("authors", &authors_vec);
    context.insert("categories", &categories_vec);
    context.insert("tags", &tags_vec);
    context.insert("selected_author", &author_filter);
    context.insert("selected_category", &category_filter);
    context.insert("selected_tag", &tag_filter);
    context.insert("sort", &sort_order);

    // Pagination
    context.insert("current_page", &page);
    context.insert("total_pages", &total_pages);
    context.insert("has_prev", &(page > 1));
    context.insert("has_next", &(page < total_pages));

    // Suggestions
    context.insert("popular_tags", &popular_tags);
    context.insert("recent_posts", &recent_posts);

    let rendered = render_template("search.html", &context)?;
    Ok(Html(rendered))
}

pub async fn style_css(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let css_content = match state.site_config.theme.as_str() {
        "hacker" => include_str!("../../templates/themes/hacker.css"),
        "minimal" => include_str!("../../templates/themes/minimal.css"),
        "dark" => include_str!("../../templates/themes/dark.css"),
        "cyberpunk" => include_str!("../../templates/themes/cyberpunk.css"),
        _ => include_str!("../../templates/style.css"), // default theme
    };

    (StatusCode::OK, [("content-type", "text/css")], css_content)
}

pub async fn docs(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let mut context = Context::new();
    // For web server, always use empty base_path
    let mut site_config = state.site_config.clone();
    site_config.base_path = None;
    context.insert("site", &site_config);
    context.insert("page_title", "技术文档中心");

    let rendered = render_template("docs.html", &context)?;
    Ok(Html(rendered))
}

fn render_template(name: &str, context: &Context) -> Result<String, StatusCode> {
    let mut tera = tera::Tera::default();
    tera.add_raw_templates(vec![
        ("base.html", include_str!("../../templates/base.html")),
        ("index.html", include_str!("../../templates/index.html")),
        ("post.html", include_str!("../../templates/post.html")),
        ("archive.html", include_str!("../../templates/archive.html")),
        ("search.html", include_str!("../../templates/search.html")),
        ("tags.html", include_str!("../../templates/tags.html")),
        (
            "tag_posts.html",
            include_str!("../../templates/tag_posts.html"),
        ),
        ("docs.html", include_str!("../../templates/docs.html")),
    ])
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Add custom filters
    tera.register_filter(
        "url_safe_tag",
        |value: &tera::Value, _: &std::collections::HashMap<String, tera::Value>| match value
            .as_str()
        {
            Some(tag) => Ok(tera::Value::String(sanitize_tag_for_url(tag))),
            None => Err(tera::Error::msg("url_safe_tag filter expects a string")),
        },
    );

    tera.register_filter("highlight_search", crate::site::filters::highlight_search);
    tera.register_filter("escape", crate::site::filters::escape_html);

    tera.render(name, context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn sanitize_tag_for_url(tag: &str) -> String {
    // For tags, we'll use a simple approach: convert to lowercase and replace spaces with hyphens
    // Chinese characters and other non-ASCII will be preserved
    tag.to_lowercase().replace(' ', "-")
}

fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

use chrono::Datelike;
use rss::{ChannelBuilder, ItemBuilder};

/// Calculate search relevance score for a post
fn calculate_search_score(post: &crate::models::BlogPost, query: &str) -> f32 {
    let query_lower = query.to_lowercase();
    let words: Vec<&str> = query_lower.split_whitespace().collect();

    if words.is_empty() {
        return 0.0;
    }

    let mut score = 0.0;
    let title_lower = post.title.to_lowercase();
    let content_lower = post.content.to_lowercase();
    let excerpt_lower = post
        .excerpt
        .as_ref()
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    for word in &words {
        // Title matches (highest weight)
        if title_lower.contains(word) {
            score += 10.0;
            // Exact word match in title
            if title_lower.split_whitespace().any(|w| w == *word) {
                score += 5.0;
            }
        }

        // Tag matches (high weight)
        for tag in &post.tags {
            if tag.to_lowercase().contains(word) {
                score += 5.0;
            }
        }

        // Category match
        if let Some(cat) = &post.category {
            if cat.to_lowercase().contains(word) {
                score += 3.0;
            }
        }

        // Excerpt match
        if excerpt_lower.contains(word) {
            score += 2.0;
        }

        // Content matches (lower weight, but count frequency)
        let content_matches = content_lower.matches(word).count();
        score += (content_matches as f32).min(5.0) * 0.5;
    }

    // Normalize score
    score / words.len() as f32
}

/// Generate excerpt with search term highlighting
fn generate_highlighted_excerpt(content: &str, query: &str, max_length: usize) -> Option<String> {
    let query_lower = query.to_lowercase();
    let words: Vec<&str> = query_lower.split_whitespace().collect();

    if words.is_empty() {
        return None;
    }

    let content_lower = content.to_lowercase();

    // Find the first occurrence of any search word
    let mut best_pos = None;
    for word in &words {
        if let Some(pos) = content_lower.find(word) {
            if best_pos.is_none() || pos < best_pos.unwrap() {
                best_pos = Some(pos);
            }
        }
    }

    let start_pos = best_pos?;

    // Find excerpt boundaries
    let excerpt_start = start_pos.saturating_sub(max_length / 2);
    let excerpt_end = (start_pos + max_length / 2).min(content.len());

    // Adjust to word boundaries
    let excerpt_start = if excerpt_start > 0 {
        content[..excerpt_start]
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(excerpt_start)
    } else {
        0
    };

    let excerpt_end = content[excerpt_end..]
        .find(|c: char| c.is_whitespace())
        .map(|i| excerpt_end + i)
        .unwrap_or(excerpt_end);

    let mut excerpt = content[excerpt_start..excerpt_end].to_string();

    // Add ellipsis if needed
    if excerpt_start > 0 {
        excerpt = format!("...{}", excerpt);
    }
    if excerpt_end < content.len() {
        excerpt = format!("{}...", excerpt);
    }

    // Highlight search terms
    for word in &words {
        let pattern = regex::Regex::new(&format!(r"(?i)\b{}\b", regex::escape(word))).ok()?;
        excerpt = pattern
            .replace_all(&excerpt, |caps: &regex::Captures| {
                format!("<mark>{}</mark>", &caps[0])
            })
            .to_string();
    }

    Some(excerpt)
}

pub async fn rss_feed(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, StatusCode> {
    let posts = state
        .blog_manager
        .list_posts(true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut items = Vec::new();

    // Take the 20 most recent posts
    let recent_posts: Vec<_> = posts.into_iter().take(20).collect();

    for (storage_id, post) in recent_posts {
        let link = format!("{}/posts/{}", state.site_config.base_url, post.slug);
        let content_html = markdown_to_html(&post.content);

        let item = ItemBuilder::default()
            .title(Some(post.title))
            .link(Some(link))
            .description(Some(content_html))
            .author(Some(post.author))
            .pub_date(Some(post.created_at.to_rfc2822()))
            .guid(Some(rss::Guid {
                value: storage_id,
                permalink: false,
            }))
            .build();

        items.push(item);
    }

    let channel = ChannelBuilder::default()
        .title(&state.site_config.title)
        .link(&state.site_config.base_url)
        .description(&state.site_config.description)
        .items(items)
        .build();

    Ok((
        StatusCode::OK,
        [("content-type", "application/rss+xml")],
        channel.to_string(),
    ))
}

pub async fn tags(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let tags = state
        .blog_manager
        .get_all_tags()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut context = Context::new();
    // Strip base_path for web server templates
    let mut site_config = state.site_config.clone();
    site_config.base_path = None;
    context.insert("site", &site_config);

    let tag_data: Vec<_> = tags
        .into_iter()
        .map(|(name, count)| {
            let mut tag_map = serde_json::Map::new();
            tag_map.insert("name".to_string(), serde_json::Value::String(name.clone()));
            tag_map.insert("count".to_string(), serde_json::Value::Number(count.into()));
            tag_map.insert(
                "url".to_string(),
                serde_json::Value::String(format!("/tags/{}", name)),
            );
            serde_json::Value::Object(tag_map)
        })
        .collect();

    context.insert("tags", &tag_data);
    context.insert("title", "Tags");

    let html = render_template("tags.html", &context)?;

    Ok(Html(html))
}

pub async fn tag_posts(
    State(state): State<Arc<AppState>>,
    Path(tag): Path<String>,
    Query(params): Query<PageQuery>,
) -> Result<Html<String>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = state.site_config.posts_per_page;

    let all_posts = state
        .blog_manager
        .get_posts_by_tag(&tag, true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_pages = all_posts.len().div_ceil(posts_per_page);
    let start = (page - 1) * posts_per_page;
    let end = (start + posts_per_page).min(all_posts.len());

    let posts_data: Vec<_> = all_posts[start..end]
        .iter()
        .map(|(id, post)| {
            let mut post_context = serde_json::to_value(post).unwrap();
            post_context["url"] = serde_json::Value::String(format!("/posts/{}", post.slug));
            post_context["content_html"] =
                serde_json::Value::String(markdown_to_html(&post.content));
            post_context["storage_id"] = serde_json::Value::String(id.clone());

            // Generate excerpt HTML if not provided
            if post.excerpt.is_none() {
                let excerpt_text = crate::utils::generate_formatted_excerpt(&post.content, 300);
                post_context["excerpt_html"] =
                    serde_json::Value::String(markdown_to_html(&excerpt_text));
            } else {
                post_context["excerpt_html"] =
                    serde_json::Value::String(markdown_to_html(post.excerpt.as_ref().unwrap()));
            }

            post_context
        })
        .collect();

    let mut context = Context::new();
    // Strip base_path for web server templates
    let mut site_config = state.site_config.clone();
    site_config.base_path = None;
    context.insert("site", &site_config);
    context.insert("posts", &posts_data);
    context.insert("tag", &tag);
    context.insert("title", &format!("Posts tagged '{}'", tag));

    // Pagination context
    context.insert("current_page", &page);
    context.insert("total_pages", &total_pages);
    context.insert("has_prev", &(page > 1));
    context.insert("has_next", &(page < total_pages));

    let html = render_template("tag_posts.html", &context)?;

    Ok(Html(html))
}

// Redirect handlers for backward compatibility
pub async fn redirect_archive() -> impl IntoResponse {
    Redirect::permanent("/archive")
}
