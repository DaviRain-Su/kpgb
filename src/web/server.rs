use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use super::{create_app, AppState};
use crate::blog::BlogManager;
use crate::site::SiteConfig;

pub struct WebServer {
    app_state: Arc<AppState>,
    addr: SocketAddr,
}

impl WebServer {
    pub fn new(blog_manager: BlogManager, site_config: SiteConfig, port: u16) -> Self {
        let app_state = Arc::new(AppState {
            blog_manager,
            site_config,
        });

        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        Self { app_state, addr }
    }

    pub async fn run(self) -> Result<()> {
        let app = create_app(self.app_state);

        info!("🚀 Web server starting on http://{}", self.addr);

        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
