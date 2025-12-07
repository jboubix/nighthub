use crate::config::settings::Settings;
use crate::error::AppError;
use crate::github::client::GithubClient;
use crate::github::models::{Repository, WorkflowRun};
use crate::ui::components::context_menu::ContextMenuComponent;
use crate::utils::logging::{log_error, log_info, log_warn};
use std::collections::HashMap;

use std::time::Duration;
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupType {
    ContextMenu,
    Logs,
}



pub struct AppState {
    pub repositories: Vec<Repository>,
    pub workflow_runs: HashMap<String, Vec<WorkflowRun>>,
    pub selected_repo: Option<usize>,
    pub selected_run: Option<usize>,
    pub popup: Option<PopupType>,
    pub context_menu: ContextMenuComponent,
    pub settings: Settings,
    pub github_client: GithubClient,
    pub last_repo_refresh_times: HashMap<String, DateTime<Utc>>,
    pub is_refreshing: bool,
}

impl AppState {
    /// Calculate refresh interval based on activity (pure tiered logic)
    fn calculate_refresh_interval(&self, repo_full_name: &str) -> Duration {
        // Tiered calculation based on activity
        if let Some(runs) = self.workflow_runs.get(repo_full_name) {
            if let Some(latest_run) = runs.first() {
                let now = Utc::now();
                let time_since_activity = now - latest_run.updated_at;
                
                if time_since_activity.num_seconds() < 7200 {  // <2 hours
                    Duration::from_secs(5)        // Very active: 5 seconds
                } else if time_since_activity.num_hours() < 24 {
                    Duration::from_secs(60)       // Moderately active: 1 minute
                } else {
                    Duration::from_secs(7200)     // Inactive: 2 hours
                }
            } else {
                // No workflow runs, treat as inactive
                Duration::from_secs(7200)
            }
        } else {
            // No workflow data, treat as inactive
            Duration::from_secs(7200)
        }
    }



pub async fn new_without_refresh(settings: Settings) -> Result<Self, AppError> {
    let github_client = GithubClient::new(settings.clone())?;
    let repositories = github_client.fetch_repositories().await?;

    Ok(AppState {
        repositories,
        workflow_runs: HashMap::new(),
        selected_repo: None,
        selected_run: None,
        popup: None,
        context_menu: ContextMenuComponent::new(),
        settings,
        github_client,
        last_repo_refresh_times: HashMap::new(),
        is_refreshing: false,
    })
}

pub async fn new(settings: Settings) -> Result<Self, AppError> {
    let mut app_state = AppState::new_without_refresh(settings).await?;
    // Initial refresh to populate workflow data and set proper timers
    let _ = app_state.refresh(true).await;
    Ok(app_state)
}



pub async fn refresh(&mut self, force_all: bool) -> Result<(), AppError> {
    self.is_refreshing = true;
    let now = Utc::now();
    
    // Collect repositories to refresh
    let repos_to_refresh: Vec<_> = if force_all {
        self.repositories.iter().cloned().collect()
    } else {
        self.repositories.iter()
            .filter(|repo| {
                if let Some(last_refresh_time) = self.last_repo_refresh_times.get(&repo.full_name) {
                    let refresh_interval = self.calculate_refresh_interval(&repo.full_name);
                    let time_since_refresh = now - *last_refresh_time;
                    let refresh_interval_chrono = chrono::Duration::from_std(refresh_interval)
                        .unwrap_or(chrono::Duration::seconds(5));
                    time_since_refresh >= refresh_interval_chrono
                } else {
                    // Never refreshed, so refresh now
                    true
                }
            })
            .cloned()
            .collect()
    };
    
    if repos_to_refresh.is_empty() {
        self.is_refreshing = false;
        return Ok(());
    }
    
    log_info(format!("Refreshing {} repositories (force_all={})", repos_to_refresh.len(), force_all));
    
    // Use a semaphore to limit concurrent requests
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
        self.settings.monitoring.max_concurrent_requests
    ));
    
    // Spawn parallel tasks for each repository with concurrency control
    let mut tasks = Vec::new();
    for repo in repos_to_refresh {
        let github_client = self.github_client.clone();
        let repo_name = repo.full_name.clone();
        let owner = repo.owner.clone();
        let name = repo.name.clone();
        let semaphore = std::sync::Arc::clone(&semaphore);
        
        let task = tokio::spawn(async move {
            // Acquire semaphore permit before making request
            let _permit = semaphore.acquire().await
                .map_err(|_| AppError::GithubError("Failed to acquire semaphore permit".to_string()))?;
            
            let runs = github_client
                .fetch_workflow_runs(&owner, &name)
                .await?;
            Ok::<(String, Vec<WorkflowRun>), AppError>((repo_name, runs))
        });
        tasks.push(task);
    }
    
    // Wait for all tasks to complete with timeout
    let results = tokio::time::timeout(
        Duration::from_secs(60), // Total timeout for all operations
        futures::future::join_all(tasks)
    ).await
    .map_err(|_| AppError::GithubError("Refresh operation timed out".to_string()))?;
    
    // Process results
    let mut success_count = 0;
    let mut error_count = 0;
    
    for result in results {
        match result {
            Ok(Ok((repo_name, runs))) => {
                self.workflow_runs.insert(repo_name.clone(), runs);
                self.last_repo_refresh_times.insert(repo_name, now);
                success_count += 1;
            }
            Ok(Err(e)) => {
                log_error(format!("Failed to refresh repository: {}", e));
                error_count += 1;
            }
            Err(e) => {
                log_error(format!("Task join error: {}", e));
                error_count += 1;
            }
        }
    }
    
    // Log summary if there were errors
    if error_count > 0 {
        log_warn(format!("Refresh completed: {} successful, {} failed", success_count, error_count));
    } else {
        log_info(format!("Refresh completed: {} successful", success_count));
    }
    
    self.is_refreshing = false;
    Ok(())
}

    pub fn seconds_until_refresh(&self) -> u64 {
        let now = Utc::now();
        let mut min_seconds_until_refresh = u64::MAX;
        
        for repo in &self.repositories {
            let refresh_interval = self.calculate_refresh_interval(&repo.full_name);
            
            if let Some(last_refresh_time) = self.last_repo_refresh_times.get(&repo.full_name) {
                let time_since_refresh = now - *last_refresh_time;
                let refresh_interval_chrono = chrono::Duration::from_std(refresh_interval)
                    .unwrap_or(chrono::Duration::seconds(5));
                
                if time_since_refresh >= refresh_interval_chrono {
                    // This repo needs refresh now
                    return 0;
                } else {
                    let seconds_until = refresh_interval_chrono.num_seconds() as u64 - time_since_refresh.num_seconds() as u64;
                    min_seconds_until_refresh = min_seconds_until_refresh.min(seconds_until);
                }
            } else {
                // Never refreshed, needs refresh now
                return 0;
            }
        }
        
        // If no repos, return 5 seconds as default
        if min_seconds_until_refresh == u64::MAX {
            5
        } else {
            min_seconds_until_refresh
        }
    }



    pub fn next_repo(&mut self) {
        let repo_count = self.repositories.len();
        if repo_count > 0 {
            let current_index = self.selected_repo.unwrap_or(0);
            self.selected_repo = Some((current_index + 1) % repo_count);
            self.selected_run = None;
        }
    }

    pub fn previous_repo(&mut self) {
        let repo_count = self.repositories.len();
        if repo_count > 0 {
            let current_index = self.selected_repo.unwrap_or(0);
            self.selected_repo = Some((current_index + repo_count - 1) % repo_count);
            self.selected_run = None;
        }
    }

    pub fn next_run(&mut self) {
        if let Some(repo_index) = self.selected_repo {
            if repo_index < self.repositories.len() {
                let repo_name = &self.repositories[repo_index].full_name;
                if let Some(runs) = self.workflow_runs.get(repo_name) {
                    let run_count = runs.len();
                    if run_count > 0 {
                        let current_index = self.selected_run.unwrap_or(0);
                        self.selected_run = Some((current_index + 1) % run_count);
                    }
                }
            }
        }
    }

    pub fn previous_run(&mut self) {
        if let Some(repo_index) = self.selected_repo {
            if repo_index < self.repositories.len() {
                let repo_name = &self.repositories[repo_index].full_name;
                if let Some(runs) = self.workflow_runs.get(repo_name) {
                    let run_count = runs.len();
                    if run_count > 0 {
                        let current_index = self.selected_run.unwrap_or(0);
                        self.selected_run = Some((current_index + run_count - 1) % run_count);
                    }
                }
            }
        }
    }

    pub fn open_context_menu(&mut self) {
        self.popup = Some(PopupType::ContextMenu);
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
    }

    pub fn get_selected_run_url(&self) -> Option<String> {
        if let (Some(repo_index), Some(run_index)) = (self.selected_repo, self.selected_run) {
            if repo_index < self.repositories.len() {
                let repo_name = &self.repositories[repo_index].full_name;
                if let Some(runs) = self.workflow_runs.get(repo_name) {
                    if run_index < runs.len() {
                        return Some(runs[run_index].html_url.clone());
                    }
                }
            }
        }
        None
    }

    pub fn open_in_browser(&self) -> Result<(), AppError> {
        if let Some(url) = self.get_selected_run_url() {
            webbrowser::open(&url)
                .map_err(|e| AppError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        }
        Ok(())
    }

    // Removed handle_ui_event since timer is computed dynamically
// UiEvent enum can be removed if not used elsewhere



    pub fn handle_key(&mut self, key: &str) {
        match key {
            "j" | "down" => {
                if self.popup.is_none() {
                    self.next_repo();
                } else if self.popup == Some(PopupType::ContextMenu) {
                    self.context_menu.next();
                }
            }
            "k" | "up" => {
                if self.popup.is_none() {
                    self.previous_repo();
                } else if self.popup == Some(PopupType::ContextMenu) {
                    self.context_menu.previous();
                }
            }
            "l" | "right" => {
                if self.popup.is_none() {
                    self.next_run();
                }
            }
            "h" | "left" => {
                if self.popup.is_none() {
                    self.previous_run();
                }
            }
            "enter" => {
                if self.popup.is_none() {
                    self.open_context_menu();
                } else if self.popup == Some(PopupType::ContextMenu) {
                    match self.context_menu.get_selected_action() {
                        "Open in Browser" => {
                            let _ = self.open_in_browser();
                            self.close_popup();
                        }
                        "Close Menu" => {
                            self.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            "esc" => {
                self.close_popup();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::{Settings, RepositoryConfig, MonitoringConfig};
    use crate::github::models::{Repository, WorkflowRun, WorkflowStatus, WorkflowConclusion};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::time::Duration;


    fn create_test_settings(repos: Vec<RepositoryConfig>) -> Settings {
        Settings {
            github_token: "test_token".to_string(),
            repositories: repos,
            monitoring: MonitoringConfig::default(),
            ui: crate::config::settings::UiConfig::default(),
            logging: crate::config::settings::LoggingConfig::default(),
        }
    }

fn create_test_app_state() -> AppState {
        let repos = vec![
            Repository {
                id: 1,
                name: "repo1".to_string(),
                owner: "owner1".to_string(),
                full_name: "owner1/repo1".to_string(),
                html_url: "https://github.com/owner1/repo1".to_string(),
                default_branch: Some("main".to_string()),
            },
            Repository {
                id: 2,
                name: "repo2".to_string(),
                owner: "owner2".to_string(),
                full_name: "owner2/repo2".to_string(),
                html_url: "https://github.com/owner2/repo2".to_string(),
                default_branch: Some("main".to_string()),
            },
        ];

        let settings = create_test_settings(vec![
            RepositoryConfig {
                owner: "owner1".to_string(),
                name: "repo1".to_string(),
                branch: None,
                workflows: None,
                enabled: true,
                refresh_interval_seconds: None,
            },
            RepositoryConfig {
                owner: "owner2".to_string(),
                name: "repo2".to_string(),
                branch: None,
                workflows: None,
                enabled: true,
                refresh_interval_seconds: Some(300), // 5 minutes
            },
        ]);

        let mut workflow_runs = HashMap::new();
        let now = Utc::now();
        
        // Add recent workflow run for repo1 (should get 5s refresh)
        workflow_runs.insert("owner1/repo1".to_string(), vec![
            WorkflowRun {
                id: 1,
                name: "CI".to_string(),
                status: WorkflowStatus::Completed,
                conclusion: Some(WorkflowConclusion::Success),
                created_at: now,
                updated_at: now,
                branch: "main".to_string(),
                commit_sha: "abc123".to_string(),
                actor: "user1".to_string(),
                html_url: "https://github.com/owner1/repo1/run/1".to_string(),
                logs_url: Some("https://github.com/owner1/repo1/run/1/logs".to_string()),
            }
        ]);

        // Add old workflow run for repo2 (should get 2hr refresh, but override to 5min)
        workflow_runs.insert("owner2/repo2".to_string(), vec![
            WorkflowRun {
                id: 2,
                name: "CI".to_string(),
                status: WorkflowStatus::Completed,
                conclusion: Some(WorkflowConclusion::Success),
                created_at: now - chrono::Duration::hours(25), // 25 hours ago
                updated_at: now - chrono::Duration::hours(25),
                branch: "main".to_string(),
                commit_sha: "def456".to_string(),
                actor: "user2".to_string(),
                html_url: "https://github.com/owner2/repo2/run/2".to_string(),
                logs_url: Some("https://github.com/owner2/repo2/run/2/logs".to_string()),
            }
        ]);

        AppState {
            repositories: repos,
            workflow_runs,
            selected_repo: None,
            selected_run: None,
            popup: None,
            context_menu: crate::ui::components::context_menu::ContextMenuComponent::new(),
            settings: settings.clone(),
            github_client: crate::github::client::GithubClient::new(settings.clone()).unwrap(),
            last_repo_refresh_times: HashMap::new(),
            ui_tx: mpsc::unbounded_channel().0,
            seconds_until_refresh: 0,
            is_refreshing: false,
        }
    }

    #[tokio::test]
    async fn test_calculate_refresh_interval_very_active() {
        let app_state = create_test_app_state();
        let interval = app_state.calculate_refresh_interval("owner1/repo1");
        assert_eq!(interval, Duration::from_secs(5)); // Very recent activity
    }

    #[tokio::test]
    async fn test_calculate_refresh_interval_with_override() {
        let app_state = create_test_app_state();
        let interval = app_state.calculate_refresh_interval("owner2/repo2");
        assert_eq!(interval, Duration::from_secs(300)); // Override takes precedence
    }

    #[tokio::test]
    async fn test_calculate_refresh_interval_moderately_active() {
        let mut app_state = create_test_app_state();
        
        // Update repo1 to have activity 12 hours ago (should get 1min refresh)
        let twelve_hours_ago = Utc::now() - chrono::Duration::hours(12);
        if let Some(runs) = app_state.workflow_runs.get_mut("owner1/repo1") {
            for run in runs {
                run.updated_at = twelve_hours_ago;
            }
        }
        
        let interval = app_state.calculate_refresh_interval("owner1/repo1");
        assert_eq!(interval, Duration::from_secs(60)); // Moderately active
    }

    #[tokio::test]
    async fn test_calculate_refresh_interval_inactive() {
        let mut app_state = create_test_app_state();
        
        // Update repo1 to have activity 25 hours ago (should get 2hr refresh)
        let twenty_five_hours_ago = Utc::now() - chrono::Duration::hours(25);
        if let Some(runs) = app_state.workflow_runs.get_mut("owner1/repo1") {
            for run in runs {
                run.updated_at = twenty_five_hours_ago;
            }
        }
        
        let interval = app_state.calculate_refresh_interval("owner1/repo1");
        assert_eq!(interval, Duration::from_secs(7200)); // Inactive
    }

    #[tokio::test]
    async fn test_calculate_refresh_interval_no_workflow_data() {
        let mut app_state = create_test_app_state();
        
        // Remove workflow data for repo1
        app_state.workflow_runs.remove("owner1/repo1");
        
        let interval = app_state.calculate_refresh_interval("owner1/repo1");
        assert_eq!(interval, Duration::from_secs(7200)); // No data = inactive
    }

    #[tokio::test]
    async fn test_calculate_refresh_interval_unknown_repo() {
        let app_state = create_test_app_state();
        
        let interval = app_state.calculate_refresh_interval("unknown/repo");
        assert_eq!(interval, Duration::from_secs(7200)); // Unknown repo = inactive
    }

    #[tokio::test]
    async fn test_seconds_until_refresh_never_refreshed() {
        let app_state = create_test_app_state();
        
        let seconds = app_state.seconds_until_refresh();
        assert_eq!(seconds, 0); // Never refreshed = needs refresh now
    }

    #[tokio::test]
    async fn test_seconds_until_refresh_some_refreshed() {
        let mut app_state = create_test_app_state();
        
        // Set some refresh times
        let now = Utc::now();
        app_state.last_repo_refresh_times.insert("owner1/repo1".to_string(), now);
        app_state.last_repo_refresh_times.insert("owner2/repo2".to_string(), now);
        
        let seconds = app_state.seconds_until_refresh();
        assert_eq!(seconds, 5); // Minimum of 5s and 300s
    }

    #[tokio::test]
    async fn test_seconds_until_refresh_mixed_times() {
        let mut app_state = create_test_app_state();
        
        let now = Utc::now();
        let four_seconds_ago = now - chrono::Duration::seconds(4);
        let two_minutes_ago = now - chrono::Duration::seconds(120);
        
        app_state.last_repo_refresh_times.insert("owner1/repo1".to_string(), four_seconds_ago);
        app_state.last_repo_refresh_times.insert("owner2/repo2".to_string(), two_minutes_ago);
        
        let seconds = app_state.seconds_until_refresh();
        assert_eq!(seconds, 1); // repo1 needs refresh in 1 second (5s - 4s elapsed)
    }

    #[tokio::test]
    async fn test_seconds_until_refresh_all_need_refresh() {
        let mut app_state = create_test_app_state();
        
        let long_ago = Utc::now() - chrono::Duration::hours(1);
        app_state.last_repo_refresh_times.insert("owner1/repo1".to_string(), long_ago);
        app_state.last_repo_refresh_times.insert("owner2/repo2".to_string(), long_ago);
        
        let seconds = app_state.seconds_until_refresh();
        assert_eq!(seconds, 0); // Both need refresh now
    }

    #[tokio::test]
    async fn test_next_repo_navigation() {
        let mut app_state = create_test_app_state();
        
        app_state.next_repo();
        assert_eq!(app_state.selected_repo, Some(1));
        
        app_state.next_repo();
        assert_eq!(app_state.selected_repo, Some(0)); // Wrap around
        
        app_state.selected_run = Some(5); // Should be reset
        app_state.next_repo();
        assert_eq!(app_state.selected_run, None);
    }

    #[tokio::test]
    async fn test_previous_repo_navigation() {
        let mut app_state = create_test_app_state();
        
        app_state.previous_repo();
        assert_eq!(app_state.selected_repo, Some(1)); // Wrap to end
        
        app_state.previous_repo();
        assert_eq!(app_state.selected_repo, Some(0));
        
        app_state.selected_run = Some(5); // Should be reset
        app_state.previous_repo();
        assert_eq!(app_state.selected_run, None);
    }

    #[tokio::test]
    async fn test_next_run_navigation() {
        let mut app_state = create_test_app_state();
        app_state.selected_repo = Some(0);
        
        // Should not crash with single run
        app_state.next_run();
        assert_eq!(app_state.selected_run, Some(0));
    }

    #[tokio::test]
    async fn test_previous_run_navigation() {
        let mut app_state = create_test_app_state();
        app_state.selected_repo = Some(0);
        
        // Should not crash with single run
        app_state.previous_run();
        assert_eq!(app_state.selected_run, Some(0));
    }

    #[tokio::test]
    async fn test_navigation_no_repos() {
        let mut app_state = AppState {
            repositories: vec![],
            workflow_runs: HashMap::new(),
            selected_repo: None,
            selected_run: None,
            popup: None,
            context_menu: crate::ui::components::context_menu::ContextMenuComponent::new(),
            settings: create_test_settings(vec![]),
            github_client: crate::github::client::GithubClient::new(create_test_settings(vec![])).unwrap(),
            last_repo_refresh_times: HashMap::new(),
            ui_tx: mpsc::unbounded_channel().0,
            seconds_until_refresh: 0,
            is_refreshing: false,
        };
        
        // Should not panic with no repositories
        app_state.next_repo();
        app_state.previous_repo();
        app_state.next_run();
        app_state.previous_run();
        
        assert_eq!(app_state.selected_repo, None);
        assert_eq!(app_state.selected_run, None);
    }

    #[tokio::test]
    async fn test_get_selected_run_url() {
        let mut app_state = create_test_app_state();
        app_state.selected_repo = Some(0);
        app_state.selected_run = Some(0);
        
        let url = app_state.get_selected_run_url();
        assert!(url.is_some());
        assert_eq!(url.unwrap(), "https://github.com/owner1/repo1/run/1");
    }

    #[tokio::test]
    async fn test_get_selected_run_url_no_selection() {
        let mut app_state = create_test_app_state();
        app_state.selected_repo = None;
        app_state.selected_run = None;
        
        let url = app_state.get_selected_run_url();
        assert!(url.is_none());
    }

    #[tokio::test]
    async fn test_get_selected_run_url_invalid_indices() {
        let mut app_state = create_test_app_state();
        app_state.selected_repo = Some(10); // Invalid
        app_state.selected_run = Some(0);
        
        let url = app_state.get_selected_run_url();
        assert!(url.is_none());
    }

    #[tokio::test]
    async fn test_popup_management() {
        let mut app_state = create_test_app_state();
        
        assert_eq!(app_state.popup, None);
        
        app_state.open_context_menu();
        assert_eq!(app_state.popup, Some(PopupType::ContextMenu));
        
        app_state.close_popup();
        assert_eq!(app_state.popup, None);
    }

    #[tokio::test]
    async fn test_handle_key_navigation() {
        let mut app_state = create_test_app_state();
        
        app_state.handle_key("j");
        assert_eq!(app_state.selected_repo, Some(1));
        
        app_state.handle_key("k");
        assert_eq!(app_state.selected_repo, Some(0));
        
        app_state.selected_repo = Some(0);
        app_state.handle_key("l");
        assert_eq!(app_state.selected_run, Some(0));
        
        app_state.handle_key("h");
        assert_eq!(app_state.selected_run, Some(0));
    }

    #[tokio::test]
    async fn test_handle_key_popup() {
        let mut app_state = create_test_app_state();
        
        app_state.handle_key("enter");
        assert_eq!(app_state.popup, Some(PopupType::ContextMenu));
        
        app_state.handle_key("esc");
        assert_eq!(app_state.popup, None);
    }

    #[tokio::test]
    async fn test_handle_key_popup_navigation() {
        let mut app_state = create_test_app_state();
        app_state.open_context_menu();
        
        // Store initial action value to avoid borrow issues
        let initial_action_value = format!("{:?}", app_state.context_menu.get_selected_action());
        
        app_state.handle_key("j"); // Should navigate context menu
        let new_action_value = format!("{:?}", app_state.context_menu.get_selected_action());
        assert_ne!(initial_action_value, new_action_value);
    }

    #[tokio::test]
    async fn test_seconds_until_refresh_no_repos() {
        let app_state = AppState {
            repositories: vec![],
            workflow_runs: HashMap::new(),
            selected_repo: None,
            selected_run: None,
            popup: None,
            context_menu: crate::ui::components::context_menu::ContextMenuComponent::new(),
            settings: create_test_settings(vec![]),
            github_client: crate::github::client::GithubClient::new(create_test_settings(vec![])).unwrap(),
            last_repo_refresh_times: HashMap::new(),
            ui_tx: mpsc::unbounded_channel().0,
            seconds_until_refresh: 0,
            is_refreshing: false,
        };
        
        let seconds = app_state.seconds_until_refresh();
        assert_eq!(seconds, 5); // Default when no repos
    }

    #[tokio::test]
    async fn test_seconds_until_refresh_just_refreshed() {
        let mut app_state = create_test_app_state();
        let now = Utc::now();
        
        // Set refresh times to now
        app_state.last_repo_refresh_times.insert("owner1/repo1".to_string(), now);
        app_state.last_repo_refresh_times.insert("owner2/repo2".to_string(), now);
        
        let seconds = app_state.seconds_until_refresh();
        // Should return the minimum refresh interval (5 seconds for very active)
        assert_eq!(seconds, 5);
    }

    #[tokio::test]
    async fn test_seconds_until_refresh_partial_time_elapsed() {
        let mut app_state = create_test_app_state();
        let now = Utc::now();
        let four_seconds_ago = now - chrono::Duration::seconds(4);
        
        // Set repo1 refreshed 4 seconds ago (5 second interval)
        app_state.last_repo_refresh_times.insert("owner1/repo1".to_string(), four_seconds_ago);
        // Set repo2 refreshed now (300 second interval due to override)
        app_state.last_repo_refresh_times.insert("owner2/repo2".to_string(), now);
        
        let seconds = app_state.seconds_until_refresh();
        assert_eq!(seconds, 1); // repo1 needs refresh in 1 second
    }

    #[tokio::test]
    async fn test_handle_ui_event_timer_update() {
        let mut app_state = create_test_app_state();
        
        // Initial state (create_test_app_state sets it to 0)
        assert_eq!(app_state.seconds_until_refresh, 0);
        
        // Handle timer update event
        app_state.handle_ui_event(UiEvent::TimerUpdate(30));
        
        assert_eq!(app_state.seconds_until_refresh, 30);
    }

    #[tokio::test]
    async fn test_handle_ui_event_workflow_runs_updated() {
        let mut app_state = create_test_app_state();
        let test_runs = vec![WorkflowRun {
            id: 999,
            name: "Test Workflow".to_string(),
            branch: "main".to_string(),
            commit_sha: "abc123".to_string(),
            actor: "testuser".to_string(),
            status: WorkflowStatus::Completed,
            conclusion: Some(WorkflowConclusion::Success),
            html_url: "https://github.com/test/repo/run/999".to_string(),
            logs_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        
        // Handle workflow runs updated event
        app_state.handle_ui_event(UiEvent::WorkflowRunsUpdated("owner1/repo1".to_string(), test_runs.clone()));
        
        // Check that workflow runs were updated
        assert!(app_state.workflow_runs.contains_key("owner1/repo1"));
        let stored_runs = app_state.workflow_runs.get("owner1/repo1").unwrap();
        assert_eq!(stored_runs.len(), 1);
        assert_eq!(stored_runs[0].id, 999);
        
        // Check that refresh time was updated
        assert!(app_state.last_repo_refresh_times.contains_key("owner1/repo1"));
        
        // Check that timer was set to refresh interval (5 seconds for very active repo)
        assert_eq!(app_state.seconds_until_refresh, 5);
    }

    #[tokio::test]
    async fn test_handle_ui_event_workflow_runs_updated_with_override() {
        let mut app_state = create_test_app_state();
        let test_runs = vec![WorkflowRun {
            id: 999,
            name: "Test Workflow".to_string(),
            branch: "main".to_string(),
            commit_sha: "abc123".to_string(),
            actor: "testuser".to_string(),
            status: WorkflowStatus::Completed,
            conclusion: Some(WorkflowConclusion::Success),
            html_url: "https://github.com/test/repo/run/999".to_string(),
            logs_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        
        // Handle workflow runs updated event for repo2 (has 300 second override)
        app_state.handle_ui_event(UiEvent::WorkflowRunsUpdated("owner2/repo2".to_string(), test_runs.clone()));
        
        // Check that timer was set to override interval (300 seconds)
        assert_eq!(app_state.seconds_until_refresh, 300);
    }

    #[tokio::test]
    async fn test_calculate_refresh_interval_very_active_new() {
        let app_state = create_test_app_state();
        
        // repo1 has very recent activity (should get 5s refresh)
        let interval = app_state.calculate_refresh_interval("owner1/repo1");
        assert_eq!(interval.as_secs(), 5);
    }

    #[tokio::test]
    async fn test_calculate_refresh_interval_with_override_new() {
        let app_state = create_test_app_state();
        
        // repo2 has override to 5 minutes (300s)
        let interval = app_state.calculate_refresh_interval("owner2/repo2");
        assert_eq!(interval.as_secs(), 300);
    }

    #[tokio::test]
    async fn test_calculate_refresh_interval_unknown_repo_new() {
        let app_state = create_test_app_state();
        
        // Unknown repo should get inactive refresh (2 hours = 7200s)
        let interval = app_state.calculate_refresh_interval("unknown/repo");
        assert_eq!(interval.as_secs(), 7200);
    }
}
