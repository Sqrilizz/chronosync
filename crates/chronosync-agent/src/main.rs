mod cli;
mod client;
mod gitsync;

use std::env;

use anyhow::{Context, Result, bail};
use chronosync_core::{
    Heartbeat, StatsSummary, current_timestamp, detect_git_branch, detect_language,
};
use clap::Parser;

use crate::cli::{Cli, Commands, GitSyncArgs, LogArgs, StatsArgs};
use crate::client::ChronoClient;
use crate::gitsync::commit_and_push_activity;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = ChronoClient::new(cli.server_url, cli.api_key);

    match cli.command {
        Commands::Log(args) => handle_log(client, args).await?,
        Commands::Stats(args) => handle_stats(client, args).await?,
        Commands::Ping => handle_ping(client).await?,
        Commands::GitSync(args) => handle_git_sync(args).await?,
    }

    Ok(())
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
