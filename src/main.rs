use clap::Parser;
use jiff::{ToSpan, Zoned};

mod data;
mod github;
mod output;

use output::{print_code_metrics, print_items, print_metrics_summary};

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

    /// Show code metrics (lines added/removed)
    #[arg(short = 'm', long)]
    metrics: bool,
}

fn main() {
    let args = Args::parse();

    let verbose = args.verbose;
    let show_metrics = args.metrics;

    // Parse --since argument or default to one week ago
    let since_date = args.since.unwrap_or_else(|| {
        let one_week_ago = Zoned::now().checked_sub(7_i32.days()).unwrap();
        one_week_ago.strftime("%Y-%m-%d").to_string()
    });

    // Get current user
    let username = match github::get_current_user() {
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
    let pr_metrics = if show_metrics {
        match github::search_prs_detailed(&username, "--created", &since_date) {
            Ok(prs) => {
                let urls: Vec<String> = prs.iter().map(|(url, _, _)| url.clone()).collect();
                let metrics: Vec<_> = prs.iter().map(|(_, _, m)| m.clone()).collect();
                print_items("PRs opened:", &urls, verbose);
                if !metrics.is_empty() {
                    print_code_metrics("  Code changes:", &metrics);
                }
                Some(metrics)
            }
            Err(err) => {
                eprintln!("Error fetching detailed PRs: {err}");
                None
            }
        }
    } else {
        match github::search_prs(&username, "--created", &since_date) {
            Ok(urls) => {
                print_items("PRs opened:", &urls, verbose);
                None
            }
            Err(err) => {
                eprintln!("Error fetching PRs opened: {err}");
                None
            }
        }
    };

    // Issues opened
    match github::search_issues(&username, "--created", &since_date) {
        Ok(urls) => print_items("Issues opened:", &urls, verbose),
        Err(err) => eprintln!("Error fetching issues opened: {err}"),
    }

    // Issues closed
    match github::search_issues(&username, "--closed", &since_date) {
        Ok(urls) => print_items("Issues closed:", &urls, verbose),
        Err(err) => eprintln!("Error fetching issues closed: {err}"),
    }

    // PR reviews given
    match github::get_pr_reviews(&username, &since_date) {
        Ok(urls) => print_items("PR reviews given:", &urls, verbose),
        Err(err) => eprintln!("Error fetching PR reviews: {err}"),
    }

    if false {
        // Comments written
        match github::get_comments(&username, &since_date) {
            Ok(urls) => print_items("Comments written:", &urls, verbose),
            Err(err) => eprintln!("Error fetching comments: {err}"),
        }
    }

    println!("{}", "=".repeat(50));

    // Show summary metrics if requested
    if let (true, Some(metrics)) = (show_metrics, pr_metrics) {
        print_metrics_summary(&metrics);
    }
}
