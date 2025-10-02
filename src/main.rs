use chrono::{Duration, Utc};
use serde::Deserialize;
use std::env;
use std::process::Command;

fn run_gh_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|err| format!("Failed to execute gh command: {err}"))?;

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

fn search_prs(username: &str, filter: &str, since: &str) -> Result<Vec<String>, String> {
    let output = run_gh_command(&[
        "search",
        "prs",
        "--author",
        username,
        filter,
        &format!(">={since}"),
        "--json",
        "url",
        "--limit",
        "1000",
    ])?;

    #[derive(Deserialize)]
    struct Item {
        url: String,
    }
    #[derive(Deserialize)]
    struct Items(Vec<Item>);

    let items: Items =
        serde_json::from_str(&output).map_err(|err| format!("Failed to parse JSON: {err}"))?;
    Ok(items.0.into_iter().map(|item| item.url).collect())
}

fn search_issues(username: &str, filter: &str, since: &str) -> Result<Vec<String>, String> {
    let output = run_gh_command(&[
        "search",
        "issues",
        "--author",
        username,
        filter,
        &format!(">={since}"),
        "--json",
        "url",
        "--limit",
        "1000",
    ])?;

    #[derive(Deserialize)]
    struct Item {
        url: String,
    }
    #[derive(Deserialize)]
    struct Items(Vec<Item>);

    let items: Items =
        serde_json::from_str(&output).map_err(|err| format!("Failed to parse JSON: {err}"))?;
    Ok(items.0.into_iter().map(|item| item.url).collect())
}

fn get_pr_reviews(username: &str, since: &str) -> Result<Vec<String>, String> {
    let output = run_gh_command(&[
        "search",
        "prs",
        "--reviewed-by",
        username,
        "--updated",
        &format!(">={since}"),
        "--json",
        "url",
        "--limit",
        "1000",
    ])?;

    #[derive(Deserialize)]
    struct Item {
        url: String,
    }
    #[derive(Deserialize)]
    struct Items(Vec<Item>);

    let items: Items =
        serde_json::from_str(&output).map_err(|err| format!("Failed to parse JSON: {err}"))?;
    Ok(items.0.into_iter().map(|item| item.url).collect())
}

fn get_comments(username: &str, since: &str) -> Result<Vec<String>, String> {
    let output = run_gh_command(&[
        "search",
        "issues",
        "--commenter",
        username,
        "--created",
        &format!(">={since}"),
        "--json",
        "url",
        "--limit",
        "1000",
    ])?;

    #[derive(Deserialize)]
    struct Item {
        url: String,
    }
    #[derive(Deserialize)]
    struct Items(Vec<Item>);

    let items: Items =
        serde_json::from_str(&output).map_err(|err| format!("Failed to parse JSON: {err}"))?;
    Ok(items.0.into_iter().map(|item| item.url).collect())
}

fn print_items(label: &str, urls: &[String], verbose: bool) {
    println!("{:19}{}", label, urls.len());
    if verbose && !urls.is_empty() {
        for url in urls {
            println!("  - {url}");
        }
    }
}

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    println!("Fetching your GitHub activity for the past week...\n");

    // Calculate date one week ago
    let one_week_ago = Utc::now() - Duration::days(7);
    let since_date = one_week_ago.format("%Y-%m-%d").to_string();

    // Get current user
    let username = match get_current_user() {
        Ok(user) => {
            println!("User: {user}\n");
            user
        }
        Err(err) => {
            eprintln!("Error: {err}");
            eprintln!("Make sure you're authenticated with 'gh auth login'");
            std::process::exit(1);
        }
    };

    println!("Activity since {since_date}:");
    println!("{}", "=".repeat(50));

    // PRs opened
    match search_prs(&username, "--created", &since_date) {
        Ok(urls) => print_items("PRs opened:", &urls, verbose),
        Err(err) => eprintln!("Error fetching PRs opened: {err}"),
    }

    if false {
        // PRs closed/merged
        match search_prs(&username, "--closed", &since_date) {
            Ok(urls) => print_items("PRs closed:", &urls, verbose),
            Err(err) => eprintln!("Error fetching PRs closed: {err}"),
        }
    }

    // Issues opened
    match search_issues(&username, "--created", &since_date) {
        Ok(urls) => print_items("Issues opened:", &urls, verbose),
        Err(err) => eprintln!("Error fetching issues opened: {err}"),
    }

    // Issues closed
    match search_issues(&username, "--closed", &since_date) {
        Ok(urls) => print_items("Issues closed:", &urls, verbose),
        Err(err) => eprintln!("Error fetching issues closed: {err}"),
    }

    // PR reviews given
    match get_pr_reviews(&username, &since_date) {
        Ok(urls) => print_items("PR reviews given:", &urls, verbose),
        Err(err) => eprintln!("Error fetching PR reviews: {err}"),
    }

    if false {
        // Comments written
        match get_comments(&username, &since_date) {
            Ok(urls) => print_items("Comments written:", &urls, verbose),
            Err(err) => eprintln!("Error fetching comments: {err}"),
        }
    }

    println!("{}", "=".repeat(50));
}
