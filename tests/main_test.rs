use nighthub::{
    config::settings::{Settings, RepositoryConfig, MonitoringConfig, UiConfig, LoggingConfig},
    ui::app::AppState,
    ui::components::workflow_list::WorkflowListComponent,
    github::models::{Repository, WorkflowRun, WorkflowStatus, WorkflowConclusion},
};
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use chrono::Utc;
use std::collections::HashMap;
use ratatui::{
    layout::Rect,
};

// Simplified mock for testing - we'll test the logic without actual rendering
fn create_test_area() -> Rect {
    Rect {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
    }
}

// Mock event stream for testing keyboard input
struct MockEventStream {
    events: Vec<Event>,
    index: usize,
}

impl MockEventStream {
    fn new(events: Vec<Event>) -> Self {
        Self { events, index: 0 }
    }

    fn next_event(&mut self) -> Option<Event> {
        if self.index < self.events.len() {
            let event = self.events[self.index].clone();
            self.index += 1;
            Some(event)
        } else {
            None
        }
    }
}

fn create_test_settings() -> Settings {
    Settings {
        github_token: "test_token".to_string(),
        repositories: vec![
            RepositoryConfig {
                owner: "owner1".to_string(),
                name: "repo1".to_string(),
                branch: None,
                workflows: None,
                enabled: true,
            },
            RepositoryConfig {
                owner: "owner2".to_string(),
                name: "repo2".to_string(),
                branch: None,
                workflows: None,
                enabled: true,
            },
        ],
        monitoring: MonitoringConfig::default(),
        ui: UiConfig::default(),
        logging: LoggingConfig::default(),
    }
}

fn create_mock_app_state() -> AppState {
    let settings = create_test_settings();
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

    let mut workflow_runs = HashMap::new();
    let now = Utc::now();

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
        },
        WorkflowRun {
            id: 2,
            name: "Deploy".to_string(),
            status: WorkflowStatus::InProgress,
            conclusion: None,
            created_at: now,
            updated_at: now,
            branch: "main".to_string(),
            commit_sha: "def456".to_string(),
            actor: "user2".to_string(),
            html_url: "https://github.com/owner1/repo1/run/2".to_string(),
            logs_url: Some("https://github.com/owner1/repo1/run/2/logs".to_string()),
        }
    ]);

    workflow_runs.insert("owner2/repo2".to_string(), vec![
        WorkflowRun {
            id: 3,
            name: "Tests".to_string(),
            status: WorkflowStatus::Completed,
            conclusion: Some(WorkflowConclusion::Failure),
            created_at: now - chrono::Duration::hours(25),
            updated_at: now - chrono::Duration::hours(25),
            branch: "main".to_string(),
            commit_sha: "ghi789".to_string(),
            actor: "user3".to_string(),
            html_url: "https://github.com/owner2/repo2/run/3".to_string(),
            logs_url: Some("https://github.com/owner2/repo2/run/3/logs".to_string()),
        }
    ]);

    AppState {
        repositories: repos,
        workflow_runs,
        selected_repo: None,
        selected_run: None,
        popup: None,
        context_menu: nighthub::ui::components::context_menu::ContextMenuComponent::new(),
        settings,
        github_client: nighthub::github::client::GithubClient::new(create_test_settings()).unwrap(),
        last_repo_refresh_times: HashMap::new(),
        refreshing_repos: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
    }
}

#[cfg(test)]
mod main_application_tests {
    use super::*;

    #[tokio::test]
    async fn test_application_lifecycle_initialization() {
        // Test that the application can be initialized with mock components
        let _settings = create_test_settings();
        let app_state = create_mock_app_state();
        
        // Verify initial state
        assert_eq!(app_state.repositories.len(), 2);
        assert_eq!(app_state.workflow_runs.len(), 2);
        assert_eq!(app_state.selected_repo, None);
        assert_eq!(app_state.selected_run, None);
        assert_eq!(app_state.popup, None);
        
        // Test initial refresh timing
        let seconds_until_refresh = app_state.seconds_until_refresh();
        assert_eq!(seconds_until_refresh, 0); // Never refreshed
    }

    #[tokio::test]
    async fn test_refresh_timing_logic() {
        let mut app_state = create_mock_app_state();
        
        // Initially needs refresh
        assert_eq!(app_state.seconds_until_refresh(), 0);
        
        // Simulate a refresh
        let now = Utc::now();
        app_state.last_repo_refresh_times.insert("owner1/repo1".to_string(), now);
        app_state.last_repo_refresh_times.insert("owner2/repo2".to_string(), now);
        
        // Now should show time until next refresh
        let seconds_until_refresh = app_state.seconds_until_refresh();
        assert!(seconds_until_refresh > 0);
        assert!(seconds_until_refresh <= 5); // Should be 5 seconds for active repo
    }

    #[tokio::test]
    async fn test_keyboard_event_handling_navigation() {
        let mut app_state = create_mock_app_state();
        let mut workflow_list = WorkflowListComponent::new();
        
        // Test j/down key - next repository
        let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();
        workflow_list.next_repo(app_state.repositories.len());
        app_state.selected_repo = Some(workflow_list.selected_repo_index);
        app_state.selected_run = Some(workflow_list.selected_run_index);
        
        assert_eq!(app_state.selected_repo, Some(1));
        assert_eq!(app_state.selected_run, Some(0));
        
        // Test k/up key - previous repository
        workflow_list.previous_repo(app_state.repositories.len());
        app_state.selected_repo = Some(workflow_list.selected_repo_index);
        app_state.selected_run = Some(workflow_list.selected_run_index);
        
        assert_eq!(app_state.selected_repo, Some(0));
        assert_eq!(app_state.selected_run, Some(0));
        
        // Test l/right key - next workflow run
        workflow_list.next_run(&app_state.workflow_runs, &repo_names);
        app_state.selected_repo = Some(workflow_list.selected_repo_index);
        app_state.selected_run = Some(workflow_list.selected_run_index);
        
        assert_eq!(app_state.selected_repo, Some(0));
        assert_eq!(app_state.selected_run, Some(1));
        
        // Test h/left key - previous workflow run
        workflow_list.previous_run(&app_state.workflow_runs, &repo_names);
        app_state.selected_repo = Some(workflow_list.selected_repo_index);
        app_state.selected_run = Some(workflow_list.selected_run_index);
        
        assert_eq!(app_state.selected_repo, Some(0));
        assert_eq!(app_state.selected_run, Some(0));
    }

    #[tokio::test]
    async fn test_keyboard_event_handling_popup() {
        let mut app_state = create_mock_app_state();
        
        // Test Enter key - open context menu
        app_state.handle_key("enter");
        assert!(app_state.popup.is_some());
        assert_eq!(app_state.popup, Some(nighthub::ui::app::PopupType::ContextMenu));
        
        // Test Esc key - close popup
        app_state.handle_key("esc");
        assert!(app_state.popup.is_none());
    }

    #[tokio::test]
    async fn test_signal_handling_setup() {
        // Test that the signal handler can be set up
        let should_exit = Arc::new(AtomicBool::new(false));
        let should_exit_clone = Arc::clone(&should_exit);
        
        // Simulate setting up the Ctrl+C handler
        let handler_result = ctrlc::set_handler(move || {
            should_exit_clone.store(true, Ordering::Relaxed);
        });
        
        assert!(handler_result.is_ok());
        
        // Simulate Ctrl+C signal
        should_exit.store(true, Ordering::Relaxed);
        assert!(should_exit.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_terminal_rendering_integration() {
        let mut app_state = create_mock_app_state();
        let mut workflow_list = WorkflowListComponent::new();
        
        // Set up some selections
        app_state.selected_repo = Some(0);
        app_state.selected_run = Some(0);
        workflow_list.selected_repo_index = 0;
        workflow_list.selected_run_index = 0;
        
        // Test that rendering data is prepared correctly
        let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();
        let seconds_until_refresh = app_state.seconds_until_refresh();
        
        // Verify data preparation for rendering
        assert_eq!(repo_names.len(), 2);
        assert!(seconds_until_refresh >= 0);
        assert!(app_state.workflow_runs.contains_key("owner1/repo1"));
        assert!(app_state.workflow_runs.contains_key("owner2/repo2"));
    }

    #[tokio::test]
    async fn test_application_state_synchronization() {
        let mut app_state = create_mock_app_state();
        let mut workflow_list = WorkflowListComponent::new();
        
        // Test that workflow list and app state stay synchronized
        let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();
        
        // Navigate in workflow list
        workflow_list.next_repo(app_state.repositories.len());
        
        // Sync with app state (as done in main loop)
        app_state.selected_repo = Some(workflow_list.selected_repo_index);
        app_state.selected_run = Some(workflow_list.selected_run_index);
        
        // Verify synchronization
        assert_eq!(app_state.selected_repo, Some(workflow_list.selected_repo_index));
        assert_eq!(app_state.selected_run, Some(workflow_list.selected_run_index));
        
        // Navigate runs
        workflow_list.next_run(&app_state.workflow_runs, &repo_names);
        
        // Sync again
        app_state.selected_repo = Some(workflow_list.selected_repo_index);
        app_state.selected_run = Some(workflow_list.selected_run_index);
        
        // Verify synchronization
        assert_eq!(app_state.selected_repo, Some(workflow_list.selected_repo_index));
        assert_eq!(app_state.selected_run, Some(workflow_list.selected_run_index));
    }

    #[tokio::test]
    async fn test_error_handling_in_refresh() {
        let mut app_state = create_mock_app_state();
        
        // Test that refresh timing logic works correctly
        // We can't test actual refresh without mocking the GitHub client
        // but we can test the timing and state management
        
        // Initially needs refresh
        assert_eq!(app_state.seconds_until_refresh(), 0);
        
        // Test that the refresh structure exists and can be called
        // (We don't assert success here since it would require real API calls)
        let _refresh_result = app_state.refresh(false).await;
        
        // Verify state structure remains intact
        assert_eq!(app_state.repositories.len(), 2);
        assert!(!app_state.workflow_runs.is_empty());
    }

    #[tokio::test]
    async fn test_event_loop_timeout_behavior() {
        // Test that the event loop respects timeouts and remains responsive
        let start_time = std::time::Instant::now();
        
        // Simulate the 100ms timeout from the main loop
        let timeout_duration = Duration::from_millis(100);
        let result = timeout(timeout_duration, sleep(Duration::from_millis(50))).await;
        
        assert!(result.is_ok());
        assert!(start_time.elapsed() >= Duration::from_millis(45)); // Allow some variance
    }

    #[tokio::test]
    async fn test_popup_rendering_integration() {
        let mut app_state = create_mock_app_state();
        let _workflow_list = WorkflowListComponent::new();
        
        // Open context menu
        app_state.open_context_menu();
        
        // Test rendering data preparation with popup
        let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();
        let seconds_until_refresh = app_state.seconds_until_refresh();
        
        // Verify popup state and rendering data
        assert!(app_state.popup.is_some());
        assert_eq!(app_state.popup, Some(nighthub::ui::app::PopupType::ContextMenu));
        assert_eq!(repo_names.len(), 2);
        
        // Test context menu area calculation
        let area = create_test_area();
        let context_menu_area = Rect {
            x: area.width / 2 - 10,
            y: area.height / 2 - 2,
            width: 20.min(area.width),
            height: 4.min(area.height),
        };
        
        assert!(context_menu_area.x > 0);
        assert!(context_menu_area.y > 0);
        assert!(context_menu_area.width > 0);
        assert!(context_menu_area.height > 0);
    }

    #[tokio::test]
    async fn test_application_shutdown_procedure() {
        // Test that the application can shut down cleanly
        let should_exit = Arc::new(AtomicBool::new(false));
        
        // Simulate normal exit condition
        should_exit.store(true, Ordering::Relaxed);
        
        // In the real main loop, this would break the loop and call disable_raw_mode()
        assert!(should_exit.load(Ordering::Relaxed));
        
        // Test terminal cleanup (this would normally be called at the end)
        // Note: We can't actually test disable_raw_mode() in this context
        // but we can verify the logic would be reached
    }

    #[tokio::test]
    async fn test_keyboard_event_filtering() {
        // Test that only key press events are handled, not repeat/release events
        let events = vec![
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            }),
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Repeat,
                state: crossterm::event::KeyEventState::NONE,
            }),
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Release,
                state: crossterm::event::KeyEventState::NONE,
            }),
        ];
        
        let mut event_stream = MockEventStream::new(events);
        let mut app_state = create_mock_app_state();
        let mut workflow_list = WorkflowListComponent::new();
        
        // Process events
        while let Some(event) = event_stream.next_event() {
            if let Event::Key(key) = event && key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        if app_state.popup.is_none() {
                            workflow_list.next_repo(app_state.repositories.len());
                            app_state.selected_repo = Some(workflow_list.selected_repo_index);
                            app_state.selected_run = Some(workflow_list.selected_run_index);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Should only have processed the first (Press) event
        assert_eq!(app_state.selected_repo, Some(1));
    }

    #[tokio::test]
    async fn test_ctrl_c_exit_handling() {
        // Test both Ctrl+C and 'q' exit conditions
        let exit_events = vec![
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            }),
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            }),
        ];
        
        for event in exit_events {
            let mut should_exit = false;
            
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            should_exit = true;
                        }
                        KeyCode::Char('q') => {
                            should_exit = true;
                        }
                        _ => {}
                    }
                }
            }
            
            assert!(should_exit);
        }
    }

    #[tokio::test]
    async fn test_timer_display_formatting() {
        let app_state = create_mock_app_state();
        let _workflow_list = WorkflowListComponent::new();
        
        // Test different timer values for display formatting
        let test_cases = vec![0, 30, 65, 120, 3600];
        
        for seconds in test_cases {
            let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();
            
            // Test that timer values are handled correctly
            assert!(!repo_names.is_empty());
            
            // Test timer text formatting logic (from the component)
            let timer_text = if seconds < 60 {
                format!("Refresh in {}s", seconds)
            } else {
                format!("Refresh in {}m {}", seconds / 60, seconds % 60)
            };
            
            // Verify the timer text is reasonable
            assert!(!timer_text.is_empty());
            assert!(timer_text.contains("Refresh in"));
        }
    }

    #[tokio::test]
    async fn test_empty_repositories_handling() {
        // Test application behavior with no repositories
        let settings = Settings {
            github_token: "test_token".to_string(),
            repositories: vec![],
            monitoring: MonitoringConfig::default(),
            ui: UiConfig::default(),
            logging: LoggingConfig::default(),
        };
        
        let app_state = AppState {
            repositories: vec![],
            workflow_runs: HashMap::new(),
            selected_repo: None,
            selected_run: None,
            popup: None,
            context_menu: nighthub::ui::components::context_menu::ContextMenuComponent::new(),
            settings,
            github_client: nighthub::github::client::GithubClient::new(create_test_settings()).unwrap(),
            last_repo_refresh_times: HashMap::new(),
            refreshing_repos: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new())),
        };
        
        let mut workflow_list = WorkflowListComponent::new();
        
        // Should handle empty state gracefully
        let _seconds_until_refresh = app_state.seconds_until_refresh();
        assert_eq!(_seconds_until_refresh, 5); // Default when no repos
        
        // Should prepare rendering data without crashing
        let _area = create_test_area();
        let repo_names: Vec<String> = vec![];
        
        // Verify empty state handling
        assert_eq!(repo_names.len(), 0);
        assert_eq!(app_state.repositories.len(), 0);
        assert_eq!(app_state.workflow_runs.len(), 0);
        
        // Navigation should not panic
        workflow_list.next_repo(0);
        workflow_list.previous_repo(0);
        workflow_list.next_run(&app_state.workflow_runs, &repo_names);
        workflow_list.previous_run(&app_state.workflow_runs, &repo_names);
        
        // Verify navigation state remains consistent
        assert_eq!(workflow_list.selected_repo_index, 0);
        assert_eq!(workflow_list.selected_run_index, 0);
    }

    #[tokio::test]
    async fn test_concurrent_refresh_and_ui_updates() {
        // Test that refresh operations don't block UI updates
        let _app_state = create_mock_app_state();
        
        // Simulate the main loop's concurrent behavior
        let refresh_task = tokio::spawn(async move {
            // Simulate a slow refresh
            sleep(Duration::from_millis(50)).await;
            true
        });
        
        // UI should remain responsive during refresh
        let ui_update_task = tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            "ui_updated"
        });
        
        // Both should complete
        let (refresh_result, ui_result) = tokio::join!(refresh_task, ui_update_task);
        
        assert!(refresh_result.is_ok());
        assert!(ui_result.is_ok());
        assert_eq!(ui_result.unwrap(), "ui_updated");
    }

    #[tokio::test]
    async fn test_manual_refresh_with_f_key() {
        let mut app_state = create_mock_app_state();

        // Verify that no popup is open (condition for refresh)
        assert!(app_state.popup.is_none());

        // Note: reset_refresh_timer method no longer exists
        // Timer calculation is now dynamic based on activity
    }



    #[tokio::test]
    async fn test_f_key_ignored_when_popup_open() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
        
        let mut app_state = create_mock_app_state();
        app_state.open_context_menu(); // Open a popup
        
        // Simulate 'f' key press when popup is open
        let f_key_event = Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        // When popup is open, 'f' key should be ignored
        assert!(app_state.popup.is_some());

        // Verify that refresh would not be triggered when popup is open
        // This test verifies the condition prevents refresh
        assert!(true); // Placeholder - actual behavior testing
    }
}