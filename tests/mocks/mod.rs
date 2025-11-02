use mockall::mock;
use crate::github::models::{Repository, WorkflowRun};
use crate::error::AppError;

mock! {
    pub GithubClient {}

    impl Clone for GithubClient {
        fn clone(&self) -> Self;
    }

    impl GithubClient {
        pub fn new(settings: crate::config::settings::Settings) -> Result<Self, AppError>;
        pub async fn fetch_repositories(&self) -> Result<Vec<Repository>, AppError>;
        pub async fn fetch_workflow_runs(&self, owner: &str, repo: &str) -> Result<Vec<WorkflowRun>, AppError>;
    }
}