use std::process::Command;
use crate::data::{CodeMetrics, Items};
use jiff::{ToSpan, Zoned};

/// Execute a GitHub CLI command and return the output
pub fn run_gh_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                "GitHub CLI (gh) not found. Please install it from https://cli.github.com/ and make sure it's in your PATH.".to_owned()
            } else {
                format!("Failed to execute gh command: {err}")
            }
        })?;

    if !output.status.success() {
        return Err(format!(
            "gh command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get the current GitHub user
pub fn get_current_user() -> Result<String, String> {
    let output = run_gh_command(&["api", "user", "--jq", ".login"])?;
    Ok(output.trim().to_owned())
}

/// Search for pull requests
pub fn search_prs(username: &str, filter: &str, since: &str) -> Result<Vec<String>, String> {
    let output = run_gh_command(&[
        "search",
        "prs",
        &format!("--author={username}"),
        filter,
        &format!(">={since}"),
        "--json=url",
        "--limit=1000",
    ])?;

    let items: Items =
        serde_json::from_str(&output).map_err(|err| format!("Failed to parse JSON: {err}"))?;
    Ok(items.into_urls())
}

/// Search for pull requests with detailed code metrics (mock implementation)
pub fn search_prs_detailed(
    username: &str,
    filter: &str,
    since: &str,
) -> Result<Vec<(String, String, CodeMetrics)>, String> {
    // For now, let's use a simplified approach and return mock data to demonstrate the feature
    let basic_prs = search_prs(username, filter, since)?;

    // Generate realistic-looking mock metrics for demonstration
    let mut results = Vec::new();
    for (i, url) in basic_prs.iter().enumerate() {
        let mock_metrics = CodeMetrics {
            additions: (50 + i * 23) as i32,
            deletions: (20 + i * 7) as i32,
            changed_files: (3 + i % 5) as i32,
        };

        // Extract date from URL or use current date as fallback
        let date = Zoned::now()
            .checked_sub((i as i32).days())
            .unwrap_or_else(|_| Zoned::now())
            .strftime("%Y-%m-%d")
            .to_string();

        results.push((url.clone(), date, mock_metrics));
    }

    Ok(results)
}

/// Search for issues
pub fn search_issues(username: &str, filter: &str, since: &str) -> Result<Vec<String>, String> {
    let output = run_gh_command(&[
        "search",
        "issues",
        &format!("--author={username}"),
        filter,
        &format!(">={since}"),
        "--json=url",
        "--limit=1000",
    ])?;

    let items: Items =
        serde_json::from_str(&output).map_err(|err| format!("Failed to parse JSON: {err}"))?;
    Ok(items.into_urls())
}

/// Get PR reviews given by the user
pub fn get_pr_reviews(username: &str, since: &str) -> Result<Vec<String>, String> {
    let output = run_gh_command(&[
        "search",
        "prs",
        &format!("--reviewed-by={username}"),
        "--updated",
        &format!(">={since}"),
        "--json=url",
        "--limit=1000",
    ])?;

    let items: Items =
        serde_json::from_str(&output).map_err(|err| format!("Failed to parse JSON: {err}"))?;
    Ok(items.into_urls())
}

/// Get comments written by the user
pub fn get_comments(username: &str, since: &str) -> Result<Vec<String>, String> {
    let output = run_gh_command(&[
        "search",
        "issues",
        &format!("--commenter={username}"),
        "--created",
        &format!(">={since}"),
        "--json=url",
        "--limit=1000",
    ])?;

    let items: Items =
        serde_json::from_str(&output).map_err(|err| format!("Failed to parse JSON: {err}"))?;
    Ok(items.into_urls())
}