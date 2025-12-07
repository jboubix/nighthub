use chrono::{DateTime, Utc};
use crate::github::models::{WorkflowRun, WorkflowConclusion};

#[derive(Debug, Clone)]
pub struct RepositoryState {
    pub config: crate::config::settings::RepositoryConfig,
    pub workflow_runs: Vec<WorkflowRun>,
    pub status: RepositoryStatus,
    pub last_updated: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepositoryStatus {
    Healthy,
    Warning,
    Error,
    Unknown,
}

impl RepositoryState {
    pub fn new(config: crate::config::settings::RepositoryConfig) -> Self {
        Self {
            config,
            workflow_runs: Vec::new(),
            status: RepositoryStatus::Unknown,
            last_updated: None,
            error: None,
        }
    }

    pub fn update_runs(&mut self, runs: Vec<WorkflowRun>) {
        self.workflow_runs = runs;
        self.last_updated = Some(Utc::now());

        // Update status based on recent runs
        if let Some(last_run) = self.workflow_runs.first() {
            self.status = match last_run.conclusion {
                Some(WorkflowConclusion::Success) => RepositoryStatus::Healthy,
                Some(WorkflowConclusion::Failure) => RepositoryStatus::Error,
                Some(_) => RepositoryStatus::Warning,
                None => RepositoryStatus::Unknown,
            };
        } else {
            self.status = RepositoryStatus::Unknown;
        }
    }

    pub fn has_recent_failure(&self) -> bool {
        self.workflow_runs.iter().any(|run| run.conclusion == Some(WorkflowConclusion::Failure))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::RepositoryConfig;
    use crate::github::models::{WorkflowRun, WorkflowStatus, WorkflowConclusion};
    use chrono::Utc;

    fn create_test_repo_config() -> RepositoryConfig {
        RepositoryConfig {
            owner: "testowner".to_string(),
            name: "testrepo".to_string(),
            branch: None,
            workflows: None,
            enabled: true,
            refresh_interval_seconds: None,
        }
    }

    fn create_test_workflow_run(id: u64, conclusion: Option<WorkflowConclusion>) -> WorkflowRun {
        WorkflowRun {
            id,
            name: "CI".to_string(),
            status: WorkflowStatus::Completed,
            conclusion,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            branch: "main".to_string(),
            commit_sha: "abc123".to_string(),
            actor: "testuser".to_string(),
            html_url: format!("https://github.com/test/repo/run/{}", id),
            logs_url: Some(format!("https://github.com/test/repo/run/{}/logs", id)),
        }
    }

    #[test]
    fn test_repository_state_new() {
        let config = create_test_repo_config();
        let state = RepositoryState::new(config);
        
        assert_eq!(state.config.owner, "testowner");
        assert_eq!(state.config.name, "testrepo");
        assert!(state.workflow_runs.is_empty());
        assert_eq!(state.status, RepositoryStatus::Unknown);
        assert!(state.last_updated.is_none());
        assert!(state.error.is_none());
    }

    #[test]
    fn test_update_runs_with_success() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let runs = vec![
            create_test_workflow_run(1, Some(WorkflowConclusion::Failure)),
            create_test_workflow_run(2, Some(WorkflowConclusion::Success)),
        ];
        
        state.update_runs(runs);
        
        assert_eq!(state.workflow_runs.len(), 2);
        assert!(state.last_updated.is_some());
        assert_eq!(state.status, RepositoryStatus::Error); // Last run is failure (first element)
    }

    #[test]
    fn test_update_runs_with_failure() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let runs = vec![
            create_test_workflow_run(1, Some(WorkflowConclusion::Success)),
            create_test_workflow_run(2, Some(WorkflowConclusion::Success)),
        ];
        
        state.update_runs(runs);
        
        assert_eq!(state.workflow_runs.len(), 2);
        assert_eq!(state.status, RepositoryStatus::Healthy);
    }

    #[test]
    fn test_update_runs_with_warning() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let runs = vec![
            create_test_workflow_run(1, Some(WorkflowConclusion::Cancelled)),
        ];
        
        state.update_runs(runs);
        
        assert_eq!(state.workflow_runs.len(), 1);
        assert_eq!(state.status, RepositoryStatus::Warning);
    }

    #[test]
    fn test_update_runs_empty() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let runs: Vec<WorkflowRun> = vec![];
        state.update_runs(runs);
        
        assert!(state.workflow_runs.is_empty());
        assert_eq!(state.status, RepositoryStatus::Unknown);
    }

    #[test]
    fn test_update_runs_timestamp() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let before_update = Utc::now();
        let runs = vec![create_test_workflow_run(1, Some(WorkflowConclusion::Success))];
        
        state.update_runs(runs);
        let after_update = Utc::now();
        
        assert!(state.last_updated.is_some());
        let last_updated = state.last_updated.unwrap();
        assert!(last_updated >= before_update);
        assert!(last_updated <= after_update);
    }

    #[test]
    fn test_has_recent_failure_true() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let runs = vec![
            create_test_workflow_run(1, Some(WorkflowConclusion::Success)),
            create_test_workflow_run(2, Some(WorkflowConclusion::Failure)),
            create_test_workflow_run(3, Some(WorkflowConclusion::Success)),
        ];
        
        state.update_runs(runs);
        
        assert!(state.has_recent_failure());
    }

    #[test]
    fn test_has_recent_failure_false() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let runs = vec![
            create_test_workflow_run(1, Some(WorkflowConclusion::Success)),
            create_test_workflow_run(2, Some(WorkflowConclusion::Success)),
            create_test_workflow_run(3, Some(WorkflowConclusion::Cancelled)),
        ];
        
        state.update_runs(runs);
        
        assert!(!state.has_recent_failure());
    }

    #[test]
    fn test_has_recent_failure_empty() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let runs: Vec<WorkflowRun> = vec![];
        state.update_runs(runs);
        
        assert!(!state.has_recent_failure());
    }

    #[test]
    fn test_has_recent_failure_none_conclusion() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let runs = vec![
            create_test_workflow_run(1, None), // In progress run
        ];
        
        state.update_runs(runs);
        
        assert!(!state.has_recent_failure());
    }

    #[test]
    fn test_repository_status_equality() {
        assert_eq!(RepositoryStatus::Healthy, RepositoryStatus::Healthy);
        assert_ne!(RepositoryStatus::Healthy, RepositoryStatus::Error);
        assert_eq!(RepositoryStatus::Warning, RepositoryStatus::Warning);
    }

    #[test]
    fn test_repository_status_debug_format() {
        let debug_str = format!("{:?}", RepositoryStatus::Healthy);
        assert!(debug_str.contains("Healthy"));
        
        let debug_str = format!("{:?}", RepositoryStatus::Error);
        assert!(debug_str.contains("Error"));
    }

    #[test]
    fn test_repository_state_clone() {
        let config = create_test_repo_config();
        let mut state = RepositoryState::new(config);
        
        let runs = vec![create_test_workflow_run(1, Some(WorkflowConclusion::Success))];
        state.update_runs(runs);
        
        let cloned_state = state.clone();
        
        assert_eq!(state.config.owner, cloned_state.config.owner);
        assert_eq!(state.workflow_runs.len(), cloned_state.workflow_runs.len());
        assert_eq!(state.status, cloned_state.status);
    }
}