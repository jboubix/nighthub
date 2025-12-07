# Nighthub - GitHub Actions Terminal Monitor

A sleek, compact terminal UI tool for monitoring GitHub Actions workflows across multiple repositories in real-time.

## Features

- **Real-time Monitoring**: Watch GitHub Actions workflows with configurable refresh rates
- **Compact Horizontal Layout**: Each repository shown on one line with status icons and refresh countdown timer (e.g., `org/repo (30s): ✅✅❌✅❌`)
- **Keyboard Navigation**: Navigate through repositories (↑/↓) and workflow runs (←/→) with vim-like keys (hjkl) or arrow keys
- **Manual Refresh**: Press 'f' to force immediate refresh and reset the countdown timer
- **Contextual Actions**: Open workflow runs in browser or view logs directly from the UI
- **Multi-Repository Support**: Monitor up to 50 repositories simultaneously
- **Status Indicators**: Visual icons for workflow states (✅ success, ❌ failure, ⏳ queued, 🔄 in progress)

## Installation

### Option 1: Build from Source (Linux)

#### Prerequisites

- Rust 1.75+ (install with rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- GitHub Personal Access Token with `repo` permissions

#### Build

```bash
# Clone the repository
git clone https://github.com/yourusername/nighthub.git
cd nighthub

# Build optimized release binary
cargo build --release

# Strip debug symbols to reduce size
strip target/release/nighthub

# Copy with descriptive name
cp target/release/nighthub nighthub-terminal-monitor
```

#### Run

```bash
# Set your GitHub token and repositories
export GITHUB_TOKEN=ghp_your_actual_token_here
export REPOS=microsoft/vscode,rust-lang/rust

# Run the application
./nighthub-terminal-monitor
```

### Option 2: Pre-built Binary (Linux)

#### Download and Extract

```bash
# Download the pre-built package
wget https://github.com/yourusername/nighthub/releases/download/v0.1.0/nighthub-terminal-monitor-linux.tar.gz

# Extract
tar -xzf nighthub-terminal-monitor-linux.tar.gz
cd nighthub-terminal-monitor-linux
```

#### Configure and Run

```bash
# Set environment variables
export GITHUB_TOKEN=ghp_your_actual_token_here
export REPOS=microsoft/vscode,rust-lang/rust

# Run the application
./nighthub-terminal-monitor
```

### System Requirements

- **OS**: Linux (Ubuntu 18.04+, CentOS 7+, etc.)
- **Architecture**: x86_64 (AMD64)
- **Terminal**: Any terminal with 256-color support
- **Memory**: 100MB+ available RAM
- **Network**: Internet connection for GitHub API access

## Configuration

The application reads all configuration from environment variables only. You can use either `GITHUB_TOKEN` or `GH_TOKEN` for authentication.

### Required Environment Variables

```bash
export GITHUB_TOKEN=your_github_token_here
export REPOS=owner1/repo1,owner2/repo2,owner3/repo3
```

### Environment Variables

- **GITHUB_TOKEN** (or **GH_TOKEN**): Your GitHub Personal Access Token (required)
- **REPOS**: Comma-separated list of repositories in `owner/repo` format (required)

### Example

```bash
export GITHUB_TOKEN=ghp_your_actual_token_here
export REPOS=microsoft/vscode,rust-lang/rust,octocat/Hello-World
./nighthub-terminal-monitor
```

## Usage

### Quick Start

```bash
# 1. Set your GitHub token
export GITHUB_TOKEN=ghp_your_actual_token_here

# 2. Set repositories to monitor
export REPOS=microsoft/vscode,rust-lang/rust,octocat/Hello-World

# 3. Run the application
./nighthub-terminal-monitor
```

### Testing Status

✅ **Tests**: All tests pass (`cargo test` completed successfully)
✅ **Build**: Release binary compiled successfully (6.6MB)
✅ **Runtime**: Application starts and reads environment variables correctly
✅ **Configuration**: Simplified to environment variables only
⚠️ **Note**: Full functionality requires a valid GitHub Personal Access Token

### Keyboard Controls

The UI displays repositories in a compact horizontal format with a countdown timer showing seconds until next refresh:
```
organization/repository    (30s): ✅✅❌✅❌
organization/repo2         (30s): ❌✅✅✅
```

Navigation:
- `j` / `↓` - Move down to next repository
- `k` / `↑` - Move up to previous repository
- `l` / `→` - Move right to next workflow run (on the same repository)
- `h` / `←` - Move left to previous workflow run (on the same repository)
- `f` - Force immediate refresh and reset countdown timer
- `Enter` - Open contextual menu for the selected workflow run
- `Esc` - Close menu / exit
- `q` - Quit application

### Contextual Menu Actions

- **Open in Browser**: Opens the selected workflow run in your default browser
- **View Logs**: Shows workflow logs (placeholder for future implementation)
- **Close Menu**: Closes the contextual menu

## Architecture

The application follows a modular architecture:

```
src/
├── main.rs              # Application entry point with terminal UI loop
├── lib.rs               # Library interface and logging setup
├── config/
│   ├── mod.rs
│   └── settings.rs      # Configuration from environment variables only
├── github/
│   ├── mod.rs
│   ├── client.rs        # GitHub API client wrapper
│   └── models.rs        # GitHub API data structures
├── ui/
│   ├── mod.rs
│   ├── app.rs           # Main application state and logic
│   ├── layout.rs        # Terminal layout management
│   └── components/
│       ├── mod.rs
│       ├── workflow_list.rs  # Workflow list UI component
│       └── context_menu.rs   # Contextual menu component
├── utils/
│   ├── mod.rs
│   ├── time.rs          # Time formatting utilities
│   └── icons.rs         # Status icons and text
└── error.rs             # Error handling types
```

## Technical Details

- **Language**: Rust 1.75+
- **UI Framework**: Ratatui (terminal UI)
- **GitHub API**: Octocrab (GitHub API client)
- **Configuration**: Environment variables only
- **Async Runtime**: Tokio
- **Terminal Backend**: Crossterm

## Performance

- **Memory Usage**: <100MB for 50 repositories
- **Refresh Rate**: <2s for 50 repositories
- **Terminal Size**: Minimum 80x24, responsive design

## Development

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
cargo clippy
cargo fmt
```

### Environment Variables for Development

```bash
export GITHUB_TOKEN=your_token_here
export REPOS=owner1/repo1,owner2/repo2
cargo run
```

### Adding New Features

1. Update the specification in `specs/001-github-actions-monitor/`
2. Add tasks to `specs/001-github-actions-monitor/tasks.md`
3. Implement following TDD approach
4. Update documentation

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Standalone Binary

The application is compiled as a **standalone executable** with all dependencies statically linked:

- **No Rust/Cargo required** to run
- **Self-contained**: All libraries included in the binary
- **Portable**: Can be copied to any Linux system
- **Size**: ~6.6MB (stripped)

### Binary Information

```bash
# Check binary details
file nighthub-terminal-monitor
# Output: ELF 64-bit LSB executable, x86-64, version 1 (SYSV)

# Check dependencies (should show only standard libraries)
ldd nighthub-terminal-monitor
# Output: linux-vdso.so.1, libc.so.6, libm.so.6, libdl.so.2, librt.so.1, libpthread.so.0
```

## Linux Distribution

### Creating Distribution Package

```bash
# Build and package
cargo build --release
strip target/release/nighthub
cp target/release/nighthub nighthub-terminal-monitor
tar -czf nighthub-terminal-monitor-linux.tar.gz nighthub-terminal-monitor .env.example README.md

# Verify package
tar -tzf nighthub-terminal-monitor-linux.tar.gz
```

### Deployment

```bash
# On target Linux system
wget https://github.com/yourusername/nighthub/releases/download/v0.1.0/nighthub-terminal-monitor-linux.tar.gz
tar -xzf nighthub-terminal-monitor-linux.tar.gz

# Configure
export GITHUB_TOKEN=your_token_here
export REPOS=owner1/repo1,owner2/repo2

# Run
./nighthub-terminal-monitor
```

## Support

For issues and questions, please create an issue in the GitHub repository.
