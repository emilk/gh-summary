# gh-summary

A command-line tool to summarize your GitHub activity.

## Installation

```bash
cargo install --path .
```

## Usage

Show activity summary for the past week:

```bash
gh-summary
```

Show activity since a specific date:

```bash
gh-summary --since 2025-09-01
```

Show detailed output with links to all items:

```bash
gh-summary --verbose
gh-summary -v
```

Combine options:

```bash
gh-summary --since 2025-09-01 --verbose
```

## Requirements

- GitHub CLI (`gh`) must be installed and authenticated
- Run `gh auth login` if not already authenticated

## What it shows

- PRs opened
- Issues opened
- Issues closed
- PR reviews given
