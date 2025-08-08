use crate::web::AppState;
use crate::web::api_helpers::{posts_to_summaries, handle_result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::web::api::{ApiResponse, PostSummary};

#[derive(Serialize)]
pub struct TagInfo {
    name: String,
    post_count: i64,
}

#[derive(Deserialize)]
pub struct TagQuery {
    tag: Option<String>,
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<TagInfo>>>, StatusCode> {
    let result = state.blog_manager.get_all_tags().await.map(|tags| {
        tags.into_iter()
            .map(|(name, count)| TagInfo { name, post_count: count })
            .collect()
    });
    Ok(handle_result(result))
}

pub async fn get_posts_by_tag(
    State(state): State<Arc<AppState>>,
    Path(tag): Path<String>,
) -> Result<Json<ApiResponse<Vec<PostSummary>>>, StatusCode> {
    let result = state.blog_manager.get_posts_by_tag(&tag, true).await
        .map(posts_to_summaries);
    Ok(handle_result(result))
}

pub async fn list_posts_with_tag_filter(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TagQuery>,
) -> Result<Json<ApiResponse<Vec<PostSummary>>>, StatusCode> {
    let result = if let Some(tag) = params.tag {
        state.blog_manager.get_posts_by_tag(&tag, true).await
    } else {
        state.blog_manager.list_posts(true).await
    }.map(posts_to_summaries);
    
    Ok(handle_result(result))
}
