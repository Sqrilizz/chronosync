use axum::extract::{Request, State};
use axum::http::header::{AUTHORIZATION, COOKIE};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use crate::db::{Database, User};

#[derive(Clone)]
pub struct AuthState {
    pub db: Database,
    pub api_key: Option<String>,
}

#[derive(Clone)]
pub struct AuthUser(pub Option<User>);

pub async fn require_auth(
    State(auth_state): State<AuthState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => Some(&h["Bearer ".len()..]),
        _ => None,
    };

    if let Some(t) = token {
        if let Ok(Some(user)) = auth_state.db.get_user_by_api_key(t).await {
            req.extensions_mut().insert(AuthUser(Some(user)));
            return Ok(next.run(req).await);
        }

        if let Some(expected_key) = &auth_state.api_key {
            if t == expected_key {
                req.extensions_mut().insert(AuthUser(None));
                return Ok(next.run(req).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn extract_session_user(headers: &HeaderMap, db: &Database) -> Option<User> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(token) = part.strip_prefix("session=") {
            if let Ok(Some(user)) = db.get_user_by_session(token).await {
                return Some(user);
            }
        }
    }
    None
}
