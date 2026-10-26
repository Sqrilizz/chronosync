use axum::Json;
use axum::extract::{Extension, Form, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use chronosync_core::{HeartbeatPayload, IngestResponse};
use serde::Deserialize;

use crate::auth::{AuthUser, extract_session_user};
use crate::config::Config;
use crate::db::Database;
use crate::web::{
    render_dashboard, render_login_page, render_profile_not_found, render_profile_page,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Config,
}

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub days: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginActionForm {
    pub username: Option<String>,
    pub token: Option<String>,
}

fn is_valid_username(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 39
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn is_valid_project(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

pub async fn index_handler(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
    headers: HeaderMap,
) -> Response {
    if extract_session_user(&headers, &state.db).await.is_some() {
        return Redirect::to("/dashboard").into_response();
    }

    let html = render_login_page(
        state.config.github_client_id.as_deref(),
        query.error.as_deref(),
    );
    Html(html).into_response()
}

pub async fn login_handler(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
    headers: HeaderMap,
) -> Response {
    if extract_session_user(&headers, &state.db).await.is_some() {
        return Redirect::to("/dashboard").into_response();
    }

    let html = render_login_page(
        state.config.github_client_id.as_deref(),
        query.error.as_deref(),
    );
    Html(html).into_response()
}

pub async fn login_action_handler(
    State(state): State<AppState>,
    Form(form): Form<LoginActionForm>,
) -> Response {
    if let Some(token) = form
        .token
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let user = match state.db.get_user_by_api_key(token).await {
            Ok(Some(u)) => u,
            _ => return Redirect::to("/login?error=Invalid+API+key").into_response(),
        };

        let session_token = match state.db.create_session(user.id).await {
            Ok(t) => t,
            Err(_) => return Redirect::to("/login?error=Session+error").into_response(),
        };

        let mut headers = HeaderMap::new();
        headers.insert(header::LOCATION, HeaderValue::from_static("/dashboard"));
        let cookie_val = format!(
            "session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000",
            session_token
        );
        headers.insert(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie_val).unwrap(),
        );
        return (StatusCode::SEE_OTHER, headers).into_response();
    }

    if let Some(username) = form
        .username
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let clean = username.trim_start_matches('@');
        if !is_valid_username(clean) {
            return Redirect::to("/login?error=Invalid+username+format").into_response();
        }

        let user = match state.db.get_or_create_user_by_username(clean).await {
            Ok(u) => u,
            Err(_) => return Redirect::to("/login?error=Database+error").into_response(),
        };

        let session_token = match state.db.create_session(user.id).await {
            Ok(t) => t,
            Err(_) => return Redirect::to("/login?error=Session+error").into_response(),
        };

        let mut headers = HeaderMap::new();
        headers.insert(header::LOCATION, HeaderValue::from_static("/dashboard"));
        let cookie_val = format!(
            "session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000",
            session_token
        );
        headers.insert(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie_val).unwrap(),
        );
        return (StatusCode::SEE_OTHER, headers).into_response();
    }

    Redirect::to("/login?error=Please+enter+handle+or+token").into_response()
}

pub async fn dashboard_handler(
    State(state): State<AppState>,
    Query(query): Query<StatsQuery>,
    headers: HeaderMap,
) -> Response {
    let user = match extract_session_user(&headers, &state.db).await {
        Some(u) => u,
        None => return Redirect::to("/login").into_response(),
    };

    let bounded_days = query.days.map(|d| d.min(3650));

    let summary = state
        .db
        .get_summary(Some(user.id), bounded_days)
        .await
        .unwrap_or_else(|_| chronosync_core::StatsSummary {
            total_seconds: 0,
            projects: Vec::new(),
            languages: Vec::new(),
            daily_activity: Vec::new(),
        });

    let recent = state
        .db
        .get_recent_heartbeats(Some(user.id), 20)
        .await
        .unwrap_or_default();

    let html = render_dashboard(
        &user,
        &summary,
        &recent,
        &state.config.app_url,
        bounded_days,
    );

    Html(html).into_response()
}

pub async fn profile_at_handler(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<StatsQuery>,
    headers: HeaderMap,
) -> Response {
    profile_view(&state, &username, query.days, &headers).await
}

pub async fn profile_user_handler(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<StatsQuery>,
    headers: HeaderMap,
) -> Response {
    profile_view(&state, &username, query.days, &headers).await
}

async fn profile_view(
    state: &AppState,
    username: &str,
    days: Option<u32>,
    headers: &HeaderMap,
) -> Response {
    let clean = username.trim().trim_start_matches('@');
    if !is_valid_username(clean) {
        return (StatusCode::BAD_REQUEST, "Invalid username format").into_response();
    }

    let viewer = extract_session_user(headers, &state.db).await;
    let target_user = match state.db.get_user_by_username(clean).await {
        Ok(Some(u)) => u,
        _ => {
            let html = render_profile_not_found(clean, &state.config.app_url, viewer.as_ref());
            return (StatusCode::NOT_FOUND, Html(html)).into_response();
        }
    };

    let bounded_days = days.map(|d| d.min(3650));

    let summary = state
        .db
        .get_summary(Some(target_user.id), bounded_days)
        .await
        .unwrap_or_else(|_| chronosync_core::StatsSummary {
            total_seconds: 0,
            projects: Vec::new(),
            languages: Vec::new(),
            daily_activity: Vec::new(),
        });

    let recent = state
        .db
        .get_recent_heartbeats(Some(target_user.id), 20)
        .await
        .unwrap_or_default();

    let html = render_profile_page(
        &target_user,
        &summary,
        &recent,
        &state.config.app_url,
        bounded_days,
        viewer.as_ref(),
    );

    Html(html).into_response()
}

pub async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "chronosync-server"
    }))
}

pub async fn heartbeat_handler(
    State(state): State<AppState>,
    Extension(AuthUser(user)): Extension<AuthUser>,
    Json(payload): Json<HeartbeatPayload>,
) -> Result<Json<IngestResponse>, (StatusCode, String)> {
    let list = payload.into_vec();
    let total = list.len();

    let valid_heartbeats: Vec<_> = list
        .into_iter()
        .filter(|hb| {
            !hb.project.trim().is_empty() && hb.timestamp > 0 && is_valid_project(&hb.project)
        })
        .collect();

    let user_id = user.as_ref().map(|u| u.id);

    let accepted = state
        .db
        .insert_heartbeats(&valid_heartbeats, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(IngestResponse {
        status: "ok".to_string(),
        accepted,
        total,
    }))
}

pub async fn summary_handler(
    State(state): State<AppState>,
    Query(query): Query<StatsQuery>,
    headers: HeaderMap,
) -> Result<Json<chronosync_core::StatsSummary>, (StatusCode, String)> {
    let user = extract_session_user(&headers, &state.db).await;
    let user = match user {
        Some(u) => u,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ));
        }
    };

    let bounded_days = query.days.map(|d| d.min(3650));

    let summary = state
        .db
        .get_summary(Some(user.id), bounded_days)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(summary))
}

pub async fn badge_user_project_handler(
    State(state): State<AppState>,
    Path((user, project)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    let clean_user = user.trim().trim_start_matches('@');
    if !is_valid_username(clean_user) || !is_valid_project(&project) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user_id = state
        .db
        .get_user_by_username(clean_user)
        .await
        .ok()
        .flatten()
        .map(|u| u.id);
    render_project_badge(&state.db, user_id, &project).await
}

pub async fn badge_project_handler(
    State(state): State<AppState>,
    Path(project): Path<String>,
) -> Result<Response, StatusCode> {
    if !is_valid_project(&project) {
        return Err(StatusCode::BAD_REQUEST);
    }
    render_project_badge(&state.db, None, &project).await
}

async fn render_project_badge(
    db: &Database,
    user_id: Option<i64>,
    project: &str,
) -> Result<Response, StatusCode> {
    let total_seconds = db
        .get_project_total_seconds(user_id, project)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let time_str = format_duration(total_seconds);
    let svg = generate_svg_badge(project, &time_str);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "image/svg+xml; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CACHE_CONTROL,
        "no-cache, no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        header::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        header::HeaderValue::from_static("default-src 'none'"),
    );

    Ok((headers, svg).into_response())
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn escape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn generate_svg_badge(label: &str, value: &str) -> String {
    let safe_label = escape_xml(label);
    let safe_value = escape_xml(value);

    let label_width = (label.len() * 7 + 16).max(60);
    let value_width = (value.len() * 7 + 16).max(50);
    let total_width = label_width + value_width;

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_width}" height="20" role="img" aria-label="{safe_label}: {safe_value}">
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <clipPath id="r">
    <rect width="{total_width}" height="20" rx="3" fill="#fff"/>
  </clipPath>
  <g clip-path="url(#r)">
    <rect width="{label_width}" height="20" fill="#2d3748"/>
    <rect x="{label_width}" width="{value_width}" height="20" fill="#3182ce"/>
    <rect width="{total_width}" height="20" fill="url(#s)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="11">
    <text aria-hidden="true" x="{label_x}" y="15" fill="#010101" fill-opacity=".3">{safe_label}</text>
    <text x="{label_x}" y="14">{safe_label}</text>
    <text aria-hidden="true" x="{value_x}" y="15" fill="#010101" fill-opacity=".3">{safe_value}</text>
    <text x="{value_x}" y="14">{safe_value}</text>
  </g>
</svg>"##,
        total_width = total_width,
        label_width = label_width,
        value_width = value_width,
        safe_label = safe_label,
        safe_value = safe_value,
        label_x = label_width / 2,
        value_x = label_width + (value_width / 2),
    )
}
