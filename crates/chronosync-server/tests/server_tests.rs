use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chronosync_core::{Heartbeat, IngestResponse, StatsSummary};
use chronosync_server::config::Config;
use chronosync_server::create_app;
use chronosync_server::db::Database;
use http_body_util::BodyExt;
use tower::ServiceExt;

fn test_config(api_key: Option<String>) -> Config {
    Config {
        host: "127.0.0.1".to_string(),
        port: 8080,
        database_url: "sqlite::memory:".to_string(),
        api_key,
        github_client_id: None,
        github_client_secret: None,
        app_url: "http://localhost:8080".to_string(),
    }
}

#[tokio::test]
async fn test_check_at_route() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let app = create_app(db.clone(), test_config(None));

    let _ = db
        .get_or_create_github_user(1, "sqrilizz", Some("Matvei"), None)
        .await
        .unwrap();

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/@sqrilizz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("@sqrilizz"));

    let res_unknown = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/@unknown_user")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_unknown.status(), StatusCode::NOT_FOUND);

    let res_bad = app
        .oneshot(
            Request::builder()
                .uri("/@bad..user!")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bad.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_health_endpoint() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let app = create_app(db, test_config(None));

    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body_str.contains("chronosync-server"));
}

#[tokio::test]
async fn test_heartbeat_ingest_and_summary() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let user = db
        .get_or_create_github_user(1, "sqrilizz", Some("Matvei"), None)
        .await
        .unwrap();
    let session = db.create_session(user.id).await.unwrap();
    let app = create_app(db, test_config(None));

    let hb1 = Heartbeat {
        project: "chronosync".to_string(),
        branch: Some("main".to_string()),
        language: Some("rust".to_string()),
        file: Some("src/main.rs".to_string()),
        editor: Some("zed".to_string()),
        timestamp: 1000,
        is_write: true,
    };

    let hb2 = Heartbeat {
        project: "chronosync".to_string(),
        branch: Some("main".to_string()),
        language: Some("rust".to_string()),
        file: Some("src/lib.rs".to_string()),
        editor: Some("zed".to_string()),
        timestamp: 1060,
        is_write: true,
    };

    let req_unauth = Request::builder()
        .method("POST")
        .uri("/api/v1/heartbeat")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&vec![hb1.clone(), hb2.clone()]).unwrap(),
        ))
        .unwrap();

    let res_unauth = app.clone().oneshot(req_unauth).await.unwrap();
    assert_eq!(res_unauth.status(), StatusCode::UNAUTHORIZED);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/heartbeat")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", user.api_key))
        .body(Body::from(serde_json::to_vec(&vec![hb1, hb2]).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let ingest_res: IngestResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(ingest_res.accepted, 2);

    let req_summary_unauth = Request::builder()
        .uri("/api/v1/stats/summary")
        .body(Body::empty())
        .unwrap();

    let res_summary_unauth = app.clone().oneshot(req_summary_unauth).await.unwrap();
    assert_eq!(res_summary_unauth.status(), StatusCode::UNAUTHORIZED);

    let req_summary = Request::builder()
        .uri("/api/v1/stats/summary")
        .header(header::COOKIE, format!("session={}", session))
        .body(Body::empty())
        .unwrap();

    let res_summary = app.oneshot(req_summary).await.unwrap();
    assert_eq!(res_summary.status(), StatusCode::OK);

    let bytes_summary = res_summary.into_body().collect().await.unwrap().to_bytes();
    let summary: StatsSummary = serde_json::from_slice(&bytes_summary).unwrap();
    assert_eq!(summary.total_seconds, 90);
    assert_eq!(summary.projects.len(), 1);
    assert_eq!(summary.projects[0].name, "chronosync");
    assert_eq!(summary.projects[0].total_seconds, 90);
}

#[tokio::test]
async fn test_auth_protection() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let app = create_app(db, test_config(Some("secret123".to_string())));

    let hb = Heartbeat {
        project: "chronosync".to_string(),
        branch: None,
        language: None,
        file: None,
        editor: None,
        timestamp: 1000,
        is_write: false,
    };

    let req_unauthorized = Request::builder()
        .method("POST")
        .uri("/api/v1/heartbeat")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&hb).unwrap()))
        .unwrap();

    let res_unauth = app.clone().oneshot(req_unauthorized).await.unwrap();
    assert_eq!(res_unauth.status(), StatusCode::UNAUTHORIZED);

    let req_authorized = Request::builder()
        .method("POST")
        .uri("/api/v1/heartbeat")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer secret123")
        .body(Body::from(serde_json::to_vec(&hb).unwrap()))
        .unwrap();

    let res_auth = app.oneshot(req_authorized).await.unwrap();
    assert_eq!(res_auth.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_badge_endpoint() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let app = create_app(db, test_config(None));

    let req = Request::builder()
        .uri("/api/v1/badge/chronosync")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let svg = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("chronosync"));
}

#[tokio::test]
async fn test_web_routes_and_self_hosted_login() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let app = create_app(db, test_config(None));

    let res_index = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res_index.status(), StatusCode::OK);
    let bytes = res_index.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("chronosync"));
    assert!(body.contains("self-hosted"));

    let res_what = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/what-we-collect")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_what.status(), StatusCode::NOT_FOUND);

    let res_docs = app
        .clone()
        .oneshot(Request::builder().uri("/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res_docs.status(), StatusCode::NOT_FOUND);

    let req_login = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("username=testdev"))
        .unwrap();

    let res_login = app.clone().oneshot(req_login).await.unwrap();
    assert_eq!(res_login.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        res_login.headers().get(header::LOCATION).unwrap(),
        "/dashboard"
    );
    let set_cookie = res_login
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(set_cookie.contains("session="));

    let res_profile = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/@testdev")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_profile.status(), StatusCode::OK);

    let cookie_val = set_cookie.split(';').next().unwrap();
    let res_dash = app
        .oneshot(
            Request::builder()
                .uri("/dashboard")
                .header(header::COOKIE, cookie_val)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_dash.status(), StatusCode::OK);
    let bytes_dash = res_dash.into_body().collect().await.unwrap().to_bytes();
    let html_dash = String::from_utf8(bytes_dash.to_vec()).unwrap();
    assert!(html_dash.contains("@testdev"));
}
