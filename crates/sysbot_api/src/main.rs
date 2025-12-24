//! SysBot.rs API Server
//!
//! A REST API server for controlling Nintendo Switch Pokemon bots.
//!
//! # Usage
//!
//! ```bash
//! # Start the server on default port 3000
//! cargo run -p sysbot_api
//!
//! # Start on a custom port
//! cargo run -p sysbot_api -- --port 8080
//! ```
//!
//! # API Endpoints
//!
//! See the routes module for full endpoint documentation.

use std::net::SocketAddr;

use sysbot_api::{create_router, AppState};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sysbot_api=debug,sysbot_base=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    let port = std::env::args()
        .skip_while(|arg| arg != "--port")
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000u16);

    let bind = std::env::args()
        .skip_while(|arg| arg != "--bind")
        .nth(1)
        .and_then(|b| b.parse().ok())
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], port)));

    // Create application state
    let state = AppState::new();

    // Create router with all endpoints
    let app = create_router(state);

    tracing::info!("Starting SysBot.rs API server on {}", bind);
    tracing::info!("Health check: http://{}/health", bind);
    tracing::info!("API base: http://{}/api/v1", bind);

    // Start server
    let listener = TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
