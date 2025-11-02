// Mock implementations for testing
use nighthub::github::models::{Repository, WorkflowRun, WorkflowStatus, WorkflowConclusion};
use chrono::Utc;

pub fn create_mock_repository(name: &str, owner: &str) -> Repository {
    Repository {
        id: 123,
        name: name.to_string(),
        owner: owner.to_string(),
        full_name: format!("{}/{}", owner, name),
        html_url: format!("https://github.com/{}/{}", owner, name),
        default_branch: Some("main".to_string()),
    }
}

pub fn create_mock_workflow_run(id: u64, name: &str, status: WorkflowStatus) -> WorkflowRun {
    WorkflowRun {
        id,
        name: name.to_string(),
        status,
        conclusion: Some(WorkflowConclusion::Success),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        branch: "main".to_string(),
        commit_sha: "abc123".to_string(),
        actor: "testuser".to_string(),
        html_url: format!("https://github.com/test/repo/run/{}", id),
        logs_url: Some(format!("https://github.com/test/repo/logs/{}", id)),
    }
}