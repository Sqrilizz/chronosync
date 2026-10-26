use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::{HeaderMap, HeaderValue, LOCATION, SET_COOKIE};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::config::Config;
use crate::db::Database;

#[derive(Clone)]
pub struct OAuthState {
    pub db: Database,
    pub config: Config,
    pub http_client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub error: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubTokenResponse {
    pub access_token: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

pub async fn github_login_handler(State(state): State<OAuthState>) -> Response {
    let client_id = match &state.config.github_client_id {
        Some(id) => id,
        None => return Redirect::to("/login?error=GitHub+OAuth+not+configured").into_response(),
    };

    let oauth_nonce = Uuid::new_v4().to_string();
    let redirect_uri = format!("{}/auth/github/callback", state.config.app_url);
    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user&state={}",
        client_id,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&oauth_nonce)
    );

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, HeaderValue::from_str(&url).unwrap());
    let cookie_val = format!(
        "oauth_state={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=300",
        oauth_nonce
    );
    headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie_val).unwrap());

    (StatusCode::SEE_OTHER, headers).into_response()
}

pub async fn github_callback_handler(
    State(state): State<OAuthState>,
    Query(query): Query<CallbackQuery>,
    headers_in: HeaderMap,
) -> Response {
    if let Some(err) = query.error {
        return Redirect::to(&format!("/login?error={}", urlencoding::encode(&err)))
            .into_response();
    }

    let expected_state = headers_in
        .get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            for part in cookies.split(';') {
                let part = part.trim();
                if let Some(val) = part.strip_prefix("oauth_state=") {
                    return Some(val.to_string());
                }
            }
            None
        });

    let state_param = query.state.as_deref().unwrap_or_default();
    if expected_state.as_deref() != Some(state_param) || state_param.is_empty() {
        return Redirect::to("/login?error=Invalid+or+expired+OAuth+state").into_response();
    }

    let code = match query.code {
        Some(c) => c,
        None => return Redirect::to("/login?error=Missing+code").into_response(),
    };

    let client_id = match &state.config.github_client_id {
        Some(id) => id,
        None => return Redirect::to("/login?error=Missing+client+id").into_response(),
    };

    let client_secret = match &state.config.github_client_secret {
        Some(sec) => sec,
        None => return Redirect::to("/login?error=Missing+client+secret").into_response(),
    };

    let mut form = HashMap::new();
    form.insert("client_id", client_id.as_str());
    form.insert("client_secret", client_secret.as_str());
    form.insert("code", code.as_str());

    let token_resp = match state
        .http_client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&form)
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => return Redirect::to("/login?error=Failed+to+exchange+code").into_response(),
    };

    let token_data: GitHubTokenResponse = match token_resp.json().await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login?error=Invalid+token+response").into_response(),
    };

    let access_token = match token_data.access_token {
        Some(t) => t,
        None => {
            let msg = token_data
                .error_description
                .unwrap_or_else(|| "Failed to obtain access token".to_string());
            return Redirect::to(&format!("/login?error={}", urlencoding::encode(&msg)))
                .into_response();
        }
    };

    let gh_user_resp = match state
        .http_client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "chronosync-server")
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Redirect::to("/login?error=Failed+to+fetch+user").into_response(),
    };

    let gh_user: GitHubUser = match gh_user_resp.json().await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/login?error=Invalid+user+response").into_response(),
    };

    let user = match state
        .db
        .get_or_create_github_user(
            gh_user.id,
            &gh_user.login,
            gh_user.name.as_deref(),
            gh_user.avatar_url.as_deref(),
        )
        .await
    {
        Ok(u) => u,
        Err(_) => return Redirect::to("/login?error=Database+error").into_response(),
    };

    let session_token = match state.db.create_session(user.id).await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login?error=Session+error").into_response(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, HeaderValue::from_static("/dashboard"));
    let cookie_val = format!(
        "session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000",
        session_token
    );
    headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie_val).unwrap());
    headers.append(
        SET_COOKIE,
        HeaderValue::from_static("oauth_state=; Path=/; HttpOnly; Max-Age=0"),
    );

    (StatusCode::SEE_OTHER, headers).into_response()
}

pub async fn demo_login_handler(State(state): State<OAuthState>) -> Response {
    let user = match state.db.get_or_create_demo_user().await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/login?error=Failed+to+create+demo+user").into_response(),
    };

    let session_token = match state.db.create_session(user.id).await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login?error=Session+error").into_response(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, HeaderValue::from_static("/dashboard"));
    let cookie_val = format!(
        "session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000",
        session_token
    );
    headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie_val).unwrap());

    (StatusCode::SEE_OTHER, headers).into_response()
}

pub async fn logout_handler(State(state): State<OAuthState>, headers_in: HeaderMap) -> Response {
    if let Some(cookie_header) = headers_in.get(axum::http::header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for part in cookie_str.split(';') {
                let part = part.trim();
                if let Some(token) = part.strip_prefix("session=") {
                    let _ = state.db.delete_session(token).await;
                }
            }
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, HeaderValue::from_static("/login"));
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_static("session=; Path=/; HttpOnly; Max-Age=0"),
    );

    (StatusCode::SEE_OTHER, headers).into_response()
}
