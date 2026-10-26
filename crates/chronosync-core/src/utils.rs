use std::path::Path;

pub fn current_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

pub fn detect_language(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "rs" => Some("rust".to_string()),
        "go" => Some("go".to_string()),
        "py" => Some("python".to_string()),
        "js" | "mjs" | "cjs" => Some("javascript".to_string()),
        "ts" | "mts" | "cts" => Some("typescript".to_string()),
        "tsx" => Some("typescriptreact".to_string()),
        "jsx" => Some("javascriptreact".to_string()),
        "c" | "h" => Some("c".to_string()),
        "cpp" | "cc" | "cxx" | "hpp" => Some("cpp".to_string()),
        "ex" | "exs" => Some("elixir".to_string()),
        "html" | "htm" => Some("html".to_string()),
        "css" | "scss" | "sass" | "less" => Some("css".to_string()),
        "json" => Some("json".to_string()),
        "toml" => Some("toml".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "md" | "markdown" => Some("markdown".to_string()),
        "sql" => Some("sql".to_string()),
        "sh" | "bash" | "zsh" => Some("shell".to_string()),
        "lua" => Some("lua".to_string()),
        "zig" => Some("zig".to_string()),
        "java" => Some("java".to_string()),
        "kt" | "kts" => Some("kotlin".to_string()),
        "swift" => Some("swift".to_string()),
        _ => None,
    }
}

pub fn detect_git_branch(dir: &Path) -> Option<String> {
    let mut current = dir.to_path_buf();
    loop {
        let git_dir = current.join(".git");
        if git_dir.is_dir() {
            let head_file = git_dir.join("HEAD");
            if let Ok(content) = std::fs::read_to_string(head_file) {
                let trimmed = content.trim();
                if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
                    return Some(branch.to_string());
                } else if !trimmed.is_empty() {
                    return Some(trimmed.chars().take(8).collect());
                }
            }
            return None;
        } else if git_dir.is_file() {
            if let Ok(content) = std::fs::read_to_string(git_dir) {
                if let Some(gitdir_rel) = content.trim().strip_prefix("gitdir: ") {
                    let gitdir_path = current.join(gitdir_rel);
                    let head_file = gitdir_path.join("HEAD");
                    if let Ok(head_content) = std::fs::read_to_string(head_file) {
                        let trimmed = head_content.trim();
                        if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
                            return Some(branch.to_string());
                        }
                    }
                }
            }
            return None;
        }

        if !current.pop() {
            break;
        }
    }
    None
}
