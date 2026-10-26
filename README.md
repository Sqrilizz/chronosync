# chronosync

Minimalist, ultra-light self-hosted developer time tracker with Bento Grid profiles.

`< 15MB RAM` &middot; `SQLite WAL` &middot; `Zero-config OAuth` &middot; `Bento UI`

---

## ⚡ Quick Start (Self-Host in 10 Seconds)

```bash
git clone https://github.com/Sqrilizz/chronosync.git
cd chronosync
docker compose up -d
```

Open `http://localhost:8080`:
1. Type your GitHub handle (e.g. `@sqrilizz`) &rarr; **Enter Dashboard**.
2. Copy your CLI config command with your pre-generated API token.
3. Your public Bento profile is immediately live at `http://localhost:8080/@your-handle`!

*No OAuth registration required. GitHub avatars are auto-fetched via `https://github.com/{username}.png`.*

---

## 🗂 Architecture

- **`chronosync-core`**: High-performance data models (`Heartbeat`, `StatsSummary`), language detection, Git branch resolving.
- **`chronosync-server`**: Axum 0.8 backend, SQLite in WAL mode, Bento Grid dashboard, public `/@username` profile system, dynamic SVG README badges, hardened with Cloudflare security standards (strict SQL isolation, IDOR protection, XSS escaping, strict CSP).
- **`chronosync-agent`**: Fast CLI binary for automatic heartbeats, status reports, and optional private commit syncing.

---

## 🖥 Web Views

- **`/`**: Minimalist dark card to enter your handle or paste an existing API token.
- **`/dashboard`**: Authenticated developer dashboard with 7d/30d/all-time stats, language distribution, projects breakdown, daily activity chart, and copyable CLI command.
- **`/@:username`**: Public Bento Grid developer profile showcasing verified coding time, languages, and live project activity.
- **`/api/v1/badge/{user}/{project}`**: Dynamic SVG badge for your GitHub repository `README.md`.

---

## 💻 CLI Usage

```bash
# Set your server and API token
export CHRONOSYNC_SERVER_URL="http://localhost:8080"
export CHRONOSYNC_API_KEY="chrono_your_token"

# Log activity (auto-detects project, branch, language)
chronosync log --file crates/chronosync-server/src/main.rs --write

# Log with automatic commit farming to private GitHub activity repo
chronosync log --file crates/chronosync-server/src/main.rs --git-sync

# Check coding stats from terminal
chronosync stats --days 7
```

---

## 🔒 Security Hardening

- Audited and verified with strict security guidelines.
- IDOR protected (`/api/v1/stats/summary` strictly scoped to authenticated user).
- Strict SQL isolation: every query is scoped by `user_id`.
- XML & HTML output escaping across SVG badges and Bento templates.
- Workstation paths sanitized (`sanitize_file_path`) to prevent local directory leakage on public profiles.
- Security headers enabled: `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, `Content-Security-Policy`.

---

## 🛠 Local Development

```bash
# Run tests
cargo test

# Run server locally
cargo run --bin chronosync-server
```
