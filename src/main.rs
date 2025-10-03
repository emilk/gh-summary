use std::process::Command;

use clap::Parser;
use jiff::{ToSpan, Zoned};
use serde::Deserialize;

/// Summarize your GitHub activity
#[derive(Parser)]
#[command(name = "gh-summary")]
#[command(about = "A command-line tool to summarize your GitHub activity")]
#[command(long_about = r#"gh-summary - Summarize your GitHub activity

REQUIREMENTS:
    GitHub CLI (gh) must be installed and authenticated
    Run 'gh auth login' if not already authenticated"#)]
struct Args {
    /// Show detailed output with links to all items
    #[arg(short, long)]
    verbose: bool,

    /// Show activity since date (YYYY-MM-DD format, default: one week ago)
    #[arg(short, long, value_name = "DATE")]
    since: Option<String>,
}

fn run_gh_command(args: &[&str]) -> Result<String, String> {
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

fn get_current_user() -> Result<String, String> {
    let output = run_gh_command(&["api", "user", "--jq", ".login"])?;
    Ok(output.trim().to_string())
}

fn search_prs(username: &str, filter: &str, since: &str) -> Result<Vec<String>, String> {
    let output = run_gh_command(&[
        "search",
        "prs",
        &format!("--author={username}"),
        filter,
        &format!(">={since}"),
        "--json=url",
        "--limit=1000",
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
        &format!("--author={username}"),
        filter,
        &format!(">={since}"),
        "--json=url",
        "--limit=1000",
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
        &format!("--reviewed-by={username}"),
        "--updated",
        &format!(">={since}"),
        "--json=url",
        "--limit=1000",
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
        &format!("--commenter={username}"),
        "--created",
        &format!(">={since}"),
        "--json=url",
        "--limit=1000",
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

fn extract_repo(url: &str) -> Option<String> {
    // Extract owner/repo from URLs like https://github.com/owner/repo/...
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 5 && parts[2] == "github.com" {
        Some(format!("{}/{}", parts[3], parts[4]))
    } else {
        None
    }
}

fn print_items(label: &str, urls: &[String], verbose: bool) {
    let repo_count = urls
        .iter()
        .filter_map(|url| extract_repo(url))
        .collect::<std::collections::HashSet<_>>()
        .len();

    if verbose {
        println!("{:19}{}", label, urls.len());
        if !urls.is_empty() {
            let mut sorted_urls = urls.to_vec();
            sorted_urls.sort();
            for url in sorted_urls {
                println!("  - {url}");
            }
        }
    } else {
        let repo_suffix = if repo_count == 1 {
            "repository"
        } else {
            "repositories"
        };
        println!(
            "{:19}{} across {} {}",
            label,
            urls.len(),
            repo_count,
            repo_suffix
        );
    }
}

fn main() {
    let args = Args::parse();

    let verbose = args.verbose;

    // Parse --since argument or default to one week ago
    let since_date = args.since.unwrap_or_else(|| {
        let one_week_ago = Zoned::now().checked_sub(7_i32.days()).unwrap();
        one_week_ago.strftime("%Y-%m-%d").to_string()
    });

    // Get current user
    let username = match get_current_user() {
        Ok(user) => {
            println!("GitHub User: {user}\n");
            user
        }
        Err(err) => {
            eprintln!("Error: {err}");
            eprintln!("Make sure you're authenticated with 'gh auth login'");
            std::process::exit(1);
        }
    };

    println!("GitHub Activity since {since_date}:");
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
