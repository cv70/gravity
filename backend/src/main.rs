mod config;
mod datasource;
mod domain;
mod infra;
mod state;
mod utils;

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, patch, delete},
    middleware,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use state::{AppState, auth_middleware};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::AppConfig::load()?;
    let app_state = AppState::new(config.clone()).await?;

    tracing::info!("Starting server on {}:{}", config.server.host, config.server.port);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes - no auth required
    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/v1/auth/register", post(domain::auth::register))
        .route("/api/v1/auth/login", post(domain::auth::login))
        .route("/api/v1/auth/refresh", post(domain::auth::refresh));

    // Protected routes - require auth via middleware
    let protected_routes = Router::new()
        .route("/api/v1/contacts", get(domain::contact::list_contacts).post(domain::contact::create_contact))
        .route("/api/v1/contacts/{id}", get(domain::contact::get_contact).patch(domain::contact::update_contact).delete(domain::contact::delete_contact))
        .route("/api/v1/campaigns", get(domain::campaign::list_campaigns).post(domain::campaign::create_campaign))
        .route("/api/v1/campaigns/{id}", get(domain::campaign::get_campaign).patch(domain::campaign::update_campaign).delete(domain::campaign::delete_campaign))
        .route("/api/v1/campaigns/{id}/launch", post(domain::campaign::launch_campaign))
        .route("/api/v1/campaigns/{id}/pause", post(domain::campaign::pause_campaign))
        .route("/api/v1/contents", get(domain::content::list_contents).post(domain::content::create_content))
        .route("/api/v1/contents/{id}", get(domain::content::get_content).delete(domain::content::delete_content))
        .route("/api/v1/analytics/dashboard", get(domain::analytics::get_dashboard))
        .route("/api/v1/analytics/funnel", get(domain::analytics::get_funnel))
        .layer(middleware::from_fn_with_state(
            app_state.auth_service.clone(),
            auth_middleware,
        ));

    // Track routes - SDK integration (no JWT auth, uses API key or anonymous)
    let track_routes = Router::new()
        .route("/api/v1/track/event", post(domain::analytics::track_event))
        .route("/api/v1/track/identify", post(domain::analytics::identify))
        .route("/api/v1/track/page", post(domain::analytics::track_page))
        .route("/api/v1/track/conversion", post(domain::analytics::track_conversion));

    let app = public_routes
        .merge(protected_routes)
        .merge(track_routes)
        .layer(cors)
        .with_state(Arc::new(app_state));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}