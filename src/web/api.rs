use crate::web::AppState;
use crate::web::api_helpers::{posts_to_summaries, handle_result};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub(crate) success: bool,
    pub(crate) data: Option<T>,
    pub(crate) error: Option<String>,
}

#[derive(Serialize)]
pub struct PostSummary {
    pub(crate) id: String,
    pub(crate) storage_id: String,
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) created_at: String,
    pub(crate) published: bool,
    pub(crate) tags: Vec<String>,
    pub(crate) excerpt: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchRequest {
    query: String,
}

pub async fn list_posts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<PostSummary>>>, StatusCode> {
    let result = state.blog_manager.list_posts(true).await
        .map(posts_to_summaries);
    Ok(handle_result(result))
}

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let result = state.blog_manager.get_post(&id).await.map(|post| {
        let mut post_json = serde_json::to_value(&post).unwrap();
        post_json["storage_id"] = serde_json::Value::String(id);
        post_json
    });
    Ok(handle_result(result))
}

pub async fn search_posts(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<ApiResponse<Vec<PostSummary>>>, StatusCode> {
    let result = state.blog_manager.search_posts(&req.query).await
        .map(posts_to_summaries);
    Ok(handle_result(result))
}
