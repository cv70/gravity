mod config;
mod datasource;
mod domain;
mod infra;
mod state;
mod utils;

use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use state::{auth_middleware, AppState};

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
    domain::automation::worker::AutomationWorker::new(app_state.registry.db_dao.clone()).start();

    tracing::info!(
        "Starting server on {}:{}",
        config.server.host,
        config.server.port
    );

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes - no auth required
    let public_routes = Router::new()
        .route(
            "/health",
            get(|| async { axum::Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/api/v1/auth/register", post(domain::auth::register))
        .route("/api/v1/auth/login", post(domain::auth::login))
        .route("/api/v1/auth/refresh", post(domain::auth::refresh));

    // Protected routes - require auth via middleware
    let protected_routes = Router::new()
        .route(
            "/api/v1/contacts",
            get(domain::contact::list_contacts).post(domain::contact::create_contact),
        )
        .route(
            "/api/v1/contacts/{id}",
            get(domain::contact::get_contact)
                .patch(domain::contact::update_contact)
                .delete(domain::contact::delete_contact),
        )
        .route(
            "/api/v1/campaigns",
            get(domain::campaign::list_campaigns).post(domain::campaign::create_campaign),
        )
        .route(
            "/api/v1/campaigns/{id}",
            get(domain::campaign::get_campaign)
                .patch(domain::campaign::update_campaign)
                .delete(domain::campaign::delete_campaign),
        )
        .route(
            "/api/v1/campaigns/{id}/launch",
            post(domain::campaign::launch_campaign),
        )
        .route(
            "/api/v1/campaigns/{id}/pause",
            post(domain::campaign::pause_campaign),
        )
        .route(
            "/api/v1/channels",
            get(domain::channel::list_channels).post(domain::channel::create_channel),
        )
        .route(
            "/api/v1/channels/{id}",
            patch(domain::channel::update_channel).delete(domain::channel::delete_channel),
        )
        .route(
            "/api/v1/contents",
            get(domain::content::list_contents).post(domain::content::create_content),
        )
        .route(
            "/api/v1/contents/{id}",
            get(domain::content::get_content).delete(domain::content::delete_content),
        )
        .route(
            "/api/v1/segments",
            get(domain::segment::list_segments).post(domain::segment::create_segment),
        )
        .route(
            "/api/v1/segments/{id}",
            get(domain::segment::get_segment)
                .patch(domain::segment::update_segment)
                .delete(domain::segment::delete_segment),
        )
        .route(
            "/api/v1/segments/{id}/preview",
            post(domain::segment::preview_segment),
        )
        .route(
            "/api/v1/workflows",
            get(domain::workflow::list_workflows).post(domain::workflow::create_workflow),
        )
        .route(
            "/api/v1/workflows/{id}",
            get(domain::workflow::get_workflow)
                .patch(domain::workflow::update_workflow)
                .delete(domain::workflow::delete_workflow),
        )
        .route(
            "/api/v1/workflows/{id}/activate",
            post(domain::workflow::activate_workflow),
        )
        .route(
            "/api/v1/workflows/{id}/deactivate",
            post(domain::workflow::deactivate_workflow),
        )
        .route(
            "/api/v1/workflows/{id}/executions",
            get(domain::workflow::list_executions),
        )
        .route(
            "/api/v1/workflows/{workflow_id}/executions/{execution_id}",
            get(domain::workflow::get_execution),
        )
        .route(
            "/api/v1/workflows/{id}/test",
            post(domain::workflow::test_workflow),
        )
        .route(
            "/api/v1/approvals",
            get(domain::approval::list_approvals).post(domain::approval::create_approval),
        )
        .route(
            "/api/v1/approvals/{id}",
            patch(domain::approval::update_approval),
        )
        .route(
            "/api/v1/audit-logs",
            get(domain::audit::list_audit_logs),
        )
        .route(
            "/api/v1/automation/dashboard",
            get(domain::automation::get_dashboard),
        )
        .route(
            "/api/v1/automation/jobs",
            get(domain::automation::list_jobs).post(domain::automation::create_job),
        )
        .route(
            "/api/v1/automation/jobs/{id}/execute",
            post(domain::automation::execute_job),
        )
        .route(
            "/api/v1/automation/runs",
            get(domain::automation::list_runs),
        )
        .route(
            "/api/v1/automation/actions",
            get(domain::automation::list_actions),
        )
        .route(
            "/api/v1/automation/approvals",
            get(domain::automation::list_approvals),
        )
        .route(
            "/api/v1/automation/approvals/{id}/decision",
            post(domain::automation::review_approval),
        )
        .route(
            "/api/v1/automation/policies",
            get(domain::automation::list_policies).post(domain::automation::create_policy),
        )
        .route(
            "/api/v1/automation/bootstrap",
            post(domain::automation::bootstrap_defaults),
        )
        .route(
            "/api/v1/automation/experiments",
            get(domain::automation::list_experiments),
        )
        .route(
            "/api/v1/analytics/dashboard",
            get(domain::analytics::get_dashboard),
        )
        .route(
            "/api/v1/analytics/funnel",
            get(domain::analytics::get_funnel),
        )
        .route(
            "/api/v1/insights/recommendations",
            get(domain::insights::get_recommendations),
        )
        .route(
            "/api/v1/insights/anomalies",
            get(domain::insights::get_anomalies),
        )
        .route(
            "/api/v1/insights/opportunities",
            get(domain::insights::get_opportunities),
        )
        .route("/api/v1/auth/me", get(domain::auth::me))
        .layer(middleware::from_fn_with_state(
            app_state.auth_service.clone(),
            auth_middleware,
        ));

    // Track routes - SDK integration (no JWT auth, uses API key or anonymous)
    let track_routes = Router::new()
        .route("/api/v1/track/event", post(domain::analytics::track_event))
        .route("/api/v1/track/identify", post(domain::analytics::identify))
        .route("/api/v1/track/page", post(domain::analytics::track_page))
        .route(
            "/api/v1/track/conversion",
            post(domain::analytics::track_conversion),
        );

    let app = public_routes
        .merge(protected_routes)
        .merge(track_routes)
        .layer(cors)
        .with_state(Arc::new(app_state));

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
            .await?;
    axum::serve(listener, app).await?;

    Ok(())
}
