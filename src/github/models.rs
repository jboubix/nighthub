use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Queued,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WorkflowConclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub owner: String,
    pub full_name: String,
    pub html_url: String,
    pub default_branch: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub status: WorkflowStatus,
    pub conclusion: Option<WorkflowConclusion>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub branch: String,
    pub commit_sha: String,
    pub actor: String,
    pub html_url: String,
    pub logs_url: Option<String>,
}

impl From<String> for WorkflowStatus {
    fn from(status: String) -> Self {
        match status.as_str() {
            "queued" => WorkflowStatus::Queued,
            "in_progress" => WorkflowStatus::InProgress,
            "completed" => WorkflowStatus::Completed,
            _ => WorkflowStatus::Queued,
        }
    }
}

impl From<Option<String>> for WorkflowConclusion {
    fn from(conclusion: Option<String>) -> Self {
        match conclusion.as_deref() {
            Some("success") => WorkflowConclusion::Success,
            Some("failure") => WorkflowConclusion::Failure,
            Some("cancelled") => WorkflowConclusion::Cancelled,
            Some("skipped") => WorkflowConclusion::Skipped,
            Some("timed_out") => WorkflowConclusion::TimedOut,
            _ => WorkflowConclusion::Success,  // Default for completed without conclusion
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_workflow_status_from_string() {
        assert_eq!(WorkflowStatus::from("queued".to_string()), WorkflowStatus::Queued);
        assert_eq!(WorkflowStatus::from("in_progress".to_string()), WorkflowStatus::InProgress);
        assert_eq!(WorkflowStatus::from("completed".to_string()), WorkflowStatus::Completed);
        assert_eq!(WorkflowStatus::from("unknown".to_string()), WorkflowStatus::Queued); // Default case
    }

    #[test]
    fn test_workflow_status_from_string_case_sensitivity() {
        assert_eq!(WorkflowStatus::from("QUEUED".to_string()), WorkflowStatus::Queued);
        assert_eq!(WorkflowStatus::from("IN_PROGRESS".to_string()), WorkflowStatus::Queued); // Not matched, defaults to Queued
    }

    #[test]
    fn test_workflow_conclusion_from_option() {
        assert_eq!(
            WorkflowConclusion::from(Some("success".to_string())),
            WorkflowConclusion::Success
        );
        assert_eq!(
            WorkflowConclusion::from(Some("failure".to_string())),
            WorkflowConclusion::Failure
        );
        assert_eq!(
            WorkflowConclusion::from(Some("cancelled".to_string())),
            WorkflowConclusion::Cancelled
        );
        assert_eq!(
            WorkflowConclusion::from(Some("skipped".to_string())),
            WorkflowConclusion::Skipped
        );
        assert_eq!(
            WorkflowConclusion::from(Some("timed_out".to_string())),
            WorkflowConclusion::TimedOut
        );
    }

    #[test]
    fn test_workflow_conclusion_from_none() {
        assert_eq!(
            WorkflowConclusion::from(None),
            WorkflowConclusion::Success // Default for completed without conclusion
        );
    }

    #[test]
    fn test_workflow_conclusion_from_unknown() {
        assert_eq!(
            WorkflowConclusion::from(Some("unknown".to_string())),
            WorkflowConclusion::Success // Default case
        );
    }

    #[test]
    fn test_workflow_conclusion_from_empty_string() {
        assert_eq!(
            WorkflowConclusion::from(Some("".to_string())),
            WorkflowConclusion::Success // Default case
        );
    }

    #[test]
    fn test_repository_creation() {
        let repo = Repository {
            id: 123,
            name: "test-repo".to_string(),
            owner: "test-owner".to_string(),
            full_name: "test-owner/test-repo".to_string(),
            html_url: "https://github.com/test-owner/test-repo".to_string(),
            default_branch: Some("main".to_string()),
        };

        assert_eq!(repo.id, 123);
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.owner, "test-owner");
        assert_eq!(repo.full_name, "test-owner/test-repo");
        assert_eq!(repo.html_url, "https://github.com/test-owner/test-repo");
        assert_eq!(repo.default_branch, Some("main".to_string()));
    }

    #[test]
    fn test_workflow_run_creation() {
        let now = Utc::now();
        let run = WorkflowRun {
            id: 456,
            name: "CI".to_string(),
            status: WorkflowStatus::Completed,
            conclusion: Some(WorkflowConclusion::Success),
            created_at: now,
            updated_at: now,
            branch: "main".to_string(),
            commit_sha: "abc123def456".to_string(),
            actor: "testuser".to_string(),
            html_url: "https://github.com/test/repo/run/456".to_string(),
            logs_url: Some("https://github.com/test/repo/run/456/logs".to_string()),
        };

        assert_eq!(run.id, 456);
        assert_eq!(run.name, "CI");
        assert_eq!(run.status, WorkflowStatus::Completed);
        assert_eq!(run.conclusion, Some(WorkflowConclusion::Success));
        assert_eq!(run.branch, "main");
        assert_eq!(run.commit_sha, "abc123def456");
        assert_eq!(run.actor, "testuser");
        assert_eq!(run.html_url, "https://github.com/test/repo/run/456");
        assert_eq!(run.logs_url, Some("https://github.com/test/repo/run/456/logs".to_string()));
    }

    #[test]
    fn test_workflow_run_with_none_conclusion() {
        let run = WorkflowRun {
            id: 789,
            name: "Build".to_string(),
            status: WorkflowStatus::InProgress,
            conclusion: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            branch: "develop".to_string(),
            commit_sha: "def456abc123".to_string(),
            actor: "builder".to_string(),
            html_url: "https://github.com/test/repo/run/789".to_string(),
            logs_url: None,
        };

        assert_eq!(run.conclusion, None);
        assert_eq!(run.status, WorkflowStatus::InProgress);
        assert_eq!(run.logs_url, None);
    }

    #[test]
    fn test_workflow_status_equality() {
        assert_eq!(WorkflowStatus::Queued, WorkflowStatus::Queued);
        assert_ne!(WorkflowStatus::Queued, WorkflowStatus::InProgress);
        assert_eq!(WorkflowStatus::Completed, WorkflowStatus::Completed);
    }

    #[test]
    fn test_workflow_conclusion_equality() {
        assert_eq!(WorkflowConclusion::Success, WorkflowConclusion::Success);
        assert_ne!(WorkflowConclusion::Success, WorkflowConclusion::Failure);
        assert_eq!(WorkflowConclusion::Cancelled, WorkflowConclusion::Cancelled);
    }

    #[test]
    fn test_workflow_status_debug_format() {
        let debug_str = format!("{:?}", WorkflowStatus::Queued);
        assert!(debug_str.contains("Queued"));
    }

    #[test]
    fn test_workflow_conclusion_debug_format() {
        let debug_str = format!("{:?}", WorkflowConclusion::Success);
        assert!(debug_str.contains("Success"));
    }
}