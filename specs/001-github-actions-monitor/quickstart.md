# Quick Start Guide: GitHub Actions Terminal Monitor

**Version**: 1.0.0  
**Date**: 2025-10-26

## Overview

The GitHub Actions Terminal Monitor is a Rust-based CLI tool that provides real-time monitoring of GitHub Actions workflows across multiple repositories in a compact terminal interface.

## Prerequisites

- Rust 1.75 or later
- GitHub Personal Access Token with `repo` scope
- Terminal that supports Unicode characters and colors

## Installation

### From Source

```bash
git clone https://github.com/your-org/github-actions-monitor.git
cd github-actions-monitor
cargo build --release
```

The binary will be available at `target/release/github-actions-monitor`.

### From Crates.io (when published)

```bash
cargo install github-actions-monitor
```

## Configuration

### 1. GitHub Personal Access Token

Create a GitHub Personal Access Token:
1. Go to https://github.com/settings/tokens
2. Click "Generate new token (classic)"
3. Give it a descriptive name (e.g., "Actions Monitor")
4. Select the `repo` scope
5. Click "Generate token"
6. Copy the token (it won't be shown again)

### 2. Environment Variables

Set the required environment variable:

```bash
export GITHUB_MONITOR_TOKEN="ghp_your_token_here"
```

Optional environment variables:

```bash
# Comma-separated list of repositories to monitor
export GITHUB_MONITOR_REPOSITORIES="owner1/repo1,owner2/repo2"

# Refresh interval in seconds (default: 60)
export GITHUB_MONITOR_MONITORING__REFRESH_INTERVAL_SECONDS=30

# Maximum concurrent API requests (default: 5)
export GITHUB_MONITOR_MONITORING__MAX_CONCURRENT_REQUESTS=3
```

### 3. Configuration File

Create a configuration file at `~/.config/github-monitor/config.toml`:

```toml
# GitHub token (can also be set via GITHUB_MONITOR_TOKEN env var)
github_token = "ghp_your_token_here"

# Repositories to monitor
[[repositories]]
owner = "rust-lang"
name = "rust"
branch = "master"

[[repositories]]
owner = "microsoft"
name = "vscode"
branch = "main"

[[repositories]]
owner = "torvalds"
name = "linux"

# Monitoring settings
[monitoring]
refresh_interval_seconds = 60
max_concurrent_requests = 5
max_retries = 3
workflow_runs_per_repo = 4

# UI settings
[ui.theme]
success_color = "green"
error_color = "red"
warning_color = "yellow"

[ui.layout]
min_terminal_width = 80
min_terminal_height = 24
compact_mode = true

[ui.icons]
success_icon = "✓"
error_icon = "✗"
running_icon = "⟳"
queued_icon = "⏸"

# Logging settings
[logging]
level = "info"
# file = "/var/log/github-monitor.log"  # Optional log file
```

## Basic Usage

### Start the Monitor

```bash
# Using environment variables only
github-actions-monitor

# Using configuration file
github-actions-monitor --config ~/.config/github-monitor/config.toml

# Override repositories from command line
github-actions-monitor --repositories "owner1/repo1,owner2/repo2"
```

### Command Line Options

```bash
github-actions-monitor --help
```

Available options:
- `--config, -c <PATH>`: Path to configuration file
- `--repositories, -r <LIST>`: Comma-separated list of repositories
- `--interval, -i <SECONDS>`: Refresh interval in seconds
- `--token, -t <TOKEN>`: GitHub personal access token
- `--log-level <LEVEL>`: Logging level (error, warn, info, debug, trace)
- `--help, -h`: Print help information
- `--version, -V`: Print version information

## User Interface

### Main Display

The terminal interface shows:

```
┌─ GitHub Actions Monitor ───────────────────────────────────────┐
│ Repository          Status    Last 4 Runs                    │
│─────────────────────────────────────────────────────────────────│
│ rust-lang/rust      ✓ ✓ ✓ ✓  2m ago • CI (master)           │
│ microsoft/vscode    ✓ ✗ ✓ ✓  5m ago • Build (main)           │
│ torvalds/linux      ✓ ✓ ✓ ✓  10m ago • Test (master)         │
│─────────────────────────────────────────────────────────────────│
│ Refresh: 60s | Rate: 4995/5000 | q:quit | Enter:menu | ↑↓:nav│
└─────────────────────────────────────────────────────────────────┘
```

### Navigation

- **↑/k**: Move up through repositories
- **↓/j**: Move down through repositories
- **Enter**: Open contextual menu for selected repository
- **Tab**: Switch between panels (if available)
- **q**: Quit application
- **r**: Manual refresh
- **p**: Toggle popup help

### Contextual Menu

When you press Enter on a repository, a menu appears:

```
┌─ Context Menu ────────────────────────────────────────────────┐
│ ▶ View recent workflow runs                                   │
│ ▶ View failure logs                                           │
│ ▶ Open repository in browser                                 │
│ ▶ Open latest workflow run                                   │
│─────────────────────────────────────────────────────────────────│
│ ESC: Close | ↑↓: Navigate | Enter: Select                    │
└─────────────────────────────────────────────────────────────────┘
```

### Status Indicators

- **✓** (green): Successful workflow run
- **✗** (red): Failed workflow run
- **⟳** (yellow): Currently running
- **⏸** (gray): Queued and waiting to run

## Advanced Usage

### Monitoring Multiple Organizations

```toml
[[repositories]]
owner = "my-org"
name = "frontend"

[[repositories]]
owner = "my-org"
name = "backend"

[[repositories]]
owner = "another-org"
name = "shared-lib"
```

### Specific Workflow Monitoring

```toml
[[repositories]]
owner = "my-org"
name = "my-repo"
workflows = ["ci.yml", "deploy.yml"]  # Only monitor these workflows
```

### Branch-Specific Monitoring

```toml
[[repositories]]
owner = "my-org"
name = "my-repo"
branch = "develop"  # Monitor develop branch instead of default
```

## Troubleshooting

### Common Issues

**"Authentication failed" error**:
- Verify your GitHub token has the `repo` scope
- Check that the token is correctly set in environment variables or config file
- Ensure the token hasn't expired

**"Rate limit exceeded" error**:
- GitHub allows 5,000 requests/hour for authenticated requests
- Reduce the refresh interval or number of monitored repositories
- Wait for the rate limit to reset (check the reset time in the error message)

**"Repository not found" error**:
- Verify the repository name and owner are correct
- Ensure the token has access to the repository (for private repos)
- Check that the repository exists and you have permission to access it

**Terminal display issues**:
- Ensure your terminal supports Unicode characters
- Check that the terminal is at least 80x24 characters
- Try disabling compact mode in the configuration

### Debug Mode

Enable debug logging for troubleshooting:

```bash
github-actions-monitor --log-level debug
```

Or in configuration file:

```toml
[logging]
level = "debug"
file = "/tmp/github-monitor-debug.log"
```

### Configuration Validation

Validate your configuration file:

```bash
github-actions-monitor --validate-config ~/.config/github-monitor/config.toml
```

## Performance Tips

1. **Optimize Refresh Interval**: Use longer intervals for large numbers of repositories
2. **Limit Concurrent Requests**: Reduce `max_concurrent_requests` if you hit rate limits
3. **Filter Workflows**: Specify only the workflows you need to monitor
4. **Use Compact Mode**: Enable compact mode for better performance on small terminals

## Integration Examples

### With tmux

Add to your `tmux.conf`:

```bash
bind-key g new-window -n "GitHub Actions" "github-actions-monitor"
```

### With systemd

Create `~/.config/systemd/user/github-monitor.service`:

```ini
[Unit]
Description=GitHub Actions Monitor
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/github-actions-monitor
Restart=on-failure
RestartSec=30

[Install]
WantedBy=default.target
```

Enable and start:

```bash
systemctl --user enable github-monitor
systemctl --user start github-monitor
```

## Support

- **Documentation**: [Link to docs]
- **Issues**: [Link to GitHub issues]
- **Discussions**: [Link to GitHub discussions]

## License

This project is licensed under the MIT License - see the LICENSE file for details.