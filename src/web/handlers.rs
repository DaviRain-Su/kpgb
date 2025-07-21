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

    let post_data = posts
        .iter()
        .find(|(_, p)| p.slug == slug)
        .ok_or(StatusCode::NOT_FOUND)?;

    let (storage_id, post) = post_data;

    let mut context = Context::new();
    // For web server, always use empty base_path
    let mut site_config = state.site_config.clone();
    site_config.base_path = None;
    context.insert("site", &site_config);
    context.insert("page_title", &post.title);
    context.insert("post", post);
    context.insert("content_html", &markdown_to_html(&post.content));
    context.insert("storage_id", storage_id);

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

    let results = if !query.is_empty() {
        state
            .blog_manager
            .search_posts(&query)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        Vec::new()
    };

    let posts: Vec<_> = results
        .iter()
        .map(|(id, post)| {
            let mut post_context = serde_json::to_value(post).unwrap();
            post_context["url"] = serde_json::Value::String(format!("/posts/{}", post.slug));
            post_context["storage_id"] = serde_json::Value::String(id.clone());
            post_context
        })
        .collect();

    let mut context = Context::new();
    // For web server, always use empty base_path
    let mut site_config = state.site_config.clone();
    site_config.base_path = None;
    context.insert("site", &site_config);
    context.insert("page_title", "Search");
    context.insert("query", &query);
    context.insert("posts", &posts);
    context.insert("count", &posts.len());

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

fn render_template(name: &str, context: &Context) -> Result<String, StatusCode> {
    let mut tera = tera::Tera::default();
    tera.add_raw_templates(vec![
        ("base.html", include_str!("../../templates/base.html")),
        ("index.html", include_str!("../../templates/index.html")),
        ("post.html", include_str!("../../templates/post.html")),
        ("archive.html", include_str!("../../templates/archive.html")),
        ("search.html", include_str!("../../templates/search.html")),
    ])
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tera.render(name, context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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

// Redirect handlers for backward compatibility
pub async fn redirect_archive() -> impl IntoResponse {
    Redirect::permanent("/archive")
}
