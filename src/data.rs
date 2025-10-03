use serde::Deserialize;

/// Code metrics for a PR (lines added, deleted, files changed)
#[derive(Debug, Clone)]
pub struct CodeMetrics {
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
}

/// Generic item structure for GitHub API responses
#[derive(Deserialize)]
pub struct Item {
    pub url: String,
}

/// Wrapper for a list of items from GitHub API
#[derive(Deserialize)]
pub struct Items(pub Vec<Item>);

impl Items {
    /// Extract URLs from the items
    pub fn into_urls(self) -> Vec<String> {
        self.0.into_iter().map(|item| item.url).collect()
    }
}
