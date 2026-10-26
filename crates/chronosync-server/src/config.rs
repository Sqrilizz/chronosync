use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub api_key: Option<String>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub app_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://chronosync.db?mode=rwc".to_string());
        let api_key = env::var("CHRONOSYNC_API_KEY")
            .ok()
            .filter(|k| !k.is_empty());

        let github_client_id = env::var("GITHUB_CLIENT_ID").ok().filter(|k| !k.is_empty());
        let github_client_secret = env::var("GITHUB_CLIENT_SECRET")
            .ok()
            .filter(|k| !k.is_empty());

        let app_url = env::var("APP_URL").unwrap_or_else(|_| {
            if let Ok(railway_domain) = env::var("RAILWAY_PUBLIC_DOMAIN") {
                format!("https://{}", railway_domain)
            } else {
                format!("http://localhost:{}", port)
            }
        });

        Self {
            host,
            port,
            database_url,
            api_key,
            github_client_id,
            github_client_secret,
            app_url,
        }
    }
}
