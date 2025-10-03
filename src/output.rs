use std::collections::HashSet;
use colored::Colorize as _;
use crate::data::CodeMetrics;

/// Extract owner/repo from GitHub URLs
pub fn extract_repo(url: &str) -> Option<String> {
    // Extract owner/repo from URLs like https://github.com/owner/repo/...
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 5 && parts[2] == "github.com" {
        Some(format!("{}/{}", parts[3], parts[4]))
    } else {
        None
    }
}

/// Print activity items with optional verbose output
pub fn print_items(label: &str, urls: &[String], verbose: bool) {
    let repo_count = urls
        .iter()
        .filter_map(|url| extract_repo(url))
        .collect::<HashSet<_>>()
        .len();

    if verbose {
        println!("{:19}{}", label.cyan().bold(), urls.len().to_string().green().bold());
        if !urls.is_empty() {
            let mut sorted_urls = urls.to_vec();
            sorted_urls.sort();
            for url in sorted_urls {
                println!("  - {}", url.bright_blue());
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
            label.cyan().bold(),
            urls.len().to_string().green().bold(),
            repo_count.to_string().yellow(),
            repo_suffix.dimmed()
        );
    }
}

/// Print code metrics summary
pub fn print_code_metrics(label: &str, metrics: &[CodeMetrics]) {
    let total_additions: i32 = metrics.iter().map(|m| m.additions).sum();
    let total_deletions: i32 = metrics.iter().map(|m| m.deletions).sum();
    let total_files: i32 = metrics.iter().map(|m| m.changed_files).sum();

    println!(
        "{:19}{} {}  {} {}  {} {}",
        label.cyan().bold(),
        "+".green(),
        total_additions.to_string().green().bold(),
        "-".red(),
        total_deletions.to_string().red().bold(),
        "files:".dimmed(),
        total_files.to_string().yellow().bold()
    );
}

/// Print detailed metrics summary
pub fn print_metrics_summary(metrics: &[CodeMetrics]) {
    println!("\n{}", "📊 Code Metrics Summary".cyan().bold().underline());
    
    let total_additions: i32 = metrics.iter().map(|m| m.additions).sum();
    let total_deletions: i32 = metrics.iter().map(|m| m.deletions).sum();
    let total_files: i32 = metrics.iter().map(|m| m.changed_files).sum();
    
    println!("Total lines added:   {}", total_additions.to_string().green().bold());
    println!("Total lines deleted: {}", total_deletions.to_string().red().bold());
    println!("Total files changed: {}", total_files.to_string().yellow().bold());
    
    let net_lines = total_additions - total_deletions;
    if net_lines > 0 {
        println!("Net contribution:    {} {}", "+".green(), net_lines.to_string().green().bold());
    } else {
        println!("Net contribution:    {}{}", net_lines.to_string().red().bold(), " (cleanup/refactoring)".dimmed());
    }
}