use crate::config::settings::Settings;
use crate::error::AppError;
use crate::github::models::{Repository, WorkflowRun, WorkflowStatus, WorkflowConclusion};
use octocrab::Octocrab;
use secrecy::SecretString;
use serde::Deserialize;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

#[async_trait]
pub trait GitHubApiClient {
    async fn get_workflow_runs(&self, route: &str) -> Result<WorkflowRunsResponse, AppError>;
    async fn get_repository(&self, route: &str) -> Result<ApiRepository, AppError>;
}

#[derive(Clone)]
pub struct GithubClient {
    client: std::sync::Arc<Box<dyn GitHubApiClient + Send + Sync>>,
    settings: Settings,
}

struct OctocrabAdapter {
    inner: Octocrab,
}

#[async_trait]
impl GitHubApiClient for OctocrabAdapter {
    async fn get_workflow_runs(&self, route: &str) -> Result<WorkflowRunsResponse, AppError> {
        self.inner.get(route, None::<&()>).await.map_err(AppError::from)
    }

    async fn get_repository(&self, route: &str) -> Result<ApiRepository, AppError> {
        self.inner.get(route, None::<&()>).await.map_err(AppError::from)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkflowRunsResponse {
    workflow_runs: Vec<ApiWorkflowRun>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiWorkflowRun {
    id: u64,
    name: String,
    status: String,
    conclusion: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    head_branch: Option<String>,
    head_sha: String,
    html_url: String,
    logs_url: Option<String>,
    actor: ApiUser,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiUser {
    login: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiRepository {
    id: u64,
    name: String,
    full_name: String,
    html_url: String,
    default_branch: Option<String>,
    owner: ApiUser,
}

impl GithubClient {
    pub fn new(settings: Settings) -> Result<Self, AppError> {
        let octocrab = Octocrab::builder()
            .personal_token(SecretString::new(settings.github_token().to_string()))
            .build()?;
        let client = std::sync::Arc::new(Box::new(OctocrabAdapter { inner: octocrab }) as Box<dyn GitHubApiClient + Send + Sync>);
        Ok(GithubClient { client, settings })
    }

    #[cfg(test)]
    pub fn new_with_client(settings: Settings, client: Box<dyn GitHubApiClient + Send + Sync>) -> Self {
        GithubClient { 
            client: std::sync::Arc::new(client as Box<dyn GitHubApiClient + Send + Sync>), 
            settings 
        }
    }

    #[cfg(test)]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub async fn fetch_workflow_runs(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<WorkflowRun>, AppError> {
        let route = format!("/repos/{}/{}/actions/runs", owner, repo);
        let response = self.client
            .get_workflow_runs(&route)
            .await?;

        let mut all_runs: Vec<WorkflowRun> = response.workflow_runs
            .into_iter()
            .map(|raw_run| {
                let status = match raw_run.status.as_str() {
                    "queued" => WorkflowStatus::Queued,
                    "in_progress" => WorkflowStatus::InProgress,
                    "completed" => WorkflowStatus::Completed,
                    _ => WorkflowStatus::Queued,
                };

                let conclusion = raw_run.conclusion.as_ref().map(|c| match c.as_str() {
                    "success" => WorkflowConclusion::Success,
                    "failure" => WorkflowConclusion::Failure,
                    "cancelled" => WorkflowConclusion::Cancelled,
                    "skipped" => WorkflowConclusion::Skipped,
                    "timed_out" => WorkflowConclusion::TimedOut,
                    _ => WorkflowConclusion::Skipped,
                });

                WorkflowRun {
                    id: raw_run.id,
                    name: raw_run.name,
                    status,
                    conclusion,
                    created_at: raw_run.created_at,
                    updated_at: raw_run.updated_at,
                    branch: raw_run.head_branch.unwrap_or_default(),
                    commit_sha: raw_run.head_sha,
                    actor: raw_run.actor.login,
                    html_url: raw_run.html_url,
                    logs_url: raw_run.logs_url,
                }
            })
            .collect();

        all_runs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        all_runs.truncate(self.settings.monitoring.workflow_runs_per_repo);

        Ok(all_runs)
    }

    pub async fn fetch_repository_info(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Repository, AppError> {
        let route = format!("/repos/{}/{}", owner, repo);
        let repo_info = self.client
            .get_repository(&route)
            .await?;

        Ok(Repository {
            id: repo_info.id,
            name: repo_info.name.clone(),
            owner: repo_info.owner.login,
            full_name: repo_info.full_name,
            html_url: repo_info.html_url,
            default_branch: repo_info.default_branch,
        })
    }

    pub async fn fetch_repositories(&self) -> Result<Vec<Repository>, AppError> {
        let mut repositories = Vec::new();
        for repo_config in self.settings.repositories() {
            let repository = self.fetch_repository_info(&repo_config.owner, &repo_config.name).await?;
            repositories.push(repository);
        }
        Ok(repositories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::{Settings, RepositoryConfig};
    use crate::error::AppError;
    use mockall::mock;
    use mockall::predicate::*;
    use async_trait::async_trait;
    use chrono::{Utc, TimeZone};

    mock! {
        GitHubApiClient {}

        #[async_trait]
        impl GitHubApiClient for GitHubApiClient {
            async fn get_workflow_runs(&self, route: &str) -> Result<WorkflowRunsResponse, AppError>;
            async fn get_repository(&self, route: &str) -> Result<ApiRepository, AppError>;
        }
    }

    fn create_test_settings() -> Settings {
        let mut settings = Settings::default();
        settings.set_github_token("test_token".to_string());
        settings.set_repositories(vec![
            RepositoryConfig {
                owner: "testowner".to_string(),
                name: "testrepo".to_string(),
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
                refresh_interval_seconds: None,
            },
        ]);
        settings
    }

    fn create_mock_workflow_runs_response() -> WorkflowRunsResponse {
        WorkflowRunsResponse {
            workflow_runs: vec![
                ApiWorkflowRun {
                    id: 1,
                    name: "CI".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("success".to_string()),
                    created_at: Utc.with_ymd_and_hms(2023, 1, 1, 10, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2023, 1, 1, 10, 30, 0).unwrap(),
                    head_branch: Some("main".to_string()),
                    head_sha: "abc123".to_string(),
                    html_url: "https://github.com/testowner/testrepo/actions/runs/1".to_string(),
                    logs_url: Some("https://github.com/testowner/testrepo/actions/runs/1/logs".to_string()),
                    actor: ApiUser {
                        login: "user1".to_string(),
                    },
                },
                ApiWorkflowRun {
                    id: 2,
                    name: "Build".to_string(),
                    status: "in_progress".to_string(),
                    conclusion: None,
                    created_at: Utc.with_ymd_and_hms(2023, 1, 1, 9, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2023, 1, 1, 9, 15, 0).unwrap(),
                    head_branch: Some("develop".to_string()),
                    head_sha: "def456".to_string(),
                    html_url: "https://github.com/testowner/testrepo/actions/runs/2".to_string(),
                    logs_url: Some("https://github.com/testowner/testrepo/actions/runs/2/logs".to_string()),
                    actor: ApiUser {
                        login: "user2".to_string(),
                    },
                },
                ApiWorkflowRun {
                    id: 3,
                    name: "Deploy".to_string(),
                    status: "queued".to_string(),
                    conclusion: None,
                    created_at: Utc.with_ymd_and_hms(2023, 1, 1, 8, 0, 0).unwrap(),
                    updated_at: Utc.with_ymd_and_hms(2023, 1, 1, 8, 0, 0).unwrap(),
                    head_branch: None,
                    head_sha: "ghi789".to_string(),
                    html_url: "https://github.com/testowner/testrepo/actions/runs/3".to_string(),
                    logs_url: None,
                    actor: ApiUser {
                        login: "user3".to_string(),
                    },
                },
            ],
        }
    }

    fn create_mock_repository_response() -> ApiRepository {
        ApiRepository {
            id: 12345,
            name: "testrepo".to_string(),
            full_name: "testowner/testrepo".to_string(),
            html_url: "https://github.com/testowner/testrepo".to_string(),
            default_branch: Some("main".to_string()),
            owner: ApiUser {
                login: "testowner".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_fetch_workflow_runs_success() {
        let mut mock_client = MockGitHubApiClient::new();
        let expected_response = create_mock_workflow_runs_response();
        
        mock_client
            .expect_get_workflow_runs()
            .with(eq("/repos/testowner/testrepo/actions/runs"))
            .times(1)
            .returning(move |_| {
                Ok(expected_response.clone())
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_workflow_runs("testowner", "testrepo").await;
        
        assert!(result.is_ok());
        let runs = result.unwrap();
        assert_eq!(runs.len(), 3);
        
        // Verify sorting by created_at (descending)
        assert!(runs[0].created_at > runs[1].created_at);
        assert!(runs[1].created_at > runs[2].created_at);
        
        // Verify first run (completed with success)
        assert_eq!(runs[0].id, 1);
        assert_eq!(runs[0].name, "CI");
        assert_eq!(runs[0].status, WorkflowStatus::Completed);
        assert_eq!(runs[0].conclusion, Some(WorkflowConclusion::Success));
        assert_eq!(runs[0].branch, "main");
        assert_eq!(runs[0].actor, "user1");
        
        // Verify second run (in_progress)
        assert_eq!(runs[1].id, 2);
        assert_eq!(runs[1].name, "Build");
        assert_eq!(runs[1].status, WorkflowStatus::InProgress);
        assert_eq!(runs[1].conclusion, None);
        assert_eq!(runs[1].branch, "develop");
        
        // Verify third run (queued)
        assert_eq!(runs[2].id, 3);
        assert_eq!(runs[2].name, "Deploy");
        assert_eq!(runs[2].status, WorkflowStatus::Queued);
        assert_eq!(runs[2].conclusion, None);
        assert_eq!(runs[2].branch, ""); // None becomes empty string
    }

    #[tokio::test]
    async fn test_fetch_workflow_runs_with_truncation() {
        let mut mock_client = MockGitHubApiClient::new();
        let mut response = create_mock_workflow_runs_response();
        
        // Add more runs to test truncation
        for i in 4..=10 {
            response.workflow_runs.push(ApiWorkflowRun {
                id: i,
                name: format!("Run{}", i),
                status: "completed".to_string(),
                conclusion: Some("success".to_string()),
                created_at: Utc.with_ymd_and_hms(2023, 1, 1, (10 - i) as u32, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2023, 1, 1, (10 - i) as u32, 30, 0).unwrap(),
                head_branch: Some("main".to_string()),
                head_sha: format!("sha{}", i),
                html_url: format!("https://github.com/testowner/testrepo/actions/runs/{}", i),
                logs_url: Some(format!("https://github.com/testowner/testrepo/actions/runs/{}/logs", i)),
                actor: ApiUser {
                    login: format!("user{}", i),
                },
            });
        }
        
        mock_client
            .expect_get_workflow_runs()
            .with(eq("/repos/testowner/testrepo/actions/runs"))
            .times(1)
            .returning(move |_| {
                Ok(response.clone())
            });

        let mut settings = create_test_settings();
        settings.monitoring.workflow_runs_per_repo = 5; // Limit to 5 runs
        
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_workflow_runs("testowner", "testrepo").await;
        
        assert!(result.is_ok());
        let runs = result.unwrap();
        assert_eq!(runs.len(), 5); // Should be truncated to 5
    }

    #[tokio::test]
    async fn test_fetch_workflow_runs_empty_response() {
        let mut mock_client = MockGitHubApiClient::new();
        let empty_response = WorkflowRunsResponse {
            workflow_runs: vec![],
        };
        
        mock_client
            .expect_get_workflow_runs()
            .with(eq("/repos/testowner/testrepo/actions/runs"))
            .times(1)
            .returning(move |_| {
                Ok(empty_response.clone())
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_workflow_runs("testowner", "testrepo").await;
        
        assert!(result.is_ok());
        let runs = result.unwrap();
        assert_eq!(runs.len(), 0);
    }

    #[tokio::test]
    async fn test_fetch_workflow_runs_network_error() {
        let mut mock_client = MockGitHubApiClient::new();
        
        mock_client
            .expect_get_workflow_runs()
            .with(eq("/repos/testowner/testrepo/actions/runs"))
            .times(1)
            .returning(|_| {
                Err(AppError::GithubError("Network error".to_string()))
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_workflow_runs("testowner", "testrepo").await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::GithubError(_) => {}, // Expected
            _ => panic!("Expected GitHub error"),
        }
    }

    #[tokio::test]
    async fn test_fetch_workflow_runs_authentication_error() {
        let mut mock_client = MockGitHubApiClient::new();
        
        mock_client
            .expect_get_workflow_runs()
            .with(eq("/repos/testowner/testrepo/actions/runs"))
            .times(1)
            .returning(|_| {
                Err(AppError::GithubError("Authentication failed".to_string()))
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_workflow_runs("testowner", "testrepo").await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_workflow_runs_malformed_response() {
        let mut mock_client = MockGitHubApiClient::new();
        
        mock_client
            .expect_get_workflow_runs()
            .with(eq("/repos/testowner/testrepo/actions/runs"))
            .times(1)
            .returning(|_| {
                Err(AppError::ParseError("Malformed JSON".to_string()))
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_workflow_runs("testowner", "testrepo").await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_workflow_runs_all_statuses_and_conclusions() {
        let mut mock_client = MockGitHubApiClient::new();
        
        let base_time = Utc.with_ymd_and_hms(2023, 1, 1, 10, 0, 0).unwrap();
        let response = WorkflowRunsResponse {
            workflow_runs: vec![
                // All possible statuses
                ApiWorkflowRun {
                    id: 1,
                    name: "Queued".to_string(),
                    status: "queued".to_string(),
                    conclusion: None,
                    created_at: base_time,
                    updated_at: base_time,
                    head_branch: Some("main".to_string()),
                    head_sha: "abc123".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/1".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
                ApiWorkflowRun {
                    id: 2,
                    name: "InProgress".to_string(),
                    status: "in_progress".to_string(),
                    conclusion: None,
                    created_at: base_time - chrono::Duration::minutes(10),
                    updated_at: base_time - chrono::Duration::minutes(10),
                    head_branch: Some("main".to_string()),
                    head_sha: "def456".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/2".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
                ApiWorkflowRun {
                    id: 3,
                    name: "Completed".to_string(),
                    status: "completed".to_string(),
                    conclusion: None,
                    created_at: base_time - chrono::Duration::minutes(20),
                    updated_at: base_time - chrono::Duration::minutes(20),
                    head_branch: Some("main".to_string()),
                    head_sha: "ghi789".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/3".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
                ApiWorkflowRun {
                    id: 4,
                    name: "UnknownStatus".to_string(),
                    status: "unknown".to_string(),
                    conclusion: None,
                    created_at: base_time - chrono::Duration::minutes(30),
                    updated_at: base_time - chrono::Duration::minutes(30),
                    head_branch: Some("main".to_string()),
                    head_sha: "jkl012".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/4".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
                // All possible conclusions
                ApiWorkflowRun {
                    id: 5,
                    name: "Success".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("success".to_string()),
                    created_at: base_time - chrono::Duration::minutes(40),
                    updated_at: base_time - chrono::Duration::minutes(40),
                    head_branch: Some("main".to_string()),
                    head_sha: "mno345".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/5".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
                ApiWorkflowRun {
                    id: 6,
                    name: "Failure".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("failure".to_string()),
                    created_at: base_time - chrono::Duration::minutes(50),
                    updated_at: base_time - chrono::Duration::minutes(50),
                    head_branch: Some("main".to_string()),
                    head_sha: "pqr678".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/6".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
                ApiWorkflowRun {
                    id: 7,
                    name: "Cancelled".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("cancelled".to_string()),
                    created_at: base_time - chrono::Duration::minutes(60),
                    updated_at: base_time - chrono::Duration::minutes(60),
                    head_branch: Some("main".to_string()),
                    head_sha: "stu901".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/7".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
                ApiWorkflowRun {
                    id: 8,
                    name: "Skipped".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("skipped".to_string()),
                    created_at: base_time - chrono::Duration::minutes(70),
                    updated_at: base_time - chrono::Duration::minutes(70),
                    head_branch: Some("main".to_string()),
                    head_sha: "vwx234".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/8".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
                ApiWorkflowRun {
                    id: 9,
                    name: "TimedOut".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("timed_out".to_string()),
                    created_at: base_time - chrono::Duration::minutes(80),
                    updated_at: base_time - chrono::Duration::minutes(80),
                    head_branch: Some("main".to_string()),
                    head_sha: "yza567".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/9".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
                ApiWorkflowRun {
                    id: 10,
                    name: "UnknownConclusion".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("unknown".to_string()),
                    created_at: base_time - chrono::Duration::minutes(90),
                    updated_at: base_time - chrono::Duration::minutes(90),
                    head_branch: Some("main".to_string()),
                    head_sha: "bcd890".to_string(),
                    html_url: "https://github.com/test/test/actions/runs/10".to_string(),
                    logs_url: None,
                    actor: ApiUser { login: "user".to_string() },
                },
            ],
        };
        
        mock_client
            .expect_get_workflow_runs()
            .with(eq("/repos/testowner/testrepo/actions/runs"))
            .times(1)
            .returning(move |_| {
                Ok(response.clone())
            });

        let mut settings = create_test_settings();
        settings.monitoring.workflow_runs_per_repo = 20; // Ensure no truncation
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_workflow_runs("testowner", "testrepo").await;
        
        assert!(result.is_ok());
        let runs = result.unwrap();
        assert_eq!(runs.len(), 10);
        
        // Verify status mapping
        assert_eq!(runs[0].status, WorkflowStatus::Queued);
        assert_eq!(runs[1].status, WorkflowStatus::InProgress);
        assert_eq!(runs[2].status, WorkflowStatus::Completed);
        assert_eq!(runs[3].status, WorkflowStatus::Queued); // Unknown defaults to Queued
        
        // Verify conclusion mapping
        assert_eq!(runs[4].conclusion, Some(WorkflowConclusion::Success));
        assert_eq!(runs[5].conclusion, Some(WorkflowConclusion::Failure));
        assert_eq!(runs[6].conclusion, Some(WorkflowConclusion::Cancelled));
        assert_eq!(runs[7].conclusion, Some(WorkflowConclusion::Skipped));
        assert_eq!(runs[8].conclusion, Some(WorkflowConclusion::TimedOut));
        assert_eq!(runs[9].conclusion, Some(WorkflowConclusion::Skipped)); // Unknown defaults to Skipped
    }

    #[tokio::test]
    async fn test_fetch_repository_info_success() {
        let mut mock_client = MockGitHubApiClient::new();
        let expected_response = create_mock_repository_response();
        
        mock_client
            .expect_get_repository()
            .with(eq("/repos/testowner/testrepo"))
            .times(1)
            .returning(move |_| {
                Ok(expected_response.clone())
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_repository_info("testowner", "testrepo").await;
        
        assert!(result.is_ok());
        let repo = result.unwrap();
        assert_eq!(repo.id, 12345);
        assert_eq!(repo.name, "testrepo");
        assert_eq!(repo.owner, "testowner");
        assert_eq!(repo.full_name, "testowner/testrepo");
        assert_eq!(repo.html_url, "https://github.com/testowner/testrepo");
        assert_eq!(repo.default_branch, Some("main".to_string()));
    }

    #[tokio::test]
    async fn test_fetch_repository_info_no_default_branch() {
        let mut mock_client = MockGitHubApiClient::new();
        let mut response = create_mock_repository_response();
        response.default_branch = None;
        
        mock_client
            .expect_get_repository()
            .with(eq("/repos/testowner/testrepo"))
            .times(1)
            .returning(move |_| {
                Ok(response.clone())
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_repository_info("testowner", "testrepo").await;
        
        assert!(result.is_ok());
        let repo = result.unwrap();
        assert_eq!(repo.default_branch, None);
    }

    #[tokio::test]
    async fn test_fetch_repository_info_not_found() {
        let mut mock_client = MockGitHubApiClient::new();
        
        mock_client
            .expect_get_repository()
            .with(eq("/repos/testowner/nonexistent"))
            .times(1)
            .returning(|_| {
                Err(AppError::GithubError("Repository not found".to_string()))
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_repository_info("testowner", "nonexistent").await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_repositories_success() {
        let mut mock_client = MockGitHubApiClient::new();
        let repo1_response = create_mock_repository_response();
        let mut repo2_response = create_mock_repository_response();
        repo2_response.id = 67890;
        repo2_response.name = "repo2".to_string();
        repo2_response.full_name = "owner2/repo2".to_string();
        repo2_response.html_url = "https://github.com/owner2/repo2".to_string();
        repo2_response.owner.login = "owner2".to_string();
        
        mock_client
            .expect_get_repository()
            .with(eq("/repos/testowner/testrepo"))
            .times(1)
            .returning(move |_| {
                Ok(repo1_response.clone())
            });
            
        mock_client
            .expect_get_repository()
            .with(eq("/repos/owner2/repo2"))
            .times(1)
            .returning(move |_| {
                Ok(repo2_response.clone())
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_repositories().await;
        
        assert!(result.is_ok());
        let repos = result.unwrap();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].name, "testrepo");
        assert_eq!(repos[1].name, "repo2");
    }

    #[tokio::test]
    async fn test_fetch_repositories_partial_failure() {
        let mut mock_client = MockGitHubApiClient::new();
        let repo1_response = create_mock_repository_response();
        
        mock_client
            .expect_get_repository()
            .with(eq("/repos/testowner/testrepo"))
            .times(1)
            .returning(move |_| {
                Ok(repo1_response.clone())
            });
            
        mock_client
            .expect_get_repository()
            .with(eq("/repos/owner2/repo2"))
            .times(1)
            .returning(|_| {
                Err(AppError::GithubError("Repository not found".to_string()))
            });

        let settings = create_test_settings();
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_repositories().await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_repositories_empty_config() {
        let mock_client = MockGitHubApiClient::new();
        
        let settings = Settings::default(); // No repositories configured
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );

        let result = github_client.fetch_repositories().await;
        
        assert!(result.is_ok());
        let repos = result.unwrap();
        assert_eq!(repos.len(), 0);
    }

    #[test]
    fn test_github_client_new_with_client() {
        let settings = create_test_settings();
        let mock_client = MockGitHubApiClient::new();
        
        let github_client = GithubClient::new_with_client(
            settings,
            Box::new(mock_client)
        );
        
        // Just test that client can be created
        assert_eq!(github_client.settings().repositories().len(), 2);
    }

    #[test]
    fn test_workflow_runs_response_deserialization() {
        let json_data = serde_json::json!({
            "workflow_runs": [
                {
                    "id": 1,
                    "name": "CI",
                    "status": "completed",
                    "conclusion": "success",
                    "created_at": "2023-01-01T10:00:00Z",
                    "updated_at": "2023-01-01T10:30:00Z",
                    "head_branch": "main",
                    "head_sha": "abc123",
                    "html_url": "https://github.com/test/repo/actions/runs/1",
                    "logs_url": "https://github.com/test/repo/actions/runs/1/logs",
                    "actor": {
                        "login": "user1"
                    }
                }
            ]
        });

        let response: Result<WorkflowRunsResponse, _> = serde_json::from_value(json_data);
        assert!(response.is_ok());
        let runs_response = response.unwrap();
        assert_eq!(runs_response.workflow_runs.len(), 1);
        assert_eq!(runs_response.workflow_runs[0].id, 1);
        assert_eq!(runs_response.workflow_runs[0].name, "CI");
    }

    #[test]
    fn test_api_repository_deserialization() {
        let json_data = serde_json::json!({
            "id": 12345,
            "name": "testrepo",
            "full_name": "testowner/testrepo",
            "html_url": "https://github.com/testowner/testrepo",
            "default_branch": "main",
            "owner": {
                "login": "testowner"
            }
        });

        let response: Result<ApiRepository, _> = serde_json::from_value(json_data);
        assert!(response.is_ok());
        let repo = response.unwrap();
        assert_eq!(repo.id, 12345);
        assert_eq!(repo.name, "testrepo");
        assert_eq!(repo.owner.login, "testowner");
    }

    #[test]
    fn test_api_workflow_run_deserialization_with_optional_fields() {
        let json_data = serde_json::json!({
            "id": 1,
            "name": "CI",
            "status": "completed",
            "conclusion": null,
            "created_at": "2023-01-01T10:00:00Z",
            "updated_at": "2023-01-01T10:30:00Z",
            "head_branch": null,
            "head_sha": "abc123",
            "html_url": "https://github.com/test/repo/actions/runs/1",
            "logs_url": null,
            "actor": {
                "login": "user1"
            }
        });

        let response: Result<ApiWorkflowRun, _> = serde_json::from_value(json_data);
        assert!(response.is_ok());
        let run = response.unwrap();
        assert_eq!(run.id, 1);
        assert_eq!(run.conclusion, None);
        assert_eq!(run.head_branch, None);
        assert_eq!(run.logs_url, None);
    }

    #[test]
    fn test_edge_case_empty_strings() {
        let json_data = serde_json::json!({
            "workflow_runs": [
                {
                    "id": 1,
                    "name": "",
                    "status": "",
                    "conclusion": "",
                    "created_at": "2023-01-01T10:00:00Z",
                    "updated_at": "2023-01-01T10:30:00Z",
                    "head_branch": "",
                    "head_sha": "",
                    "html_url": "",
                    "logs_url": "",
                    "actor": {
                        "login": ""
                    }
                }
            ]
        });

        let response: Result<WorkflowRunsResponse, _> = serde_json::from_value(json_data);
        assert!(response.is_ok());
        let runs_response = response.unwrap();
        assert_eq!(runs_response.workflow_runs.len(), 1);
        
        let run = &runs_response.workflow_runs[0];
        assert_eq!(run.name, "");
        assert_eq!(run.status, ""); // Will be mapped to Queued in the client
        assert_eq!(run.conclusion, Some("".to_string()));
        assert_eq!(run.head_branch, Some("".to_string()));
        assert_eq!(run.head_sha, "");
        assert_eq!(run.html_url, "");
        assert_eq!(run.logs_url, Some("".to_string()));
        assert_eq!(run.actor.login, "");
    }
}
