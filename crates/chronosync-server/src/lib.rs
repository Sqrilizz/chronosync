pub mod auth;
pub mod config;
pub mod db;
pub mod handlers;
pub mod oauth;
pub mod web;

use axum::Router;
use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::decompression::RequestDecompressionLayer;
use tower_http::trace::TraceLayer;

use crate::auth::{AuthState, require_auth};
use crate::config::Config;
use crate::db::Database;
use crate::handlers::{
    AppState, badge_project_handler, badge_user_project_handler, dashboard_handler, health_handler,
    heartbeat_handler, index_handler, login_action_handler, login_handler, profile_at_handler,
    profile_user_handler, summary_handler,
};
use crate::oauth::{
    OAuthState, demo_login_handler, github_callback_handler, github_login_handler, logout_handler,
};

async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(
        axum::http::header::HeaderName::from_static("x-content-type-options"),
        axum::http::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        axum::http::header::HeaderName::from_static("x-frame-options"),
        axum::http::HeaderValue::from_static("DENY"),
    );
    headers.insert(
        axum::http::header::HeaderName::from_static("referrer-policy"),
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        axum::http::header::HeaderName::from_static("content-security-policy"),
        axum::http::HeaderValue::from_static("default-src 'self' https://cdn.tailwindcss.com https://fonts.googleapis.com https://fonts.gstatic.com https://avatars.githubusercontent.com; img-src 'self' data: https://avatars.githubusercontent.com https://github.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com; frame-ancestors 'none';"),
    );
    response
}

pub fn create_app(database: Database, config: Config) -> Router {
    let app_state = AppState {
        db: database.clone(),
        config: config.clone(),
    };

    let auth_state = AuthState {
        db: database.clone(),
        api_key: config.api_key.clone(),
    };

    let oauth_state = OAuthState {
        db: database,
        config,
        http_client: reqwest::Client::new(),
    };

    let protected_routes = Router::new()
        .route("/api/v1/heartbeat", post(heartbeat_handler))
        .route_layer(middleware::from_fn_with_state(auth_state, require_auth));

    let public_routes = Router::new()
        .route("/", get(index_handler))
        .route("/login", get(login_handler))
        .route("/auth/login", post(login_action_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/health", get(health_handler))
        .route("/@{username}", get(profile_at_handler))
        .route("/u/{username}", get(profile_user_handler))
        .route("/api/v1/stats/summary", get(summary_handler))
        .route("/api/v1/badge/{project}", get(badge_project_handler))
        .route(
            "/api/v1/badge/{user}/{project}",
            get(badge_user_project_handler),
        );

    let auth_routes = Router::new()
        .route("/auth/github", get(github_login_handler))
        .route("/auth/github/callback", get(github_callback_handler))
        .route("/auth/demo", get(demo_login_handler))
        .route("/logout", get(logout_handler))
        .with_state(oauth_state);

    Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .with_state(app_state)
        .merge(auth_routes)
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(RequestDecompressionLayer::new())
}
