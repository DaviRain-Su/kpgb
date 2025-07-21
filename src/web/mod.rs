pub mod server;
pub mod handlers;
pub mod api;

use axum::{Router, routing::{get, post}};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use crate::blog::BlogManager;
use crate::site::SiteConfig;

pub struct AppState {
    pub blog_manager: BlogManager,
    pub site_config: SiteConfig,
}

pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        // Web UI routes
        .route("/", get(handlers::index))
        .route("/posts/:slug", get(handlers::post))
        .route("/archive", get(handlers::archive))
        .route("/search", get(handlers::search))
        
        // API routes
        .route("/api/posts", get(api::list_posts))
        .route("/api/posts/:id", get(api::get_post))
        .route("/api/search", post(api::search_posts))
        
        // Static files
        .route("/css/style.css", get(handlers::style_css))
        
        // CORS for API access
        .layer(CorsLayer::permissive())
        
        // State
        .with_state(state)
}