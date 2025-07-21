use crate::web::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct PostSummary {
    id: String,
    storage_id: String,
    title: String,
    author: String,
    created_at: String,
    published: bool,
    tags: Vec<String>,
    excerpt: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchRequest {
    query: String,
}

pub async fn list_posts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<PostSummary>>>, StatusCode> {
    match state.blog_manager.list_posts(true).await {
        Ok(posts) => {
            let summaries: Vec<PostSummary> = posts
                .into_iter()
                .map(|(storage_id, post)| PostSummary {
                    id: post.id,
                    storage_id,
                    title: post.title,
                    author: post.author,
                    created_at: post.created_at.to_rfc3339(),
                    published: post.published,
                    tags: post.tags,
                    excerpt: post.excerpt,
                })
                .collect();

            Ok(Json(ApiResponse {
                success: true,
                data: Some(summaries),
                error: None,
            }))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    match state.blog_manager.get_post(&id).await {
        Ok(post) => {
            let mut post_json = serde_json::to_value(&post).unwrap();
            post_json["storage_id"] = serde_json::Value::String(id);

            Ok(Json(ApiResponse {
                success: true,
                data: Some(post_json),
                error: None,
            }))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

pub async fn search_posts(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<ApiResponse<Vec<PostSummary>>>, StatusCode> {
    match state.blog_manager.search_posts(&req.query).await {
        Ok(results) => {
            let summaries: Vec<PostSummary> = results
                .into_iter()
                .map(|(storage_id, post)| PostSummary {
                    id: post.id,
                    storage_id,
                    title: post.title,
                    author: post.author,
                    created_at: post.created_at.to_rfc3339(),
                    published: post.published,
                    tags: post.tags,
                    excerpt: post.excerpt,
                })
                .collect();

            Ok(Json(ApiResponse {
                success: true,
                data: Some(summaries),
                error: None,
            }))
        }
        Err(e) => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        })),
    }
}
