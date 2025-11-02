# NightHub Terminal Monitor - Linux

## Quick Setup

1. **Set Environment Variables**:
   export GITHUB_TOKEN=ghp_your_actual_token_here
   export REPOS=microsoft/vscode,rust-lang/rust,octocat/Hello-World

2. **Run Application**:
   ./nighthub-terminal-monitor

## Requirements
- Linux (Ubuntu, CentOS, etc.)
- Terminal with 256 colors
- Internet connection

## Controls
- j/k: Navigate repositories
- h/l: Navigate workflow runs
- Enter: Open context menu
- Esc: Close menu
- q: Quit

## Troubleshooting
- Ensure GITHUB_TOKEN has 'repo' permissions
- Check terminal supports colors: echo $TERM
- Verify network connectivity to github.com
- Make sure REPOS is set with valid owner/repo format
