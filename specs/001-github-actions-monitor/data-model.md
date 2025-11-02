# Data Model: GitHub Actions Terminal Monitor

**Date**: 2025-10-26  
**Purpose**: Define data structures and relationships for the monitoring system

## Core Entities

### WorkflowRun

Represents a single execution of a GitHub Actions workflow.

**Fields**:
- `id: u64` - Unique identifier for the workflow run
- `repository_id: String` - Repository identifier in "owner/repo" format
- `workflow_name: String` - Name of the workflow file
- `status: WorkflowStatus` - Current status of the run
- `conclusion: Option<WorkflowConclusion>` - Final result (if completed)
- `created_at: DateTime<Utc>` - When the run was created
- `updated_at: DateTime<Utc>` - Last update timestamp
- `branch: String` - Git branch the run was triggered on
- `commit_sha: String` - Commit hash that triggered the run
- `actor: String` - User or service that triggered the run
- `url: String` - GitHub URL for the run
- `logs_url: Option<String>` - URL to download logs (if available)

**Validation Rules**:
- `id` must be > 0
- `repository_id` must be in "owner/repo" format
- `workflow_name` cannot be empty
- `branch` cannot be empty
- `commit_sha` must be a valid Git SHA (40 hex characters)

**State Transitions**:
```
queued → in_progress → completed
                ↓
             completed (with conclusion: success/failure/cancelled)
```

### Repository

Represents a GitHub repository being monitored.

**Fields**:
- `owner: String` - Repository owner (user or organization)
- `name: String` - Repository name
- `default_branch: String` - Default branch (usually "main" or "master")
- `is_monitored: bool` - Whether this repo is actively monitored
- `last_updated: Option<DateTime<Utc>>` - Last successful data fetch
- `error_count: u32` - Number of consecutive errors

**Validation Rules**:
- `owner` cannot be empty and must contain valid GitHub username characters
- `name` cannot be empty and must be a valid repository name
- `default_branch` cannot be empty

### Configuration

Represents application configuration and settings.

**Fields**:
- `github_token: Secret<String>` - GitHub personal access token
- `repositories: Vec<RepositoryConfig>` - List of repositories to monitor
- `refresh_interval: Duration` - How often to refresh data
- `max_repositories: usize` - Maximum number of repositories to monitor
- `ui_theme: ThemeConfig` - Terminal UI appearance settings
- `log_level: LogLevel` - Application logging level

**Validation Rules**:
- `github_token` must be a valid GitHub token format
- `repositories` list cannot be empty (unless auto-discovery is enabled)
- `refresh_interval` must be at least 10 seconds
- `max_repositories` must be between 1 and 100

### RepositoryConfig

Configuration for a single repository to monitor.

**Fields**:
- `owner: String` - Repository owner
- `name: String` - Repository name
- `branch: Option<String>` - Specific branch to monitor (None = default branch)
- `workflows: Option<Vec<String>>` - Specific workflows to monitor (None = all)
- `enabled: bool` - Whether monitoring is enabled for this repo

**Validation Rules**:
- `owner` and `name` cannot be empty
- If specified, `branch` must be a valid branch name
- If specified, `workflows` list cannot be empty

## Enums

### WorkflowStatus

Current status of a workflow run.

**Variants**:
- `Queued` - Waiting to run
- `InProgress` - Currently running
- `Completed` - Finished (check conclusion for result)

### WorkflowConclusion

Final result of a completed workflow run.

**Variants**:
- `Success` - All jobs completed successfully
- `Failure` - One or more jobs failed
- `Cancelled` - Run was cancelled
- `Skipped` - Run was skipped
- `TimedOut` - Run exceeded time limit

### LogLevel

Application logging verbosity.

**Variants**:
- `Error` - Only error messages
- `Warn` - Errors and warnings
- `Info` - Normal informational messages
- `Debug` - Detailed debugging information
- `Trace` - Most verbose logging

## UI State Models

### AppState

Main application state for the terminal UI.

**Fields**:
- `repositories: Vec<RepositoryState>` - Current state of all monitored repositories
- `selected_repo_index: usize` - Currently selected repository
- `selected_run_index: usize` - Currently selected workflow run
- `show_popup: bool` - Whether a popup menu is displayed
- `popup_type: PopupType` - Type of popup to show
- `last_refresh: DateTime<Utc>` - When data was last refreshed
- `refresh_in_progress: bool` - Whether data refresh is currently happening
- `error_message: Option<String>` - Current error to display

### RepositoryState

UI state for a single repository.

**Fields**:
- `config: RepositoryConfig` - Repository configuration
- `workflow_runs: Vec<WorkflowRun>` - Last 4 workflow runs (sorted by creation time)
- `status: RepositoryStatus` - Overall repository status
- `last_updated: Option<DateTime<Utc>>` - Last successful update
- `error: Option<String>` - Current error message

### RepositoryStatus

Overall status of a repository based on its workflow runs.

**Variants**:
- `Healthy` - All recent runs successful
- `Warning` - Some recent runs failed or cancelled
- `Error` - Most recent run failed
- `Unknown` - No recent runs or error fetching data

## Data Relationships

```
Configuration
├── RepositoryConfig (1..*)
│   └── Repository (1:1)
│       └── WorkflowRun (1..4, most recent)
└── AppState (1:1)
    └── RepositoryState (1..*)
        ├── RepositoryConfig (1:1)
        └── WorkflowRun (0..4)
```

## Data Flow

1. **Configuration Loading**: Load from config file and environment variables
2. **Repository Discovery**: Validate and initialize repository configurations
3. **Data Fetching**: Periodically fetch workflow runs from GitHub API
4. **State Update**: Update UI state with new data
5. **UI Rendering**: Render terminal interface based on current state

## Performance Considerations

### Memory Management
- Limit to last 4 workflow runs per repository (as per requirements)
- Use efficient data structures (Vec, HashMap) for lookups
- Clean up old data to prevent memory leaks

### Caching Strategy
- Cache workflow definitions to reduce API calls
- Cache repository metadata (default branch, etc.)
- Use in-memory caching with TTL for API responses

### Concurrency
- Use async/await for all API operations
- Limit concurrent API calls to respect rate limits
- Use channels for communication between data fetcher and UI

## Error Handling

### Error Types
- `ConfigurationError` - Invalid configuration
- `ApiError` - GitHub API errors (rate limiting, auth, etc.)
- `NetworkError` - Network connectivity issues
- `UiError` - Terminal UI errors

### Error Recovery
- Automatic retry with exponential backoff for transient errors
- Graceful degradation when repositories are unavailable
- Clear error messages with suggested actions

## Validation Rules Summary

### Input Validation
- All string fields must be non-empty unless explicitly optional
- GitHub tokens must match expected format patterns
- Repository identifiers must follow GitHub naming conventions
- URLs must be valid and accessible

### Business Logic Validation
- Maximum 50 repositories per configuration
- Maximum 4 workflow runs displayed per repository
- Minimum 10-second refresh interval
- Valid date/time ranges for timestamps

This data model provides a solid foundation for implementing the GitHub Actions Terminal Monitor with clear separation of concerns, proper validation, and efficient data handling.