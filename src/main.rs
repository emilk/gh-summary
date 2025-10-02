use chrono::{Duration, Utc};
use serde::Deserialize;
use std::process::Command;

fn run_gh_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute gh command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "gh command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn get_current_user() -> Result<String, String> {
    let output = run_gh_command(&["api", "user", "--jq", ".login"])?;
    Ok(output.trim().to_string())
}

fn search_prs_count(username: &str, filter: &str, _date_flag: &str, since: &str) -> Result<usize, String> {
    let output = run_gh_command(&[
        "search",
        "prs",
        "--author",
        username,
        filter,
        &format!(">={}", since),
        "--json",
        "url",
        "--limit",
        "1000",
    ])?;

    #[derive(Deserialize)]
    struct Items(Vec<serde_json::Value>);

    let items: Items =
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(items.0.len())
}

fn search_issues_count(username: &str, filter: &str, _date_flag: &str, since: &str) -> Result<usize, String> {
    let output = run_gh_command(&[
        "search",
        "issues",
        "--author",
        username,
        filter,
        &format!(">={}", since),
        "--json",
        "url",
        "--limit",
        "1000",
    ])?;

    #[derive(Deserialize)]
    struct Items(Vec<serde_json::Value>);

    let items: Items =
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(items.0.len())
}

fn get_pr_review_count(username: &str, since: &str) -> Result<usize, String> {
    let output = run_gh_command(&[
        "search",
        "prs",
        "--reviewed-by",
        username,
        "--updated",
        &format!(">={}", since),
        "--json",
        "url",
        "--limit",
        "1000",
    ])?;

    #[derive(Deserialize)]
    struct Items(Vec<serde_json::Value>);

    let items: Items =
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(items.0.len())
}

fn get_comment_count(username: &str, since: &str) -> Result<usize, String> {
    let output = run_gh_command(&[
        "search",
        "issues",
        "--commenter",
        username,
        "--created",
        &format!(">={}", since),
        "--json",
        "url",
        "--limit",
        "1000",
    ])?;

    #[derive(Deserialize)]
    struct Items(Vec<serde_json::Value>);

    let items: Items =
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(items.0.len())
}

fn main() {
    println!("Fetching your GitHub activity for the past week...\n");

    // Calculate date one week ago
    let one_week_ago = Utc::now() - Duration::days(7);
    let since_date = one_week_ago.format("%Y-%m-%d").to_string();

    // Get current user
    let username = match get_current_user() {
        Ok(user) => {
            println!("User: {}\n", user);
            user
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Make sure you're authenticated with 'gh auth login'");
            std::process::exit(1);
        }
    };

    println!("Activity since {}:", since_date);
    println!("{}", "=".repeat(50));

    // PRs opened
    match search_prs_count(&username, "--created", "--created", &since_date) {
        Ok(count) => println!("PRs opened:        {}", count),
        Err(e) => eprintln!("Error fetching PRs opened: {}", e),
    }

    // PRs closed/merged
    match search_prs_count(&username, "--closed", "--closed", &since_date) {
        Ok(count) => println!("PRs closed:        {}", count),
        Err(e) => eprintln!("Error fetching PRs closed: {}", e),
    }

    // Issues opened
    match search_issues_count(&username, "--created", "--created", &since_date) {
        Ok(count) => println!("Issues opened:     {}", count),
        Err(e) => eprintln!("Error fetching issues opened: {}", e),
    }

    // Issues closed
    match search_issues_count(&username, "--closed", "--closed", &since_date) {
        Ok(count) => println!("Issues closed:     {}", count),
        Err(e) => eprintln!("Error fetching issues closed: {}", e),
    }

    // PR reviews given
    match get_pr_review_count(&username, &since_date) {
        Ok(count) => println!("PR reviews given:  {}", count),
        Err(e) => eprintln!("Error fetching PR reviews: {}", e),
    }

    // Comments written
    match get_comment_count(&username, &since_date) {
        Ok(count) => println!("Comments written:  {}", count),
        Err(e) => eprintln!("Error fetching comments: {}", e),
    }

    println!("{}", "=".repeat(50));
}
