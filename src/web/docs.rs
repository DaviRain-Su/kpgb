use axum::{
    extract::{Path, Query, State},
    response::Html,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use tera::Context;
use crate::docs::DocsDatabase;
use super::AppState;

#[derive(Deserialize)]
pub struct DocsQuery {
    category: Option<String>,
}

/// 文档首页 - 显示所有分类
pub async fn docs_index(
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let categories = state.docs_db
        .get_all_categories()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut context = Context::new();
    context.insert("site", &state.site_config);
    context.insert("page_title", "Documentation");
    context.insert("categories", &categories);

    let html = state
        .templates
        .render("docs_index.html", &context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

/// 文档分类页 - 显示某个分类下的所有文档
pub async fn docs_category(
    State(state): State<Arc<AppState>>,
    Path(category_slug): Path<String>,
) -> Result<Html<String>, StatusCode> {
    // 查找分类
    let categories = state.docs_db
        .get_all_categories()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let category = categories
        .iter()
        .find(|c| c.slug == category_slug)
        .ok_or(StatusCode::NOT_FOUND)?;

    // 获取该分类下的所有章节
    let sections = state.docs_db
        .get_sections_by_category(&category.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut context = Context::new();
    context.insert("site", &state.site_config);
    context.insert("page_title", &format!("{} - Documentation", category.name));
    context.insert("category", category);
    context.insert("sections", &sections);
    context.insert("categories", &categories); // 用于侧边栏

    let html = state
        .templates
        .render("docs_category.html", &context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

/// 文档详情页 - 显示具体的文档内容
pub async fn docs_detail(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, StatusCode> {
    // 获取文档
    let section = state.docs_db
        .get_section_by_slug(&slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 获取所有分类（用于侧边栏）
    let categories = state.docs_db
        .get_all_categories()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 将 Markdown 转换为 HTML
    let content_html = super::markdown_to_html(&section.content);

    let mut context = Context::new();
    context.insert("site", &state.site_config);
    context.insert("page_title", &format!("{} - Documentation", section.title));
    context.insert("section", &section);
    context.insert("content_html", &content_html);
    context.insert("categories", &categories);

    let html = state
        .templates
        .render("docs_detail.html", &context)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

/// API: 获取文档内容（用于翻译）
pub async fn api_get_doc(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
    let section = state.docs_db
        .get_section_by_slug(&slug)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(axum::Json(serde_json::json!({
        "id": section.id,
        "title": section.title,
        "content": section.content,
        "source_url": section.source_url,
        "is_translated": section.is_translated
    })))
}

/// API: 更新文档翻译
#[derive(Deserialize)]
pub struct UpdateDocRequest {
    content: String,
}

pub async fn api_update_doc(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    axum::Json(req): axum::Json<UpdateDocRequest>,
) -> Result<StatusCode, StatusCode> {
    state.docs_db
        .update_section_content(&id, &req.content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}