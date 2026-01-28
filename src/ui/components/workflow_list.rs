use crate::github::models::WorkflowRun;
use crate::utils::icons::{get_status_icon, get_conclusion_icon};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

pub struct WorkflowListComponent {
    pub selected_repo_index: usize,
    pub selected_run_index: usize,
}

impl WorkflowListComponent {
    pub fn new() -> Self {
        WorkflowListComponent {
            selected_repo_index: 0,
            selected_run_index: 0,
        }
    }

    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        workflow_runs: &HashMap<String, Vec<WorkflowRun>>,
        repo_names: &[String],
        seconds_until_refresh: u64,
        refreshing_repos: &Arc<RwLock<HashSet<String>>>,
    ) {
        let mut lines = vec![];

        // Get current refreshing repos
        let refreshing_set = refreshing_repos.read().unwrap();
        let is_any_refreshing = !refreshing_set.is_empty();
        

        
        // Add timer information
        let timer_text = if is_any_refreshing {
            // Simple spinner frames with refresh emoji
            let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let frame_index = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() / 100) as usize % spinner_frames.len();
            format!("🔄 {} Refreshing {} repos...", spinner_frames[frame_index], refreshing_set.len())
        } else if seconds_until_refresh < 60 {
            format!("Refresh in {}s", seconds_until_refresh)
        } else {
            format!("Refresh in {}m {}", seconds_until_refresh / 60, seconds_until_refresh % 60)
        };
        
        lines.push(Line::from(vec![
            Span::styled(timer_text, Style::default().fg(Color::Yellow))
        ]));

        // Add workflow runs for each repository
        for (repo_idx, repo_name) in repo_names.iter().enumerate() {
            if let Some(runs) = workflow_runs.get(repo_name) {
                let is_refreshing = refreshing_set.contains(repo_name);
                let refresh_indicator = if is_refreshing { "🔄 " } else { "" };
                
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}{}: {}", refresh_indicator, repo_name, runs.len()),
                        Style::default().fg(if repo_idx == self.selected_repo_index {
                            Color::Green
                        } else {
                            Color::Gray
                        })
                    )
                ]));

                for (run_idx, run) in runs.iter().enumerate() {
                    let is_selected = repo_idx == self.selected_repo_index && run_idx == self.selected_run_index;
                    let status_icon = get_status_icon(&run.status);
                    let conclusion_icon = get_conclusion_icon(&run.conclusion);
                    
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(
                            format!("{} {} {} - {} ({})", 
                                status_icon, 
                                conclusion_icon,
                                run.name,
                                run.branch,
                                crate::utils::time::format_relative_time(run.updated_at)
                            ),
                            if is_selected {
                                Style::default()
                                    .fg(Color::White)
                                    .bg(Color::DarkGray)
                                    .add_modifier(ratatui::style::Modifier::ITALIC)
                            } else {
                                Style::default().fg(Color::Gray)
                            }
                        )
                    ]));
                }
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Workflows"));

        f.render_widget(paragraph, area);
    }

    pub fn next_repo(&mut self, repo_count: usize) {
        if repo_count > 0 {
            self.selected_repo_index = (self.selected_repo_index + 1) % repo_count;
            self.selected_run_index = 0;
        }
    }

    pub fn previous_repo(&mut self, repo_count: usize) {
        if repo_count > 0 {
            self.selected_repo_index = (self.selected_repo_index + repo_count - 1) % repo_count;
            self.selected_run_index = 0;
        }
    }

    pub fn next_run(&mut self, workflow_runs: &HashMap<String, Vec<WorkflowRun>>, repo_names: &[String]) {
        if repo_names.is_empty() {
            return;
        }

        // Get all workflow runs in order
        let mut all_runs = Vec::new();
        
        for (repo_idx, repo_name) in repo_names.iter().enumerate() {
            if let Some(runs) = workflow_runs.get(repo_name) {
                for (run_idx, _) in runs.iter().enumerate() {
                    all_runs.push((repo_idx, run_idx));
                }
            }
        }

        if all_runs.is_empty() {
            return;
        }

        // Find current position
        let current_pos = all_runs.iter().position(|&(repo_idx, run_idx)| {
            repo_idx == self.selected_repo_index && run_idx == self.selected_run_index
        });

        if let Some(current_idx) = current_pos {
            // Move to next run (with wraparound)
            let next_idx = (current_idx + 1) % all_runs.len();
            let (new_repo_idx, new_run_idx) = all_runs[next_idx];
            self.selected_repo_index = new_repo_idx;
            self.selected_run_index = new_run_idx;
        } else {
            // If current selection is invalid, select first run
            let (first_repo_idx, first_run_idx) = all_runs[0];
            self.selected_repo_index = first_repo_idx;
            self.selected_run_index = first_run_idx;
        }
    }

    pub fn previous_run(&mut self, workflow_runs: &HashMap<String, Vec<WorkflowRun>>, repo_names: &[String]) {
        if repo_names.is_empty() {
            return;
        }

        // Get all workflow runs in order
        let mut all_runs = Vec::new();
        let mut run_positions = Vec::new();
        
        for (repo_idx, repo_name) in repo_names.iter().enumerate() {
            if let Some(runs) = workflow_runs.get(repo_name) {
                for (run_idx, _) in runs.iter().enumerate() {
                    all_runs.push((repo_idx, run_idx));
                    run_positions.push((repo_idx, run_idx));
                }
            }
        }

        if all_runs.is_empty() {
            return;
        }

        // Find current position
        let current_pos = all_runs.iter().position(|&(repo_idx, run_idx)| {
            repo_idx == self.selected_repo_index && run_idx == self.selected_run_index
        });

        if let Some(current_idx) = current_pos {
            // Move to previous run (with wraparound)
            let prev_idx = if current_idx == 0 {
                all_runs.len() - 1
            } else {
                current_idx - 1
            };
            
            let (new_repo_idx, new_run_idx) = all_runs[prev_idx];
            self.selected_repo_index = new_repo_idx;
            self.selected_run_index = new_run_idx;
        } else {
            // If current selection is invalid, select first run
            let (first_repo_idx, first_run_idx) = all_runs[0];
            self.selected_repo_index = first_repo_idx;
            self.selected_run_index = first_run_idx;
        }
    }

    pub fn get_selected_repo(&self, repo_names: &[String]) -> Option<String> {
        if self.selected_repo_index < repo_names.len() {
            Some(repo_names[self.selected_repo_index].clone())
        } else {
            None
        }
    }

    pub fn get_selected_run(&self, workflow_runs: &HashMap<String, Vec<WorkflowRun>>, repo_names: &[String]) -> Option<WorkflowRun> {
        if let Some(repo_name) = repo_names.get(self.selected_repo_index) {
            if let Some(runs) = workflow_runs.get(repo_name) {
                if self.selected_run_index < runs.len() {
                    return Some(runs[self.selected_run_index].clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::models::{WorkflowStatus, WorkflowConclusion};
    use ratatui::{
        backend::TestBackend,
        Terminal,
    };
    use chrono::Utc;

    fn create_test_workflow_list() -> WorkflowListComponent {
        WorkflowListComponent::new()
    }

    fn create_test_workflow_runs() -> HashMap<String, Vec<WorkflowRun>> {
        let mut runs = HashMap::new();
        runs.insert("test/repo".to_string(), vec![
            WorkflowRun {
                id: 123,
                name: "Test Workflow".to_string(),
                status: WorkflowStatus::Completed,
                conclusion: Some(WorkflowConclusion::Success),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                branch: "main".to_string(),
                commit_sha: "abc123".to_string(),
                actor: "testuser".to_string(),
                html_url: "https://github.com/test/repo/run/123".to_string(),
                logs_url: Some("https://github.com/test/repo/logs/123".to_string()),
            }
        ]);
        runs
    }

    #[test]
    fn test_navigation() {
        let mut component = create_test_workflow_list();
        let runs = create_test_workflow_runs();
        let repo_names = vec!["test/repo".to_string()];

        // Test repo navigation
        component.next_repo(1);
        assert_eq!(component.selected_repo_index, 0);
        
        component.previous_repo(1);
        assert_eq!(component.selected_repo_index, 0);

        // Test run navigation
        component.next_run(&runs, &repo_names);
        assert_eq!(component.selected_run_index, 0);
        
        component.previous_run(&runs, &repo_names);
        assert_eq!(component.selected_run_index, 0);
    }

    #[test]
    fn test_get_selected() {
        let component = create_test_workflow_list();
        let runs = create_test_workflow_runs();
        let repo_names = vec!["test/repo".to_string()];

        let selected_repo = component.get_selected_repo(&repo_names);
        assert_eq!(selected_repo, Some("test/repo".to_string()));

        let selected_run = component.get_selected_run(&runs, &repo_names);
        assert!(selected_run.is_some());
        assert_eq!(selected_run.unwrap().id, 123);
    }

    #[test]
    fn test_render_with_timer() {
        let component = create_test_workflow_list();
        let runs = create_test_workflow_runs();
        let repo_names = vec!["test/repo".to_string()];
        let refreshing_repos = Arc::new(RwLock::new(HashSet::new()));
        
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut frame = terminal.get_frame();
        
        // Test with different timer values
        let area = frame.area();
        component.render(&mut frame, area, &runs, &repo_names, 0, &refreshing_repos);
        
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut frame = terminal.get_frame();
        let area = frame.area();
        component.render(&mut frame, area, &runs, &repo_names, 30, &refreshing_repos);
        
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut frame = terminal.get_frame();
        let area = frame.area();
        component.render(&mut frame, area, &runs, &repo_names, 120, &refreshing_repos);
        
        // Test refreshing state
        let mut refreshing = refreshing_repos.write().unwrap();
        refreshing.insert("test/repo".to_string());
        drop(refreshing);
        
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut frame = terminal.get_frame();
        let area = frame.area();
        component.render(&mut frame, area, &runs, &repo_names, 0, &refreshing_repos);
    }
}