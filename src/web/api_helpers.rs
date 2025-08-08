use crate::models::BlogPost;
use crate::web::api::{ApiResponse, PostSummary};
use axum::response::Json;

pub fn create_post_summary(storage_id: String, post: BlogPost) -> PostSummary {
    PostSummary {
        id: post.id,
        storage_id,
        title: post.title,
        author: post.author,
        created_at: post.created_at.to_rfc3339(),
        published: post.published,
        tags: post.tags,
        excerpt: post.excerpt,
    }
}

pub fn posts_to_summaries(posts: Vec<(String, BlogPost)>) -> Vec<PostSummary> {
    posts
        .into_iter()
        .map(|(storage_id, post)| create_post_summary(storage_id, post))
        .collect()
}

pub fn success_response<T>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data: Some(data),
        error: None,
    })
}

pub fn error_response<T>(error: impl ToString) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: false,
        data: None,
        error: Some(error.to_string()),
    })
}

pub fn handle_result<T, E: ToString>(result: Result<T, E>) -> Json<ApiResponse<T>> {
    match result {
        Ok(data) => success_response(data),
        Err(e) => error_response(e),
    }
}
