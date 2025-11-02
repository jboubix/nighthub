use chrono::{DateTime, Utc};
use nighthub::config::settings::RepositoryConfig;
use nighthub::github::models::{Repository, WorkflowRun, WorkflowStatus, WorkflowConclusion};

pub fn create_test_repository_config(owner: &str, name: &str) -> RepositoryConfig {
    RepositoryConfig {
        owner: owner.to_string(),
        name: name.to_string(),
        branch: None,
        workflows: None,
        enabled: true,
        refresh_interval_seconds: None,
    }
}

pub fn create_test_repository_config_with_override(owner: &str, name: &str, override_secs: u64) -> RepositoryConfig {
    RepositoryConfig {
        owner: owner.to_string(),
        name: name.to_string(),
        branch: None,
        workflows: None,
        enabled: true,
        refresh_interval_seconds: Some(override_secs),
    }
}

pub fn create_test_repository(id: u64, owner: &str, name: &str) -> Repository {
    Repository {
        id,
        name: name.to_string(),
        owner: owner.to_string(),
        full_name: format!("{}/{}", owner, name),
        html_url: format!("https://github.com/{}/{}", owner, name),
        default_branch: Some("main".to_string()),
    }
}

pub fn create_test_workflow_run(
    id: u64,
    name: &str,
    status: WorkflowStatus,
    conclusion: Option<WorkflowConclusion>,
    updated_at: DateTime<Utc>,
) -> WorkflowRun {
    WorkflowRun {
        id,
        name: name.to_string(),
        status,
        conclusion,
        created_at: updated_at,
        updated_at,
        branch: "main".to_string(),
        commit_sha: "abc123".to_string(),
        actor: "testuser".to_string(),
        html_url: format!("https://github.com/test/repo/run/{}", id),
        logs_url: Some(format!("https://github.com/test/repo/run/{}/logs", id)),
    }
}

pub fn create_test_workflow_run_recent(id: u64) -> WorkflowRun {
    create_test_workflow_run(
        id,
        "CI",
        WorkflowStatus::Completed,
        Some(WorkflowConclusion::Success),
        Utc::now(),
    )
}

pub fn create_test_workflow_run_hours_ago(id: u64, hours_ago: i64) -> WorkflowRun {
    create_test_workflow_run(
        id,
        "CI",
        WorkflowStatus::Completed,
        Some(WorkflowConclusion::Success),
        Utc::now() - chrono::Duration::hours(hours_ago),
    )
}