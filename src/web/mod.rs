pub mod api;
pub mod handlers;
pub mod server;

use crate::blog::BlogManager;
use crate::site::SiteConfig;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

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
        .route("/feed.xml", get(handlers::rss_feed))
        // Redirects for backward compatibility
        .route("/archive.html", get(handlers::redirect_archive))
        // CORS for API access
        .layer(CorsLayer::permissive())
        // State
        .with_state(state)
}
