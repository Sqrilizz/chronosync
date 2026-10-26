use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "chronosync",
    version,
    about = "Minimalist time-tracker for geeks"
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        env = "CHRONOSYNC_SERVER_URL",
        default_value = "http://localhost:8080"
    )]
    pub server_url: String,

    #[arg(long, global = true, env = "CHRONOSYNC_API_KEY")]
    pub api_key: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Log(LogArgs),
    Stats(StatsArgs),
    Ping,
    GitSync(GitSyncArgs),
}

#[derive(Args, Debug)]
pub struct LogArgs {
    #[arg(short, long)]
    pub project: Option<String>,

    #[arg(short, long)]
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub branch: Option<String>,

    #[arg(short, long)]
    pub language: Option<String>,

    #[arg(short, long, default_value = "cli")]
    pub editor: String,

    #[arg(short, long)]
    pub write: bool,

    #[arg(short, long)]
    pub timestamp: Option<i64>,

    #[arg(long)]
    pub git_sync: bool,

    #[arg(
        long,
        env = "CHRONOSYNC_GIT_REPO",
        default_value = "https://github.com/Sqrilizz/chronosync-activity.git"
    )]
    pub git_repo: String,
}

#[derive(Args, Debug)]
pub struct StatsArgs {
    #[arg(short, long)]
    pub days: Option<u32>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct GitSyncArgs {
    #[arg(
        long,
        env = "CHRONOSYNC_GIT_REPO",
        default_value = "https://github.com/Sqrilizz/chronosync-activity.git"
    )]
    pub repo_url: String,

    #[arg(long)]
    pub repo_dir: Option<PathBuf>,

    #[arg(short, long)]
    pub project: Option<String>,

    #[arg(short, long)]
    pub file: Option<String>,

    #[arg(short, long)]
    pub language: Option<String>,
}
