mod cli;
mod client;
mod gitsync;

use std::env;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result, bail};
use chronosync_core::{
    Heartbeat, StatsSummary, current_timestamp, detect_git_branch, detect_language,
};
use clap::Parser;

use crate::cli::{Cli, Commands, GitSyncArgs, LogArgs, StatsArgs, WatchArgs};
use crate::client::ChronoClient;
use crate::gitsync::commit_and_push_activity;

#[tokio::main]
async fn main() -> Result<()> {
    let mut cli = Cli::parse();
    let (cfg_url, cfg_key) = load_config();
    if cli.server_url == "http://localhost:8080" {
        if let Some(u) = cfg_url {
            cli.server_url = u;
        }
    }
    if cli.api_key.is_none() {
        cli.api_key = cfg_key;
    }

    let client = ChronoClient::new(cli.server_url, cli.api_key);

    match cli.command {
        Commands::Log(args) => handle_log(client, args).await?,
        Commands::Stats(args) => handle_stats(client, args).await?,
        Commands::Ping => handle_ping(client).await?,
        Commands::GitSync(args) => handle_git_sync(args).await?,
        Commands::Watch(args) => handle_watch(client, args).await?,
    }

    Ok(())
}

fn load_config() -> (Option<String>, Option<String>) {
    if let Ok(home) = env::var("HOME") {
        let path = PathBuf::from(home)
            .join(".config")
            .join("chronosync")
            .join("config.toml");
        if let Ok(content) = std::fs::read_to_string(path) {
            let mut server = None;
            let mut key = None;
            for line in content.lines() {
                let line = line.trim();
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'').trim();
                    if k == "server_url" && !v.is_empty() {
                        server = Some(v.to_string());
                    } else if k == "api_key" && !v.is_empty() {
                        key = Some(v.to_string());
                    }
                }
            }
            return (server, key);
        }
    }
    (None, None)
}

async fn handle_ping(client: ChronoClient) -> Result<()> {
    let latency = client.ping().await?;
    println!("Server is reachable (latency: {:.2?})", latency);
    Ok(())
}

async fn handle_git_sync(args: GitSyncArgs) -> Result<()> {
    let project = args.project.unwrap_or_else(|| "default".to_string());
    let ts = current_timestamp();

    println!(
        "Syncing activity to GitHub repository: {}...",
        args.repo_url
    );
    let msg = commit_and_push_activity(
        &args.repo_url,
        args.repo_dir.as_deref(),
        &project,
        args.file.as_deref(),
        args.language.as_deref(),
        ts,
    )?;

    println!("GitHub sync successful: {}", msg);
    Ok(())
}

async fn handle_watch(client: ChronoClient, args: WatchArgs) -> Result<()> {
    let watch_dir = args
        .path
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let watch_dir = std::fs::canonicalize(&watch_dir).unwrap_or(watch_dir);

    let project = args.project.unwrap_or_else(|| {
        watch_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "default".to_string())
    });

    println!(
        "Watching '{}' for project '{}' every {}s...",
        watch_dir.display(),
        project,
        args.interval
    );
    println!("Press Ctrl+C to stop.");

    let mut last_reported_time: Option<SystemTime> = None;

    loop {
        if let Some((file, mtime)) = find_newest_file(&watch_dir) {
            let is_new = match last_reported_time {
                Some(prev) => mtime > prev,
                None => {
                    if let Ok(elapsed) = mtime.elapsed() {
                        elapsed.as_secs() < (args.interval * 3)
                    } else {
                        false
                    }
                }
            };

            if is_new {
                last_reported_time = Some(mtime);
                let rel_file = file
                    .strip_prefix(&watch_dir)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .to_string();
                let language = detect_language(&rel_file);
                let branch = detect_git_branch(&watch_dir);
                let ts = current_timestamp();

                let hb = Heartbeat {
                    project: project.clone(),
                    branch,
                    language: language.clone(),
                    file: Some(rel_file.clone()),
                    editor: Some("watch".to_string()),
                    timestamp: ts,
                    is_write: true,
                };

                match client.send_heartbeat(&hb).await {
                    Ok(_) => {
                        let now_str = chrono::Local::now().format("%H:%M:%S");
                        println!(
                            "[{}] Logged: {} | {} | {}",
                            now_str,
                            project,
                            language.as_deref().unwrap_or("general"),
                            rel_file
                        );
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to send heartbeat: {}", e);
                    }
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(args.interval)).await;
    }
}

fn find_newest_file(dir: &Path) -> Option<(PathBuf, SystemTime)> {
    let mut newest: Option<(PathBuf, SystemTime)> = None;
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "node_modules"
                    || file_name == "dist"
                {
                    continue;
                }
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        stack.push(path);
                    } else if ft.is_file() {
                        if let Ok(meta) = entry.metadata() {
                            if let Ok(mod_time) = meta.modified() {
                                match &newest {
                                    Some((_, best)) if mod_time > *best => {
                                        newest = Some((path, mod_time));
                                    }
                                    None => {
                                        newest = Some((path, mod_time));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    newest
}

async fn handle_log(client: ChronoClient, args: LogArgs) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current working directory")?;

    let file_str = args.file.as_ref().map(|f| f.to_string_lossy().to_string());

    let language = args
        .language
        .or_else(|| file_str.as_deref().and_then(detect_language));

    let branch = args.branch.or_else(|| {
        let inspect_dir = match &args.file {
            Some(f) if f.is_absolute() => f.parent().unwrap_or(&current_dir),
            _ => &current_dir,
        };
        detect_git_branch(inspect_dir)
    });

    let project = match args.project {
        Some(p) => p,
        None => {
            if let Some(f) = &args.file {
                if let Some(parent) = f.parent().and_then(|p| p.file_name()) {
                    parent.to_string_lossy().to_string()
                } else {
                    current_dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "default".to_string())
                }
            } else {
                current_dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "default".to_string())
            }
        }
    };

    if project.trim().is_empty() {
        bail!("Project name cannot be empty");
    }

    let timestamp = args.timestamp.unwrap_or_else(current_timestamp);

    let hb = Heartbeat {
        project: project.clone(),
        branch,
        language: language.clone(),
        file: file_str.clone(),
        editor: Some(args.editor),
        timestamp,
        is_write: args.write,
    };

    let resp = client.send_heartbeat(&hb).await?;
    println!(
        "Logged heartbeat for project '{}' [lang: {}, branch: {}] (status: {})",
        hb.project,
        hb.language.as_deref().unwrap_or("none"),
        hb.branch.as_deref().unwrap_or("none"),
        resp.status
    );

    if args.git_sync {
        println!("Farming activity commit to GitHub ({})...", args.git_repo);
        match commit_and_push_activity(
            &args.git_repo,
            None,
            &project,
            file_str.as_deref(),
            language.as_deref(),
            timestamp,
        ) {
            Ok(msg) => println!("GitHub commit pushed: {}", msg),
            Err(e) => eprintln!("Warning: GitHub git sync failed: {}", e),
        }
    }

    Ok(())
}

async fn handle_stats(client: ChronoClient, args: StatsArgs) -> Result<()> {
    let summary = client.get_summary(args.days).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
        return Ok(());
    }

    render_terminal_summary(&summary, args.days);
    Ok(())
}

fn render_terminal_summary(summary: &StatsSummary, days: Option<u32>) {
    let period = match days {
        Some(d) => format!("last {} days", d),
        None => "all time".to_string(),
    };

    println!("==================================================");
    println!("  Chronosync Stats Summary ({})", period);
    println!("  Total Time: {}", format_duration(summary.total_seconds));
    println!("==================================================");

    if !summary.projects.is_empty() {
        println!("\nProjects:");
        for p in &summary.projects {
            let bar = make_bar(p.percent);
            println!(
                "  {:<20} {:>10}  {:>5.1}%  {}",
                p.name,
                format_duration(p.total_seconds),
                p.percent,
                bar
            );
        }
    }

    if !summary.languages.is_empty() {
        println!("\nLanguages:");
        for l in &summary.languages {
            let bar = make_bar(l.percent);
            println!(
                "  {:<20} {:>10}  {:>5.1}%  {}",
                l.name,
                format_duration(l.total_seconds),
                l.percent,
                bar
            );
        }
    }

    if !summary.daily_activity.is_empty() {
        println!("\nDaily Activity:");
        for d in &summary.daily_activity {
            println!("  {:<12} {}", d.date, format_duration(d.total_seconds));
        }
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

fn make_bar(percent: f64) -> String {
    let total_chars = 20;
    let filled = ((percent / 100.0) * total_chars as f64).round() as usize;
    let filled = filled.min(total_chars);
    let empty = total_chars - filled;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}
