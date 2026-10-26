use chronosync_core::{Heartbeat, StatsSummary};

use crate::db::User;

pub fn render_login_page(github_client_id: Option<&str>, error: Option<&str>) -> String {
    let error_html = error
        .map(|err| {
            format!(
                r#"<div class="p-3 rounded-xl border border-red-900/50 bg-red-950/20 text-xs text-red-300 font-mono">{}</div>"#,
                escape_html(err)
            )
        })
        .unwrap_or_default();

    let github_button = match github_client_id {
        Some(_) => {
            r#"
        <div class="relative flex py-1 items-center">
            <div class="flex-grow border-t border-zinc-800/80"></div>
            <span class="flex-shrink mx-2 text-[10px] text-zinc-600 font-mono">or OAuth</span>
            <div class="flex-grow border-t border-zinc-800/80"></div>
        </div>
        <a href="/auth/github" class="w-full flex items-center justify-center gap-2 py-2.5 px-4 bg-zinc-900 hover:bg-zinc-800 border border-zinc-700/60 text-zinc-200 rounded-xl font-mono text-xs transition-colors">
            <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
                <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"></path>
            </svg>
            <span>GitHub OAuth</span>
        </a>
        "#
        }
        None => "",
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>chronosync &mdash; self-hosted time tracker</title>
    <meta name="theme-color" content="#8b5cf6">
    <meta property="og:site_name" content="chronosync">
    <meta property="og:type" content="website">
    <meta property="og:title" content="chronosync &mdash; self-hosted developer time tracker">
    <meta property="og:description" content="Minimalist developer activity &amp; time tracker with Bento Grid profiles, SQLite WAL, and dynamic SVG badges.">
    <meta property="og:image" content="https://github.com/Sqrilizz.png">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="chronosync &mdash; self-hosted developer time tracker">
    <meta name="twitter:description" content="Minimalist developer activity &amp; time tracker with Bento Grid profiles, SQLite WAL, and dynamic SVG badges.">
    <meta name="twitter:image" content="https://github.com/Sqrilizz.png">
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        body {{ font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif; }}
        code, pre, .font-mono {{ font-family: 'JetBrains Mono', monospace; }}
    </style>
</head>
<body class="bg-[#09090b] text-zinc-200 min-h-screen flex items-center justify-center p-4 antialiased bg-[linear-gradient(rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:48px_48px]">
    <div class="w-full max-w-sm rounded-2xl bg-zinc-900/40 border border-zinc-800/80 p-7 space-y-5 shadow-2xl backdrop-blur-md">
        <div>
            <div class="flex items-center gap-2 mb-2">
                <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
                <span class="text-xs text-zinc-500 font-mono">self-hosted tracker</span>
            </div>
            <h1 class="text-2xl font-bold tracking-tight text-white font-mono">&gt; chronosync</h1>
            <p class="text-xs text-zinc-400 font-mono mt-1">developer activity &amp; time tracker</p>
        </div>

        {error_html}

        <form method="POST" action="/auth/login" class="space-y-3">
            <div>
                <label class="block text-[11px] text-zinc-400 font-mono mb-1.5 font-medium">GitHub handle</label>
                <div class="relative">
                    <span class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 font-mono text-xs">@</span>
                    <input type="text" name="username" required placeholder="your-handle" autofocus
                        class="w-full bg-zinc-950 border border-zinc-800 rounded-xl pl-8 pr-3 py-2.5 text-xs font-mono text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-violet-500 transition-colors" />
                </div>
            </div>
            <button type="submit" class="w-full py-2.5 px-4 bg-white hover:bg-zinc-200 text-zinc-950 rounded-xl font-mono text-xs font-semibold transition-colors shadow-sm flex items-center justify-center gap-1.5">
                <span>Enter Dashboard</span>
                <span>&rarr;</span>
            </button>
        </form>

        <form method="POST" action="/auth/login" class="pt-3 border-t border-zinc-800/60 space-y-2">
            <label class="block text-[11px] text-zinc-500 font-mono">Or paste API key</label>
            <div class="flex gap-2">
                <input type="password" name="token" placeholder="chrono_..." required
                    class="flex-1 bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs font-mono text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-violet-500 transition-colors" />
                <button type="submit" class="px-3.5 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded-xl font-mono text-xs font-medium transition-colors">
                    Sign in
                </button>
            </div>
        </form>

        {github_button}

        <div class="pt-3 border-t border-zinc-800/60 text-xs font-mono flex items-center justify-between">
            <a href="/@sqrilizz" class="text-zinc-400 hover:text-white transition-colors">Demo: @sqrilizz &rarr;</a>
            <a href="/auth/demo" class="text-zinc-500 hover:text-zinc-300 transition-colors">demo mode</a>
        </div>

        <div class="pt-2 text-[11px] text-zinc-600 font-mono flex items-center justify-between">
            <span>&lt; 15MB RAM</span>
            <span class="text-zinc-800">&middot;</span>
            <span>SQLite WAL</span>
            <span class="text-zinc-800">&middot;</span>
            <span>zero bloat</span>
        </div>
    </div>
</body>
</html>"##,
        github_button = github_button,
        error_html = error_html
    )
}

pub fn render_dashboard(
    user: &User,
    summary: &StatsSummary,
    recent_heartbeats: &[Heartbeat],
    app_url: &str,
    days: Option<u32>,
) -> String {
    let days_param = days.unwrap_or(7);
    let total_time_str = format_duration(summary.total_seconds);
    let projects_count = summary.projects.len();
    let top_language = summary
        .languages
        .first()
        .map(|l| escape_html(&l.name))
        .unwrap_or_else(|| "none".to_string());

    let daily_avg = if !summary.daily_activity.is_empty() {
        summary.total_seconds / (summary.daily_activity.len() as u64)
    } else {
        0
    };
    let daily_avg_str = format_duration(daily_avg);

    let max_daily_seconds = summary
        .daily_activity
        .iter()
        .map(|d| d.total_seconds)
        .max()
        .unwrap_or(1)
        .max(1);

    let mut bento_chart_bars = String::new();
    if summary.daily_activity.is_empty() {
        bento_chart_bars.push_str(r#"<div class="h-36 flex items-center justify-center text-xs text-zinc-600 font-mono">No activity recorded for this period</div>"#);
    } else {
        bento_chart_bars.push_str(r#"<div class="h-44 flex items-end gap-2.5 pt-6">"#);
        for d in &summary.daily_activity {
            let height_pct = ((d.total_seconds as f64 / max_daily_seconds as f64) * 100.0).max(8.0);
            let time_formatted = format_duration(d.total_seconds);
            let safe_date = escape_html(&d.date);
            let day_label = if safe_date.len() >= 10 {
                &safe_date[5..]
            } else {
                &safe_date
            };
            bento_chart_bars.push_str(&format!(
                r#"<div class="flex-1 flex flex-col items-center h-full justify-end group relative cursor-default">
                    <div class="absolute -top-7 opacity-0 group-hover:opacity-100 transition-opacity bg-zinc-800 border border-zinc-700 text-[11px] font-mono px-2 py-0.5 rounded-md shadow whitespace-nowrap z-20 pointer-events-none">
                        {}: {}
                    </div>
                    <div class="w-full bg-violet-500/20 group-hover:bg-violet-500/50 border-t-2 border-violet-400 transition-all rounded-t-sm" style="height: {:.1}%"></div>
                    <span class="text-[10px] text-zinc-500 font-mono mt-2 truncate">{}</span>
                </div>"#,
                safe_date, time_formatted, height_pct, day_label
            ));
        }
        bento_chart_bars.push_str("</div>");
    }

    let mut bento_project_rows = String::new();
    if summary.projects.is_empty() {
        bento_project_rows.push_str(r#"<div class="py-4 text-center text-xs text-zinc-600 font-mono">No projects recorded</div>"#);
    } else {
        for p in &summary.projects {
            let safe_p_name = escape_html(&p.name);
            bento_project_rows.push_str(&format!(
                r#"<div class="relative py-2.5 px-3 rounded-xl hover:bg-zinc-800/40 flex items-center justify-between text-xs transition-colors overflow-hidden group">
                    <div class="absolute inset-y-0 left-0 bg-violet-500/10 group-hover:bg-violet-500/20 rounded-xl transition-colors" style="width: {:.1}%"></div>
                    <span class="relative z-10 text-zinc-200 font-mono truncate font-medium">{}</span>
                    <div class="relative z-10 flex items-center gap-2 font-mono text-[11px]">
                        <span class="text-zinc-300 font-medium">{}</span>
                        <span class="text-zinc-500 w-12 text-right">{:.1}%</span>
                    </div>
                </div>"#,
                p.percent, safe_p_name, format_duration(p.total_seconds), p.percent
            ));
        }
    }

    let mut bento_language_rows = String::new();
    if summary.languages.is_empty() {
        bento_language_rows.push_str(r#"<div class="py-4 text-center text-xs text-zinc-600 font-mono">No languages recorded</div>"#);
    } else {
        for l in &summary.languages {
            let safe_l_name = escape_html(&l.name);
            bento_language_rows.push_str(&format!(
                r#"<div class="relative py-2.5 px-3 rounded-xl hover:bg-zinc-800/40 flex items-center justify-between text-xs transition-colors overflow-hidden group">
                    <div class="absolute inset-y-0 left-0 bg-emerald-500/10 group-hover:bg-emerald-500/20 rounded-xl transition-colors" style="width: {:.1}%"></div>
                    <span class="relative z-10 text-zinc-200 font-mono truncate font-medium">{}</span>
                    <div class="relative z-10 flex items-center gap-2 font-mono text-[11px]">
                        <span class="text-zinc-300 font-medium">{}</span>
                        <span class="text-zinc-500 w-12 text-right">{:.1}%</span>
                    </div>
                </div>"#,
                l.percent, safe_l_name, format_duration(l.total_seconds), l.percent
            ));
        }
    }

    let mut bento_recent_rows = String::new();
    if recent_heartbeats.is_empty() {
        bento_recent_rows.push_str(r#"<div class="py-4 text-center text-xs text-zinc-600 font-mono">No activity stream</div>"#);
    } else {
        for hb in recent_heartbeats.iter().take(6) {
            let file_str = hb
                .file
                .as_deref()
                .map(sanitize_file_path)
                .unwrap_or_else(|| "-".to_string());
            let safe_file = escape_html(&file_str);
            let safe_proj = escape_html(&hb.project);
            let safe_lang = escape_html(hb.language.as_deref().unwrap_or("-"));
            let safe_editor = escape_html(hb.editor.as_deref().unwrap_or("-"));
            let date_time = chrono::DateTime::from_timestamp(hb.timestamp, 0)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| hb.timestamp.to_string());

            bento_recent_rows.push_str(&format!(
                r#"<div class="flex items-center justify-between py-2 text-xs font-mono border-b border-zinc-800/40 last:border-0 hover:bg-zinc-800/20 px-2 rounded-lg transition-colors">
                    <div class="flex items-center gap-2 truncate">
                        <span class="text-zinc-100 font-medium">{}</span>
                        <span class="text-zinc-600">&middot;</span>
                        <span class="text-zinc-400 truncate max-w-[200px]" title="{}">{}</span>
                    </div>
                    <div class="flex items-center gap-2.5 text-zinc-500 text-[11px] flex-shrink-0">
                        <span class="text-violet-400">{}</span>
                        <span class="bg-zinc-800/70 px-1.5 py-0.5 rounded text-zinc-400">{}</span>
                        <span class="text-zinc-600">{}</span>
                    </div>
                </div>"#,
                safe_proj, safe_file, safe_file, safe_lang, safe_editor, date_time
            ));
        }
    }

    let raw_sample_project = summary
        .projects
        .first()
        .map(|p| p.name.as_str())
        .unwrap_or("chronosync");
    let safe_username = escape_html(&user.username);
    let safe_api_key = escape_html(&user.api_key);
    let badge_url = format!(
        "{}/api/v1/badge/{}/{}",
        app_url, safe_username, raw_sample_project
    );
    let badge_markdown = format!("[![Chronosync]({})]({})", badge_url, app_url);
    let profile_url = format!("{}/@{}", app_url, safe_username);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>chronosync &mdash; {username}</title>
    <meta name="theme-color" content="#8b5cf6">
    <meta property="og:site_name" content="chronosync">
    <meta property="og:title" content="chronosync &mdash; @{username}">
    <meta property="og:description" content="Developer time tracker &amp; Bento Grid activity dashboard.">
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        body {{ font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif; }}
        code, pre, .font-mono {{ font-family: 'JetBrains Mono', monospace; }}
    </style>
</head>
<body class="bg-[#09090b] text-zinc-200 min-h-screen antialiased p-4 sm:p-6 lg:p-8 bg-[linear-gradient(rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:48px_48px]">
    <div class="max-w-6xl mx-auto space-y-4">

        <header class="flex items-center justify-between pb-3 border-b border-zinc-800/60">
            <div class="flex items-center gap-3">
                <span class="font-bold text-white text-base tracking-tight font-mono">&gt; chronosync</span>
                <span class="flex items-center gap-1.5 text-xs text-zinc-400 font-mono">
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                    <span>online</span>
                </span>
                <span class="text-zinc-700">&middot;</span>
                <span class="text-xs text-zinc-500 font-mono">@{username}</span>
            </div>

            <div class="flex items-center gap-3">
                <div class="flex items-center bg-zinc-900/60 border border-zinc-800/80 rounded-xl p-1 text-xs font-mono">
                    <a href="?days=7" class="px-3 py-1 rounded-lg {active_7}">7d</a>
                    <a href="?days=30" class="px-3 py-1 rounded-lg {active_30}">30d</a>
                    <a href="?days=365" class="px-3 py-1 rounded-lg {active_all}">all</a>
                </div>

                <div class="flex items-center gap-2 pl-2 text-xs font-mono">
                    <a href="/@{username}" class="text-violet-400 hover:text-violet-300 transition-colors border border-violet-900/50 bg-violet-950/30 px-2.5 py-1 rounded-xl">public profile &rarr;</a>
                    <a href="#setup" class="text-zinc-400 hover:text-white transition-colors border border-zinc-800 bg-zinc-900/60 px-2.5 py-1 rounded-xl">setup</a>
                    <a href="/logout" class="text-zinc-500 hover:text-red-400 transition-colors">logout</a>
                </div>
            </div>
        </header>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="absolute inset-0 bg-gradient-to-br from-violet-500/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
                <div class="relative z-10 flex items-center justify-between">
                    <span class="text-[11px] font-mono text-zinc-500 uppercase tracking-wider">Total Coding Time ({days_param}d)</span>
                    <span class="text-xs font-mono px-2 py-0.5 rounded-md bg-violet-500/10 text-violet-300 border border-violet-500/20">active session</span>
                </div>
                <div class="relative z-10 my-4">
                    <div class="text-4xl sm:text-5xl font-bold font-mono tracking-tight text-white">{total_time_str}</div>
                    <div class="text-xs text-zinc-400 font-mono mt-2">Daily average: <strong class="text-zinc-200">{daily_avg_str} / day</strong></div>
                </div>
                <div class="relative z-10 pt-3 border-t border-zinc-800/50 flex items-center justify-between text-xs font-mono text-zinc-500">
                    <span>{projects_count} active repositories</span>
                    <span>developer: @{username}</span>
                </div>
            </div>

            <div class="rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="absolute inset-0 bg-gradient-to-br from-violet-500/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
                <div class="relative z-10 text-[11px] font-mono text-zinc-500 uppercase tracking-wider">Top Language</div>
                <div class="relative z-10 my-2">
                    <div class="text-3xl font-bold font-mono text-violet-400">{top_language}</div>
                    <div class="text-xs text-zinc-400 font-mono mt-1">core project stack</div>
                </div>
                <div class="relative z-10 text-[11px] font-mono text-zinc-500">
                    dominant technology
                </div>
            </div>

            <div class="rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="absolute inset-0 bg-gradient-to-br from-emerald-500/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
                <div class="relative z-10 text-[11px] font-mono text-zinc-500 uppercase tracking-wider">API Authentication</div>
                <div class="relative z-10 my-2">
                    <div class="flex items-center gap-2">
                        <input type="password" value="{api_key}" id="tokenInput" readonly class="bg-zinc-950 border border-zinc-800 text-xs font-mono text-zinc-300 px-2.5 py-1.5 w-32 rounded-lg select-all focus:outline-none">
                        <button onclick="copyToken()" class="border border-zinc-800 hover:border-zinc-600 bg-zinc-800/40 px-2.5 py-1.5 text-xs font-mono text-zinc-200 rounded-lg transition-colors">copy</button>
                    </div>
                </div>
                <div class="relative z-10 text-[11px] font-mono text-zinc-500">
                    Bearer token for CLI &amp; IDEs
                </div>
            </div>

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="flex items-center justify-between pb-2 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Daily Coding Activity</span>
                    <span class="text-[11px] font-mono text-zinc-500">last {days_param} days</span>
                </div>
                <div class="my-auto">
                    {bento_chart_bars}
                </div>
            </div>

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="flex items-center justify-between pb-3 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Projects Breakdown</span>
                    <span class="text-[11px] font-mono text-zinc-500">Time &middot; %</span>
                </div>
                <div class="space-y-1.5 my-2">
                    {bento_project_rows}
                </div>
                <div class="pt-2 border-t border-zinc-800/40 text-[11px] font-mono text-zinc-500 text-right">
                    {projects_count} total projects tracked
                </div>
            </div>

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="flex items-center justify-between pb-3 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Languages Distribution</span>
                    <span class="text-[11px] font-mono text-zinc-500">Time &middot; %</span>
                </div>
                <div class="space-y-1.5 my-2">
                    {bento_language_rows}
                </div>
                <div class="pt-2 border-t border-zinc-800/40 text-[11px] font-mono text-zinc-500 text-right">
                    technology metrics
                </div>
            </div>

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="flex items-center justify-between pb-3 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Recent Activity Stream</span>
                    <span class="text-[11px] font-mono text-emerald-400">live daemon</span>
                </div>
                <div class="space-y-1 my-2">
                    {bento_recent_rows}
                </div>
                <div class="pt-2 border-t border-zinc-800/40 text-[11px] font-mono text-zinc-500 text-right">
                    auto-synced with agent
                </div>
            </div>

            <div id="setup" class="lg:col-span-4 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 space-y-4 hover:border-zinc-700/80 transition-all shadow-sm">
                <div class="flex items-center justify-between pb-3 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Integration &amp; Setup</span>
                    <span class="text-[11px] font-mono text-zinc-500">quick configuration</span>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 text-xs font-mono">
                    <div class="p-3.5 rounded-xl bg-zinc-950 border border-zinc-800/80 space-y-1.5">
                        <div class="text-zinc-200 font-medium"># 1. Configure CLI</div>
                        <pre class="text-zinc-400 overflow-x-auto text-[11px]">export CHRONOSYNC_SERVER_URL="{app_url}"
export CHRONOSYNC_API_KEY="{api_key}"

chronosync log --file src/main.rs --write</pre>
                    </div>

                    <div class="p-3.5 rounded-xl bg-zinc-950 border border-zinc-800/80 space-y-1.5">
                        <div class="text-zinc-200 font-medium"># 2. Public Bento Profile</div>
                        <pre class="text-zinc-400 overflow-x-auto text-[11px]">{profile_url}</pre>
                        <a href="{profile_url}" class="inline-block text-violet-400 hover:text-violet-300 text-[11px] pt-1">Open public profile &rarr;</a>
                    </div>

                    <div class="p-3.5 rounded-xl bg-zinc-950 border border-zinc-800/80 space-y-1.5">
                        <div class="text-zinc-200 font-medium"># 3. GitHub README Badge</div>
                        <div class="py-1"><img src="{badge_url}" alt="badge"></div>
                        <pre class="text-zinc-400 overflow-x-auto text-[11px]">{badge_markdown}</pre>
                    </div>
                </div>
            </div>

        </div>

        <footer class="pt-4 pb-6 text-center text-xs text-zinc-600 font-mono flex justify-between border-t border-zinc-800/40">
            <span>chronosync v0.1.0</span>
            <span class="text-zinc-700">&middot;</span>
            <span>Матвей &bull; Tallinn</span>
            <span class="text-zinc-700">&middot;</span>
            <a href="https://sqrilizz.tech" class="text-violet-400 hover:text-violet-200 transition-colors">sqrilizz.tech</a>
        </footer>
    </div>

    <script>
        function copyToken() {{
            const input = document.getElementById('tokenInput');
            input.type = 'text';
            navigator.clipboard.writeText(input.value);
            input.type = 'password';
        }}
    </script>
</body>
</html>"##,
        username = safe_username,
        total_time_str = total_time_str,
        daily_avg_str = daily_avg_str,
        projects_count = projects_count,
        top_language = top_language,
        days_param = days_param,
        api_key = safe_api_key,
        active_7 = if days_param == 7 {
            "bg-zinc-800 text-white font-medium"
        } else {
            "text-zinc-500 hover:text-zinc-300"
        },
        active_30 = if days_param == 30 {
            "bg-zinc-800 text-white font-medium"
        } else {
            "text-zinc-500 hover:text-zinc-300"
        },
        active_all = if days_param == 365 {
            "bg-zinc-800 text-white font-medium"
        } else {
            "text-zinc-500 hover:text-zinc-300"
        },
        bento_chart_bars = bento_chart_bars,
        bento_project_rows = bento_project_rows,
        bento_language_rows = bento_language_rows,
        bento_recent_rows = bento_recent_rows,
        app_url = app_url,
        badge_url = badge_url,
        badge_markdown = badge_markdown,
        profile_url = profile_url
    )
}

pub fn render_profile_page(
    profile_user: &User,
    summary: &StatsSummary,
    recent_heartbeats: &[Heartbeat],
    app_url: &str,
    days: Option<u32>,
    viewer: Option<&User>,
) -> String {
    let days_param = days.unwrap_or(7);
    let total_time_str = format_duration(summary.total_seconds);
    let projects_count = summary.projects.len();
    let top_language = summary
        .languages
        .first()
        .map(|l| escape_html(&l.name))
        .unwrap_or_else(|| "none".to_string());

    let daily_avg = if !summary.daily_activity.is_empty() {
        summary.total_seconds / (summary.daily_activity.len() as u64)
    } else {
        0
    };
    let daily_avg_str = format_duration(daily_avg);

    let max_daily_seconds = summary
        .daily_activity
        .iter()
        .map(|d| d.total_seconds)
        .max()
        .unwrap_or(1)
        .max(1);

    let mut bento_chart_bars = String::new();
    if summary.daily_activity.is_empty() {
        bento_chart_bars.push_str(r#"<div class="h-36 flex items-center justify-center text-xs text-zinc-600 font-mono">No public activity recorded for this period</div>"#);
    } else {
        bento_chart_bars.push_str(r#"<div class="h-44 flex items-end gap-2.5 pt-6">"#);
        for d in &summary.daily_activity {
            let height_pct = ((d.total_seconds as f64 / max_daily_seconds as f64) * 100.0).max(8.0);
            let time_formatted = format_duration(d.total_seconds);
            let safe_date = escape_html(&d.date);
            let day_label = if safe_date.len() >= 10 {
                &safe_date[5..]
            } else {
                &safe_date
            };
            bento_chart_bars.push_str(&format!(
                r#"<div class="flex-1 flex flex-col items-center h-full justify-end group relative cursor-default">
                    <div class="absolute -top-7 opacity-0 group-hover:opacity-100 transition-opacity bg-zinc-800 border border-zinc-700 text-[11px] font-mono px-2 py-0.5 rounded-md shadow whitespace-nowrap z-20 pointer-events-none">
                        {}: {}
                    </div>
                    <div class="w-full bg-violet-500/20 group-hover:bg-violet-500/50 border-t-2 border-violet-400 transition-all rounded-t-sm" style="height: {:.1}%"></div>
                    <span class="text-[10px] text-zinc-500 font-mono mt-2 truncate">{}</span>
                </div>"#,
                safe_date, time_formatted, height_pct, day_label
            ));
        }
        bento_chart_bars.push_str("</div>");
    }

    let mut bento_project_rows = String::new();
    if summary.projects.is_empty() {
        bento_project_rows.push_str(r#"<div class="py-4 text-center text-xs text-zinc-600 font-mono">No public projects</div>"#);
    } else {
        for p in &summary.projects {
            let safe_p_name = escape_html(&p.name);
            bento_project_rows.push_str(&format!(
                r#"<div class="relative py-2.5 px-3 rounded-xl hover:bg-zinc-800/40 flex items-center justify-between text-xs transition-colors overflow-hidden group">
                    <div class="absolute inset-y-0 left-0 bg-violet-500/10 group-hover:bg-violet-500/20 rounded-xl transition-colors" style="width: {:.1}%"></div>
                    <span class="relative z-10 text-zinc-200 font-mono truncate font-medium">{}</span>
                    <div class="relative z-10 flex items-center gap-2 font-mono text-[11px]">
                        <span class="text-zinc-300 font-medium">{}</span>
                        <span class="text-zinc-500 w-12 text-right">{:.1}%</span>
                    </div>
                </div>"#,
                p.percent, safe_p_name, format_duration(p.total_seconds), p.percent
            ));
        }
    }

    let mut bento_language_rows = String::new();
    if summary.languages.is_empty() {
        bento_language_rows.push_str(r#"<div class="py-4 text-center text-xs text-zinc-600 font-mono">No languages recorded</div>"#);
    } else {
        for l in &summary.languages {
            let safe_l_name = escape_html(&l.name);
            bento_language_rows.push_str(&format!(
                r#"<div class="relative py-2.5 px-3 rounded-xl hover:bg-zinc-800/40 flex items-center justify-between text-xs transition-colors overflow-hidden group">
                    <div class="absolute inset-y-0 left-0 bg-emerald-500/10 group-hover:bg-emerald-500/20 rounded-xl transition-colors" style="width: {:.1}%"></div>
                    <span class="relative z-10 text-zinc-200 font-mono truncate font-medium">{}</span>
                    <div class="relative z-10 flex items-center gap-2 font-mono text-[11px]">
                        <span class="text-zinc-300 font-medium">{}</span>
                        <span class="text-zinc-500 w-12 text-right">{:.1}%</span>
                    </div>
                </div>"#,
                l.percent, safe_l_name, format_duration(l.total_seconds), l.percent
            ));
        }
    }

    let mut bento_recent_rows = String::new();
    if recent_heartbeats.is_empty() {
        bento_recent_rows.push_str(r#"<div class="py-4 text-center text-xs text-zinc-600 font-mono">No recent heartbeats</div>"#);
    } else {
        for hb in recent_heartbeats.iter().take(6) {
            let file_str = hb
                .file
                .as_deref()
                .map(sanitize_file_path)
                .unwrap_or_else(|| "-".to_string());
            let safe_file = escape_html(&file_str);
            let safe_proj = escape_html(&hb.project);
            let safe_lang = escape_html(hb.language.as_deref().unwrap_or("-"));
            let safe_branch = escape_html(hb.branch.as_deref().unwrap_or("main"));
            let date_time = chrono::DateTime::from_timestamp(hb.timestamp, 0)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| hb.timestamp.to_string());

            bento_recent_rows.push_str(&format!(
                r#"<div class="flex items-center justify-between py-2 text-xs font-mono border-b border-zinc-800/40 last:border-0 hover:bg-zinc-800/20 px-2 rounded-lg transition-colors">
                    <div class="flex items-center gap-2 truncate">
                        <span class="text-zinc-100 font-medium">{}</span>
                        <span class="text-zinc-600">&middot;</span>
                        <span class="text-zinc-400 truncate max-w-[180px]" title="{}">{}</span>
                    </div>
                    <div class="flex items-center gap-2.5 text-zinc-500 text-[11px] flex-shrink-0">
                        <span class="text-violet-400">{}</span>
                        <span class="bg-zinc-800/70 px-1.5 py-0.5 rounded text-zinc-400">{}</span>
                        <span class="text-zinc-600">{}</span>
                    </div>
                </div>"#,
                safe_proj, safe_file, safe_file, safe_lang, safe_branch, date_time
            ));
        }
    }

    let raw_sample_project = summary
        .projects
        .first()
        .map(|p| p.name.as_str())
        .unwrap_or("chronosync");
    let safe_username = escape_html(&profile_user.username);
    let avatar_url = safe_avatar_url(profile_user.avatar_url.as_deref());
    let display_name = escape_html(
        profile_user
            .name
            .as_deref()
            .unwrap_or(profile_user.username.as_str()),
    );

    let badge_url = format!(
        "{}/api/v1/badge/{}/{}",
        app_url, safe_username, raw_sample_project
    );
    let badge_markdown = format!(
        "[![{}'s chronosync]({})]({}/@{})",
        safe_username, badge_url, app_url, safe_username
    );

    let auth_header_right = match viewer {
        Some(v) if v.id == profile_user.id => {
            r#"<a href="/dashboard" class="text-violet-400 hover:text-white transition-colors border border-violet-900/60 bg-violet-950/40 px-3 py-1 rounded-xl text-xs font-mono">dashboard &rarr;</a>"#
                .to_string()
        }
        Some(_) => {
            r#"<a href="/dashboard" class="text-zinc-400 hover:text-white transition-colors border border-zinc-800 bg-zinc-900/60 px-3 py-1 rounded-xl text-xs font-mono">my dashboard</a>"#
                .to_string()
        }
        None => {
            r#"<a href="/login" class="text-zinc-300 hover:text-white transition-colors border border-zinc-700 bg-zinc-800/80 px-3 py-1 rounded-xl text-xs font-mono">sign in</a>"#
                .to_string()
        }
    };

    let user_profile_href = format!("{}/@{}", app_url, safe_username);

    let og_description = if summary.total_seconds > 0 {
        format!(
            "Tracked {} ({}d) across {} active projects. Top language: {}.",
            total_time_str, days_param, projects_count, top_language
        )
    } else {
        "Developer profile on chronosync. Tracked coding activity, languages, and stats."
            .to_string()
    };
    let safe_og_description = escape_html(&og_description);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{display_name} (@{username}) &mdash; chronosync</title>
    <meta name="theme-color" content="#8b5cf6">
    <meta property="og:site_name" content="chronosync">
    <meta property="og:type" content="profile">
    <meta property="og:title" content="{display_name} (@{username}) &mdash; chronosync">
    <meta property="og:description" content="{og_description}">
    <meta property="og:url" content="{user_profile_href}">
    <meta property="og:image" content="{avatar_url}">
    <meta property="profile:username" content="{username}">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="{display_name} (@{username}) &mdash; chronosync">
    <meta name="twitter:description" content="{og_description}">
    <meta name="twitter:image" content="{avatar_url}">
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        body {{ font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif; }}
        code, pre, .font-mono {{ font-family: 'JetBrains Mono', monospace; }}
    </style>
</head>
<body class="bg-[#09090b] text-zinc-200 min-h-screen antialiased p-4 sm:p-6 lg:p-8 flex flex-col justify-between bg-[linear-gradient(rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:48px_48px]">
    <div class="max-w-6xl mx-auto w-full space-y-4">

        <header class="flex items-center justify-between pb-3 border-b border-zinc-800/60">
            <a href="/" class="font-bold text-white text-base tracking-tight font-mono">&gt; chronosync</a>
            <div class="flex items-center gap-2 text-xs font-mono">
                {auth_header_right}
            </div>
        </header>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">

            <div class="lg:col-span-4 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4 hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="absolute inset-0 bg-gradient-to-r from-violet-500/10 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
                <div class="relative z-10 flex items-center gap-4">
                    <img src="{avatar_url}" alt="{username}" class="w-16 h-16 rounded-2xl border border-zinc-700/80 object-cover shadow-md">
                    <div>
                        <div class="flex items-center gap-2">
                            <h1 class="text-2xl font-bold text-white font-mono">{display_name}</h1>
                            <span class="flex items-center gap-1 text-[11px] font-mono text-emerald-400 bg-emerald-950/40 border border-emerald-800/50 px-2 py-0.5 rounded-full">
                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                                active
                            </span>
                        </div>
                        <div class="flex items-center gap-3 text-xs text-zinc-400 font-mono mt-1">
                            <span class="text-violet-400">@{username}</span>
                            <span class="text-zinc-600">&middot;</span>
                            <a href="https://github.com/{username}" target="_blank" rel="noopener noreferrer" class="text-zinc-500 hover:text-white transition-colors" title="GitHub profile">
                                github.com/{username} &nearr;
                            </a>
                        </div>
                    </div>
                </div>

                <div class="relative z-10 flex items-center gap-3 self-end sm:self-center">
                    <div class="flex items-center bg-zinc-900/60 border border-zinc-800/80 rounded-xl p-1 text-xs font-mono">
                        <a href="?days=7" class="px-3 py-1 rounded-lg {active_7}">7d</a>
                        <a href="?days=30" class="px-3 py-1 rounded-lg {active_30}">30d</a>
                        <a href="?days=365" class="px-3 py-1 rounded-lg {active_all}">all</a>
                    </div>
                </div>
            </div>

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="absolute inset-0 bg-gradient-to-br from-violet-500/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
                <div class="relative z-10 flex items-center justify-between">
                    <span class="text-[11px] font-mono text-zinc-500 uppercase tracking-wider">Public Tracked Time ({days_param}d)</span>
                    <span class="text-xs font-mono px-2 py-0.5 rounded-md bg-violet-500/10 text-violet-300 border border-violet-500/20">verified</span>
                </div>
                <div class="relative z-10 my-4">
                    <div class="text-4xl sm:text-5xl font-bold font-mono tracking-tight text-white">{total_time_str}</div>
                    <div class="text-xs text-zinc-400 font-mono mt-2">Daily average: <strong class="text-zinc-200">{daily_avg_str} / day</strong></div>
                </div>
                <div class="relative z-10 pt-3 border-t border-zinc-800/50 flex items-center justify-between text-xs font-mono text-zinc-500">
                    <span>{projects_count} active repositories</span>
                    <span>developer: @{username}</span>
                </div>
            </div>

            <div class="rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="absolute inset-0 bg-gradient-to-br from-violet-500/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
                <div class="relative z-10 text-[11px] font-mono text-zinc-500 uppercase tracking-wider">Primary Language</div>
                <div class="relative z-10 my-2">
                    <div class="text-3xl font-bold font-mono text-violet-400">{top_language}</div>
                    <div class="text-xs text-zinc-400 font-mono mt-1">core project stack</div>
                </div>
                <div class="relative z-10 text-[11px] font-mono text-zinc-500">
                    dominant technology
                </div>
            </div>

            <div class="rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="absolute inset-0 bg-gradient-to-br from-emerald-500/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
                <div class="relative z-10 text-[11px] font-mono text-zinc-500 uppercase tracking-wider">Public Profile Badge</div>
                <div class="relative z-10 my-2">
                    <img src="{badge_url}" alt="badge" class="h-6">
                </div>
                <div class="relative z-10 text-[11px] font-mono text-zinc-500 truncate" title="{badge_markdown}">
                    live SVG for GitHub README
                </div>
            </div>

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="flex items-center justify-between pb-2 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Daily Coding Activity</span>
                    <span class="text-[11px] font-mono text-zinc-500">last {days_param} days</span>
                </div>
                <div class="my-auto">
                    {bento_chart_bars}
                </div>
            </div>

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="flex items-center justify-between pb-3 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Top Projects</span>
                    <span class="text-[11px] font-mono text-zinc-500">Time &middot; %</span>
                </div>
                <div class="space-y-1.5 my-2">
                    {bento_project_rows}
                </div>
                <div class="pt-2 border-t border-zinc-800/40 text-[11px] font-mono text-zinc-500 text-right">
                    {projects_count} total projects tracked
                </div>
            </div>

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="flex items-center justify-between pb-3 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Languages Distribution</span>
                    <span class="text-[11px] font-mono text-zinc-500">Time &middot; %</span>
                </div>
                <div class="space-y-1.5 my-2">
                    {bento_language_rows}
                </div>
                <div class="pt-2 border-t border-zinc-800/40 text-[11px] font-mono text-zinc-500 text-right">
                    technology metrics
                </div>
            </div>

            <div class="lg:col-span-2 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 flex flex-col justify-between hover:border-zinc-700/80 transition-all shadow-sm relative overflow-hidden group">
                <div class="flex items-center justify-between pb-3 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Public Activity Feed</span>
                    <span class="text-[11px] font-mono text-emerald-400">sanitized</span>
                </div>
                <div class="space-y-1 my-2">
                    {bento_recent_rows}
                </div>
                <div class="pt-2 border-t border-zinc-800/40 text-[11px] font-mono text-zinc-500 text-right">
                    zero absolute path exposure
                </div>
            </div>

            <div class="lg:col-span-4 rounded-2xl bg-zinc-900/40 border border-zinc-800/70 p-6 space-y-3 hover:border-zinc-700/80 transition-all shadow-sm">
                <div class="flex items-center justify-between pb-2 border-b border-zinc-800/50">
                    <span class="text-xs font-semibold text-zinc-200 uppercase tracking-wider font-mono">Embed in GitHub Profile</span>
                    <span class="text-[11px] font-mono text-zinc-500">README snippet</span>
                </div>
                <div class="flex flex-col sm:flex-row items-center gap-3">
                    <input type="text" readonly value="{badge_markdown}" class="flex-1 w-full bg-zinc-950 border border-zinc-800 rounded-xl px-3 py-2 text-xs font-mono text-zinc-300 select-all focus:outline-none">
                    <a href="{user_profile_href}" class="w-full sm:w-auto text-center px-4 py-2 rounded-xl bg-violet-600 hover:bg-violet-500 text-white font-mono text-xs font-semibold transition-colors">
                        Share Link &rarr;
                    </a>
                </div>
            </div>

        </div>

        <footer class="pt-4 pb-6 text-center text-xs text-zinc-600 font-mono flex justify-between border-t border-zinc-800/40">
            <span>chronosync v0.1.0</span>
            <span class="text-zinc-700">&middot;</span>
            <span>Матвей &bull; Tallinn</span>
            <span class="text-zinc-700">&middot;</span>
            <a href="https://sqrilizz.tech" class="text-violet-400 hover:text-violet-200 transition-colors">sqrilizz.tech</a>
        </footer>
    </div>
</body>
</html>"##,
        username = safe_username,
        display_name = display_name,
        avatar_url = avatar_url,
        total_time_str = total_time_str,
        daily_avg_str = daily_avg_str,
        projects_count = projects_count,
        top_language = top_language,
        days_param = days_param,
        active_7 = if days_param == 7 {
            "bg-zinc-800 text-white font-medium"
        } else {
            "text-zinc-500 hover:text-zinc-300"
        },
        active_30 = if days_param == 30 {
            "bg-zinc-800 text-white font-medium"
        } else {
            "text-zinc-500 hover:text-zinc-300"
        },
        active_all = if days_param == 365 {
            "bg-zinc-800 text-white font-medium"
        } else {
            "text-zinc-500 hover:text-zinc-300"
        },
        bento_chart_bars = bento_chart_bars,
        bento_project_rows = bento_project_rows,
        bento_language_rows = bento_language_rows,
        bento_recent_rows = bento_recent_rows,
        badge_url = badge_url,
        badge_markdown = badge_markdown,
        auth_header_right = auth_header_right,
        user_profile_href = user_profile_href,
        og_description = safe_og_description
    )
}

pub fn render_profile_not_found(username: &str, app_url: &str, viewer: Option<&User>) -> String {
    let safe_username = escape_html(username);
    let auth_header_right = match viewer {
        Some(_) => {
            r#"<a href="/dashboard" class="text-zinc-400 hover:text-white transition-colors border border-zinc-800 bg-zinc-900/60 px-3 py-1 rounded-xl text-xs font-mono">my dashboard</a>"#
                .to_string()
        }
        None => {
            r#"<a href="/login" class="text-zinc-300 hover:text-white transition-colors border border-zinc-700 bg-zinc-800/80 px-3 py-1 rounded-xl text-xs font-mono">sign in</a>"#
                .to_string()
        }
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>@{username} not found &mdash; chronosync</title>
    <meta name="theme-color" content="#71717a">
    <meta property="og:site_name" content="chronosync">
    <meta property="og:title" content="@{username} not found &mdash; chronosync">
    <meta property="og:description" content="This user has not registered on chronosync yet.">
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        body {{ font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif; }}
        code, pre, .font-mono {{ font-family: 'JetBrains Mono', monospace; }}
    </style>
</head>
<body class="bg-[#09090b] text-zinc-200 min-h-screen antialiased p-4 sm:p-6 lg:p-8 flex flex-col justify-between bg-[linear-gradient(rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:48px_48px]">
    <div class="max-w-6xl mx-auto w-full space-y-4">
        <header class="flex items-center justify-between pb-3 border-b border-zinc-800/60">
            <a href="/" class="font-bold text-white text-base tracking-tight font-mono">&gt; chronosync</a>
            <div class="flex items-center gap-2 text-xs font-mono">
                {auth_header_right}
            </div>
        </header>

        <div class="max-w-lg mx-auto my-16 rounded-2xl bg-zinc-900/40 border border-zinc-800/80 p-8 text-center space-y-5 shadow-xl backdrop-blur-md">
            <div class="w-12 h-12 rounded-2xl bg-zinc-800/80 border border-zinc-700 mx-auto flex items-center justify-center text-zinc-400 font-mono text-lg font-bold">
                ?
            </div>
            <div>
                <h1 class="text-xl font-bold font-mono text-white">@{username}</h1>
                <p class="text-xs text-zinc-400 font-mono mt-1">This user has not registered on chronosync yet.</p>
            </div>
            <div class="pt-4 border-t border-zinc-800/60 flex items-center justify-center gap-3 text-xs font-mono">
                <a href="{app_url}" class="px-4 py-2 rounded-xl bg-zinc-800 text-zinc-200 hover:text-white transition-colors">home</a>
                <a href="{app_url}/login" class="px-4 py-2 rounded-xl bg-violet-600 hover:bg-violet-500 text-white transition-colors">claim profile</a>
            </div>
        </div>
    </div>

    <footer class="pt-4 pb-6 text-center text-xs text-zinc-600 font-mono border-t border-zinc-800/40 max-w-6xl mx-auto w-full">
        <span>chronosync v0.1.0</span>
        <span class="text-zinc-700">&middot;</span>
        <a href="https://sqrilizz.tech" class="text-violet-400 hover:text-violet-200 transition-colors">sqrilizz.tech</a>
    </footer>
</body>
</html>"##,
        username = safe_username,
        auth_header_right = auth_header_right,
        app_url = app_url
    )
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

fn safe_avatar_url(url: Option<&str>) -> String {
    match url {
        Some(u) if u.starts_with("https://") || u.starts_with("http://") => escape_html(u),
        _ => "https://avatars.githubusercontent.com/u/9919?v=4".to_string(),
    }
}

fn sanitize_file_path(path: &str) -> String {
    let clean = path.replace('\\', "/");
    let parts: Vec<&str> = clean.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() >= 2 {
        format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else if let Some(last) = parts.last() {
        last.to_string()
    } else {
        "-".to_string()
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
