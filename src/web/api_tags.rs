use crate::web::AppState;
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
    match state.blog_manager.get_all_tags().await {
        Ok(tags) => {
            let tag_infos: Vec<TagInfo> = tags
                .into_iter()
                .map(|(name, count)| TagInfo {
                    name,
                    post_count: count,
                })
                .collect();

            Ok(Json(ApiResponse {
                success: true,
                data: Some(tag_infos),
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

pub async fn get_posts_by_tag(
    State(state): State<Arc<AppState>>,
    Path(tag): Path<String>,
) -> Result<Json<ApiResponse<Vec<PostSummary>>>, StatusCode> {
    match state.blog_manager.get_posts_by_tag(&tag, true).await {
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

pub async fn list_posts_with_tag_filter(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TagQuery>,
) -> Result<Json<ApiResponse<Vec<PostSummary>>>, StatusCode> {
    let posts = if let Some(tag) = params.tag {
        state.blog_manager.get_posts_by_tag(&tag, true).await
    } else {
        state.blog_manager.list_posts(true).await
    };

    match posts {
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