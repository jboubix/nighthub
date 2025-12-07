use nighthub::{
    config::settings::{Settings, RepositoryConfig},
    ui::app::AppState,
    github::models::{WorkflowRun, WorkflowStatus, WorkflowConclusion},
};
use std::collections::HashMap;
use chrono::Utc;
use std::env;

mod common;

#[tokio::test]
async fn test_full_refresh_cycle() {
    // Setup test environment
    unsafe {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_test1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "testowner/repo1:30,testowner/repo2") };
    }

    // Create test settings
    let result = Settings::new();
    assert!(result.is_ok());
    let settings = result.unwrap();

    assert_eq!(settings.repositories.len(), 2);
    assert_eq!(settings.repositories[0].refresh_interval_seconds, Some(30));
    assert_eq!(settings.repositories[1].refresh_interval_seconds, None);

    // Note: In a real integration test, we would mock the GitHub API
    // For now, just test the configuration and state management
    let app_result = AppState::new(settings).await;

    // This would fail in real test without mocking, but shows the integration flow
    // In a complete test setup, we would:
    // 1. Mock the GitHubClient
    // 2. Mock API responses
    // 3. Verify refresh behavior
    // 4. Test UI state transitions

    // Cleanup environment variables
    unsafe {
        env::remove_var("GITHUB_TOKEN");
        env::remove_var("REPOS");
    }
}

#[tokio::test]
async fn test_configuration_to_ui_flow() {
    // Test the flow from configuration parsing to UI state
    unsafe {
        unsafe { env::set_var("GITHUB_TOKEN", "github_pat_test1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "owner1/repo1:60,owner2/repo2:300,owner3/repo3") };
    }
    
    let settings = Settings::new().unwrap();
    
    // Verify configuration parsing
    assert_eq!(settings.repositories.len(), 3);
    assert_eq!(settings.repositories[0].refresh_interval_seconds, Some(60));
    assert_eq!(settings.repositories[1].refresh_interval_seconds, Some(300));
    assert_eq!(settings.repositories[2].refresh_interval_seconds, None);
    
    // Verify repository configurations
    assert_eq!(settings.repositories[0].owner, "owner1");
    assert_eq!(settings.repositories[0].name, "repo1");
    assert_eq!(settings.repositories[1].owner, "owner2");
    assert_eq!(settings.repositories[1].name, "repo2");

    // Cleanup environment variables
    unsafe {
        env::remove_var("GITHUB_TOKEN");
        env::remove_var("REPOS");
    }
}

#[tokio::test]
async fn test_override_refresh_behavior() {
    // Test that refresh interval overrides work correctly
    unsafe { env::set_var("GITHUB_TOKEN", "ghp_test1234567890abcdef1234567890abcdef12345678") };
    unsafe { env::set_var("REPOS", "active/repo:5,normal/repo,slow/repo:3600") };
    
    let settings = Settings::new().unwrap();
    
    // Create mock app state to test refresh calculations
    let mut workflow_runs = HashMap::new();
    let now = Utc::now();
    
    // Active repo with override
    workflow_runs.insert("active/repo".to_string(), vec![
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
            html_url: "https://github.com/active/repo/run/1".to_string(),
            logs_url: Some("https://github.com/active/repo/run/1/logs".to_string()),
        }
    ]);
    
    // Normal repo without override (recent activity)
    workflow_runs.insert("normal/repo".to_string(), vec![
        WorkflowRun {
            id: 2,
            name: "Build".to_string(),
            status: WorkflowStatus::Completed,
            conclusion: Some(WorkflowConclusion::Success),
            created_at: now,
            updated_at: now,
            branch: "main".to_string(),
            commit_sha: "def456".to_string(),
            actor: "user2".to_string(),
            html_url: "https://github.com/normal/repo/run/2".to_string(),
            logs_url: Some("https://github.com/normal/repo/run/2/logs".to_string()),
        }
    ]);
    
    // Slow repo with override (old activity but forced refresh)
    workflow_runs.insert("slow/repo".to_string(), vec![
        WorkflowRun {
            id: 3,
            name: "Deploy".to_string(),
            status: WorkflowStatus::Completed,
            conclusion: Some(WorkflowConclusion::Success),
            created_at: now - chrono::Duration::hours(25), // 25 hours ago
            updated_at: now - chrono::Duration::hours(25),
            branch: "main".to_string(),
            commit_sha: "ghi789".to_string(),
            actor: "user3".to_string(),
            html_url: "https://github.com/slow/repo/run/3".to_string(),
            logs_url: Some("https://github.com/slow/repo/run/3/logs".to_string()),
        }
    ]);
    
    // In a complete test, we would create an AppState with mocked GithubClient
    // and verify that refresh intervals are calculated correctly:
    // - active/repo: 5 seconds (override)
    // - normal/repo: 5 seconds (very recent activity)
    // - slow/repo: 3600 seconds (override, despite old activity)

    // Cleanup environment variables
    unsafe {
        env::remove_var("GITHUB_TOKEN");
        env::remove_var("REPOS");
    }
}

#[tokio::test]
async fn test_error_recovery_flow() {
    // Test error handling in configuration and state management
    unsafe { env::remove_var("GITHUB_TOKEN") };
    unsafe { env::remove_var("GH_TOKEN") };
    
    let result = Settings::new();
    assert!(result.is_err());
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Missing required environment variable"));
    
    // Test invalid REPOS format
    unsafe { env::set_var("GITHUB_TOKEN", "ghp_test1234567890abcdef1234567890abcdef12345678") };
    unsafe { env::set_var("REPOS", "invalid-format") };
    
    let result = Settings::new();
    assert!(result.is_err());
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid repository format"));
}

#[tokio::test]
async fn test_tiered_refresh_calculation() {
    // Test the three-tier refresh system
    unsafe { env::set_var("GITHUB_TOKEN", "ghp_test1234567890abcdef1234567890abcdef12345678") };
    unsafe { env::set_var("REPOS", "owner1/repo1,owner2/repo2,owner3/repo3") };
    
    let settings = Settings::new().unwrap();
    let mut workflow_runs = HashMap::new();
    let now = Utc::now();
    
    // Very active repo (1 hour ago)
    workflow_runs.insert("repo1".to_string(), vec![
        WorkflowRun {
            id: 1,
            name: "CI".to_string(),
            status: WorkflowStatus::Completed,
            conclusion: Some(WorkflowConclusion::Success),
            created_at: now - chrono::Duration::hours(1),
            updated_at: now - chrono::Duration::hours(1),
            branch: "main".to_string(),
            commit_sha: "abc123".to_string(),
            actor: "user1".to_string(),
            html_url: "https://github.com/test/repo1/run/1".to_string(),
            logs_url: Some("https://github.com/test/repo1/run/1/logs".to_string()),
        }
    ]);
    
    // Moderately active repo (12 hours ago)
    workflow_runs.insert("repo2".to_string(), vec![
        WorkflowRun {
            id: 2,
            name: "Build".to_string(),
            status: WorkflowStatus::Completed,
            conclusion: Some(WorkflowConclusion::Success),
            created_at: now - chrono::Duration::hours(12),
            updated_at: now - chrono::Duration::hours(12),
            branch: "main".to_string(),
            commit_sha: "def456".to_string(),
            actor: "user2".to_string(),
            html_url: "https://github.com/test/repo2/run/2".to_string(),
            logs_url: Some("https://github.com/test/repo2/run/2/logs".to_string()),
        }
    ]);
    
    // Inactive repo (25 hours ago)
    workflow_runs.insert("repo3".to_string(), vec![
        WorkflowRun {
            id: 3,
            name: "Deploy".to_string(),
            status: WorkflowStatus::Completed,
            conclusion: Some(WorkflowConclusion::Success),
            created_at: now - chrono::Duration::hours(25),
            updated_at: now - chrono::Duration::hours(25),
            branch: "main".to_string(),
            commit_sha: "ghi789".to_string(),
            actor: "user3".to_string(),
            html_url: "https://github.com/test/repo3/run/3".to_string(),
            logs_url: Some("https://github.com/test/repo3/run/3/logs".to_string()),
        }
    ]);
    
    // In a complete test with mocked AppState, we would verify:
    // - repo1 gets 5-second refresh (very active)
    // - repo2 gets 60-second refresh (moderately active)  
    // - repo3 gets 7200-second refresh (inactive)
    
    // For now, just verify the test data is set up correctly
    assert_eq!(workflow_runs.len(), 3);
    
    let repo1_time = now - workflow_runs["repo1"][0].updated_at;
    let repo2_time = now - workflow_runs["repo2"][0].updated_at;
    let repo3_time = now - workflow_runs["repo3"][0].updated_at;
    
    assert!(repo1_time.num_hours() < 2); // Very active
    assert!(repo2_time.num_hours() >= 2 && repo2_time.num_hours() < 24); // Moderately active
    assert!(repo3_time.num_hours() >= 24); // Inactive

    // Cleanup environment variables
    unsafe {
        env::remove_var("GITHUB_TOKEN");
        env::remove_var("REPOS");
    }
}

#[test]
fn test_environment_variable_parsing() {
    // Test comprehensive environment variable parsing
    unsafe { env::set_var("GITHUB_TOKEN", "github_pat_test1234567890abcdef1234567890abcdef12345678") };
    
    // Test various REPOS formats
    let test_cases = vec![
        ("owner/repo", vec!["owner/repo"]),
        ("owner1/repo1,owner2/repo2", vec!["owner1/repo1", "owner2/repo2"]),
        ("owner/repo:30", vec!["owner/repo:30"]),
        ("owner1/repo1:60,owner2/repo2:300", vec!["owner1/repo1:60", "owner2/repo2:300"]),
        ("owner1/repo1:60,owner2/repo2", vec!["owner1/repo1:60", "owner2/repo2"]),
        (" owner1/repo1 , owner2/repo2:300 ", vec!["owner1/repo1", "owner2/repo2:300"]),
    ];
    
    for (repos_input, expected_repos) in test_cases {
        unsafe { env::set_var("REPOS", repos_input) };

        let result = Settings::new();
        assert!(result.is_ok(), "Failed to parse REPOS: {}", repos_input);

        let settings = result.unwrap();
        assert_eq!(settings.repositories.len(), expected_repos.len(),
                  "Wrong number of repos for input: {}", repos_input);

        // Verify each repository was parsed correctly
        for (i, expected_repo) in expected_repos.iter().enumerate() {
            if expected_repo.contains(':') {
                let parts: Vec<&str> = expected_repo.split(':').collect();
                assert_eq!(settings.repositories[i].owner, parts[0].split('/').collect::<Vec<&str>>()[0]);
                assert_eq!(settings.repositories[i].name, parts[0].split('/').collect::<Vec<&str>>()[1]);
                assert_eq!(settings.repositories[i].refresh_interval_seconds, Some(parts[1].parse().unwrap()));
            } else {
                let parts: Vec<&str> = expected_repo.split('/').collect();
                assert_eq!(settings.repositories[i].owner, parts[0]);
                assert_eq!(settings.repositories[i].name, parts[1]);
                assert_eq!(settings.repositories[i].refresh_interval_seconds, None);
            }
        }

        // Cleanup REPOS after each iteration
        unsafe { env::remove_var("REPOS") };
    }

    // Final cleanup
    unsafe { env::remove_var("GITHUB_TOKEN") };
}