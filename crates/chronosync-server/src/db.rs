use std::collections::BTreeMap;
use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use chronosync_core::{DailySummary, Heartbeat, LanguageSummary, ProjectSummary, StatsSummary};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub github_id: Option<i64>,
    pub username: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub api_key: String,
    pub created_at: i64,
}

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await?;

        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                github_id INTEGER UNIQUE,
                username TEXT NOT NULL,
                name TEXT,
                avatar_url TEXT,
                api_key TEXT UNIQUE NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES users(id),
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS heartbeats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER,
                project TEXT NOT NULL,
                branch TEXT,
                language TEXT,
                file TEXT,
                editor TEXT,
                timestamp INTEGER NOT NULL,
                is_write INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_heartbeats_project_ts ON heartbeats(project, timestamp);
            CREATE INDEX IF NOT EXISTS idx_heartbeats_user_ts ON heartbeats(user_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_heartbeats_ts ON heartbeats(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        let has_user_id = sqlx::query("PRAGMA table_info(heartbeats)")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .any(|r| r.get::<String, _>("name") == "user_id");

        if !has_user_id {
            let _ = sqlx::query("ALTER TABLE heartbeats ADD COLUMN user_id INTEGER")
                .execute(&self.pool)
                .await;
        }

        Ok(())
    }

    pub async fn get_or_create_github_user(
        &self,
        github_id: i64,
        username: &str,
        name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<User> {
        let existing = sqlx::query(
            r#"
            SELECT id, github_id, username, name, avatar_url, api_key, created_at
            FROM users
            WHERE github_id = ?1
            "#,
        )
        .bind(github_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = existing {
            return Ok(User {
                id: r.get("id"),
                github_id: r.get("github_id"),
                username: r.get("username"),
                name: r.get("name"),
                avatar_url: r.get("avatar_url"),
                api_key: r.get("api_key"),
                created_at: r.get("created_at"),
            });
        }

        let api_key = format!("chrono_{}", Uuid::new_v4().simple());
        let now = chrono::Utc::now().timestamp();

        let id = sqlx::query(
            r#"
            INSERT INTO users (github_id, username, name, avatar_url, api_key, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(github_id)
        .bind(username)
        .bind(name)
        .bind(avatar_url)
        .bind(&api_key)
        .bind(now)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(User {
            id,
            github_id: Some(github_id),
            username: username.to_string(),
            name: name.map(ToString::to_string),
            avatar_url: avatar_url.map(ToString::to_string),
            api_key,
            created_at: now,
        })
    }

    pub async fn get_or_create_user_by_username(&self, username: &str) -> Result<User> {
        let clean = username.trim().trim_start_matches('@');
        if let Some(existing) = self.get_user_by_username(clean).await? {
            return Ok(existing);
        }

        let api_key = format!("chrono_{}", Uuid::new_v4().simple());
        let now = chrono::Utc::now().timestamp();
        let avatar_url = format!("https://github.com/{clean}.png");

        let id = sqlx::query(
            r#"
            INSERT INTO users (username, name, avatar_url, api_key, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(clean)
        .bind(clean)
        .bind(&avatar_url)
        .bind(&api_key)
        .bind(now)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(User {
            id,
            github_id: None,
            username: clean.to_string(),
            name: Some(clean.to_string()),
            avatar_url: Some(avatar_url),
            api_key,
            created_at: now,
        })
    }

    pub async fn get_or_create_demo_user(&self) -> Result<User> {
        let demo_github_id = 99999999i64;
        self.get_or_create_github_user(
            demo_github_id,
            "geek_dev",
            Some("Geek Developer"),
            Some("https://avatars.githubusercontent.com/u/9919?v=4"),
        )
        .await
    }

    pub async fn create_session(&self, user_id: i64) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + (30 * 86400);

        sqlx::query(
            r#"
            INSERT INTO sessions (token, user_id, created_at, expires_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(&token)
        .bind(user_id)
        .bind(now)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(token)
    }

    pub async fn get_user_by_session(&self, token: &str) -> Result<Option<User>> {
        let now = chrono::Utc::now().timestamp();
        let row = sqlx::query(
            r#"
            SELECT u.id, u.github_id, u.username, u.name, u.avatar_url, u.api_key, u.created_at
            FROM sessions s
            JOIN users u ON s.user_id = u.id
            WHERE s.token = ?1 AND s.expires_at > ?2
            "#,
        )
        .bind(token)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            github_id: r.get("github_id"),
            username: r.get("username"),
            name: r.get("name"),
            avatar_url: r.get("avatar_url"),
            api_key: r.get("api_key"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn get_user_by_api_key(&self, key: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, github_id, username, name, avatar_url, api_key, created_at
            FROM users
            WHERE api_key = ?1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            github_id: r.get("github_id"),
            username: r.get("username"),
            name: r.get("name"),
            avatar_url: r.get("avatar_url"),
            api_key: r.get("api_key"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let clean = username.trim().trim_start_matches('@');
        let row = sqlx::query(
            r#"
            SELECT id, github_id, username, name, avatar_url, api_key, created_at
            FROM users
            WHERE LOWER(username) = LOWER(?1)
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(clean)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            github_id: r.get("github_id"),
            username: r.get("username"),
            name: r.get("name"),
            avatar_url: r.get("avatar_url"),
            api_key: r.get("api_key"),
            created_at: r.get("created_at"),
        }))
    }

    pub async fn regenerate_api_key(&self, user_id: i64) -> Result<String> {
        let new_key = format!("chrono_{}", Uuid::new_v4().simple());
        sqlx::query(
            r#"
            UPDATE users
            SET api_key = ?1
            WHERE id = ?2
            "#,
        )
        .bind(&new_key)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(new_key)
    }

    pub async fn delete_session(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE token = ?1")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_heartbeats(
        &self,
        heartbeats: &[Heartbeat],
        user_id: Option<i64>,
    ) -> Result<usize> {
        if heartbeats.is_empty() {
            return Ok(0);
        }

        let now = chrono::Utc::now().timestamp();
        let mut tx = self.pool.begin().await?;

        for hb in heartbeats {
            sqlx::query(
                r#"
                INSERT INTO heartbeats (user_id, project, branch, language, file, editor, timestamp, is_write, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
            )
            .bind(user_id)
            .bind(&hb.project)
            .bind(&hb.branch)
            .bind(&hb.language)
            .bind(&hb.file)
            .bind(&hb.editor)
            .bind(hb.timestamp)
            .bind(if hb.is_write { 1 } else { 0 })
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(heartbeats.len())
    }

    pub async fn get_summary(
        &self,
        user_id: Option<i64>,
        days: Option<u32>,
    ) -> Result<StatsSummary> {
        let since_ts = match days {
            Some(d) => chrono::Utc::now().timestamp() - (d as i64 * 86400),
            None => 0,
        };

        let rows = match user_id {
            Some(uid) => {
                sqlx::query(
                    r#"
                    SELECT project, language, timestamp
                    FROM heartbeats
                    WHERE timestamp >= ?1 AND user_id = ?2
                    ORDER BY timestamp ASC
                    "#,
                )
                .bind(since_ts)
                .bind(uid)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                return Ok(StatsSummary {
                    total_seconds: 0,
                    projects: Vec::new(),
                    languages: Vec::new(),
                    daily_activity: Vec::new(),
                });
            }
        };

        if rows.is_empty() {
            return Ok(StatsSummary {
                total_seconds: 0,
                projects: Vec::new(),
                languages: Vec::new(),
                daily_activity: Vec::new(),
            });
        }

        struct Item {
            project: String,
            language: Option<String>,
            timestamp: i64,
        }

        let items: Vec<Item> = rows
            .into_iter()
            .map(|r| Item {
                project: r.get::<String, _>("project"),
                language: r.get::<Option<String>, _>("language"),
                timestamp: r.get::<i64, _>("timestamp"),
            })
            .collect();

        let mut project_seconds: HashMap<String, u64> = HashMap::new();
        let mut language_seconds: HashMap<String, u64> = HashMap::new();
        let mut daily_seconds: BTreeMap<String, u64> = BTreeMap::new();
        let mut total_seconds: u64 = 0;

        let max_gap: i64 = 120;
        let base_heartbeat_duration: u64 = 30;

        let mut prev_ts_per_project: HashMap<String, i64> = HashMap::new();

        for item in &items {
            let duration = match prev_ts_per_project.get(&item.project) {
                Some(&prev) => {
                    let diff = item.timestamp - prev;
                    if diff > 0 && diff <= max_gap {
                        diff as u64
                    } else if diff <= 0 {
                        0
                    } else {
                        base_heartbeat_duration
                    }
                }
                None => base_heartbeat_duration,
            };

            prev_ts_per_project.insert(item.project.clone(), item.timestamp);
            total_seconds += duration;

            *project_seconds.entry(item.project.clone()).or_insert(0) += duration;

            let lang = item.language.clone().unwrap_or_else(|| "Other".to_string());
            *language_seconds.entry(lang).or_insert(0) += duration;

            let date_str = match Utc.timestamp_opt(item.timestamp, 0) {
                chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d").to_string(),
                _ => "Unknown".to_string(),
            };
            *daily_seconds.entry(date_str).or_insert(0) += duration;
        }

        let mut projects: Vec<ProjectSummary> = project_seconds
            .into_iter()
            .map(|(name, sec)| {
                let percent = if total_seconds > 0 {
                    (sec as f64 / total_seconds as f64) * 100.0
                } else {
                    0.0
                };
                ProjectSummary {
                    name,
                    total_seconds: sec,
                    percent: (percent * 100.0).round() / 100.0,
                }
            })
            .collect();
        projects.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

        let mut languages: Vec<LanguageSummary> = language_seconds
            .into_iter()
            .map(|(name, sec)| {
                let percent = if total_seconds > 0 {
                    (sec as f64 / total_seconds as f64) * 100.0
                } else {
                    0.0
                };
                LanguageSummary {
                    name,
                    total_seconds: sec,
                    percent: (percent * 100.0).round() / 100.0,
                }
            })
            .collect();
        languages.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

        let daily_activity: Vec<DailySummary> = daily_seconds
            .into_iter()
            .map(|(date, sec)| DailySummary {
                date,
                total_seconds: sec,
            })
            .collect();

        Ok(StatsSummary {
            total_seconds,
            projects,
            languages,
            daily_activity,
        })
    }

    pub async fn get_project_total_seconds(
        &self,
        user_id: Option<i64>,
        project: &str,
    ) -> Result<u64> {
        let rows = match user_id {
            Some(uid) => {
                sqlx::query(
                    r#"
                    SELECT timestamp
                    FROM heartbeats
                    WHERE project = ?1 AND user_id = ?2
                    ORDER BY timestamp ASC
                    "#,
                )
                .bind(project)
                .bind(uid)
                .fetch_all(&self.pool)
                .await?
            }
            None => return Ok(0),
        };

        if rows.is_empty() {
            return Ok(0);
        }

        let max_gap: i64 = 120;
        let base_heartbeat_duration: u64 = 30;
        let mut total = 0u64;
        let mut prev_ts: Option<i64> = None;

        for r in rows {
            let ts: i64 = r.get("timestamp");
            let duration = match prev_ts {
                Some(prev) => {
                    let diff = ts - prev;
                    if diff > 0 && diff <= max_gap {
                        diff as u64
                    } else if diff <= 0 {
                        0
                    } else {
                        base_heartbeat_duration
                    }
                }
                None => base_heartbeat_duration,
            };
            total += duration;
            prev_ts = Some(ts);
        }

        Ok(total)
    }

    pub async fn get_recent_heartbeats(
        &self,
        user_id: Option<i64>,
        limit: i64,
    ) -> Result<Vec<Heartbeat>> {
        let rows = match user_id {
            Some(uid) => {
                sqlx::query(
                    r#"
                    SELECT project, branch, language, file, editor, timestamp, is_write
                    FROM heartbeats
                    WHERE user_id = ?1
                    ORDER BY timestamp DESC
                    LIMIT ?2
                    "#,
                )
                .bind(uid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => return Ok(Vec::new()),
        };

        Ok(rows
            .into_iter()
            .map(|r| Heartbeat {
                project: r.get("project"),
                branch: r.get("branch"),
                language: r.get("language"),
                file: r.get("file"),
                editor: r.get("editor"),
                timestamp: r.get("timestamp"),
                is_write: r.get::<i64, _>("is_write") == 1,
            })
            .collect())
    }
}
