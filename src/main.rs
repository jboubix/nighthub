use nighthub::{
    config::settings::Settings,
    ui::app::AppState,
    ui::components::workflow_list::WorkflowListComponent,
    setup_logging,
};
use clap::Parser;
use std::error::Error;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use ctrlc;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, poll},
    terminal::{disable_raw_mode, enable_raw_mode},
};

#[derive(Parser)]
#[command(name = "nighthub")]
#[command(about = "A terminal monitor for GitHub Actions")]
struct Args {
    #[arg(long)]
    fixed: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _args = Args::parse();
    setup_logging();

    let settings = Settings::new()?;
    let mut app_state = AppState::new(settings).await?;

    app_state.refresh().await?;

    enable_raw_mode()?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    let mut workflow_list = WorkflowListComponent::new();

    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_clone = Arc::clone(&should_exit);

    ctrlc::set_handler(move || {
        should_exit_clone.store(true, Ordering::Relaxed);
    }).expect("Error setting Ctrl+C handler");

    loop {
        if should_exit.load(Ordering::Relaxed) {
            break;
        }

        // Check if it's time to refresh
        if app_state.seconds_until_refresh() == 0 {
            if let Err(e) = app_state.refresh().await {
                eprintln!("Error refreshing: {}", e);
            }
        }

        terminal.draw(|f| {
            // Create a list of repository names for the UI component
            let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();

            // Get seconds until next refresh
            let seconds_until_refresh = app_state.seconds_until_refresh();

            // Render the workflow list component with timer
            workflow_list.render(f, f.area(), &app_state.workflow_runs, &repo_names, seconds_until_refresh);

            // Render context menu if open
            if let Some(popup_type) = app_state.popup {
                match popup_type {
                    nighthub::ui::app::PopupType::ContextMenu => {
                        let area = f.area();
                        let context_menu_area = ratatui::layout::Rect {
                            x: area.width / 2 - 10,
                            y: area.height / 2 - 2,
                            width: 20.min(area.width),
                            height: 4.min(area.height),
                        };
                        app_state.context_menu.render(f, context_menu_area);
                    }
                    _ => {}
                }
            }
        })?;

        // Poll for events with timeout to keep UI responsive and update timer
        if poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events, not repeat/release events
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
                        KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => {
                        if app_state.popup.is_none() {
                            workflow_list.next_repo(app_state.repositories.len());
                            // Sync app_state with workflow_list selection
                            app_state.selected_repo = Some(workflow_list.selected_repo_index);
                            app_state.selected_run = Some(workflow_list.selected_run_index);
                        } else if app_state.popup == Some(nighthub::ui::app::PopupType::ContextMenu) {
                            app_state.context_menu.next();
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if app_state.popup.is_none() {
                            workflow_list.previous_repo(app_state.repositories.len());
                            // Sync app_state with workflow_list selection
                            app_state.selected_repo = Some(workflow_list.selected_repo_index);
                            app_state.selected_run = Some(workflow_list.selected_run_index);
                        } else if app_state.popup == Some(nighthub::ui::app::PopupType::ContextMenu) {
                            app_state.context_menu.previous();
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if app_state.popup.is_none() {
                            let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();
                            workflow_list.next_run(&app_state.workflow_runs, &repo_names);
                            // Sync app_state with workflow_list selection
                            app_state.selected_repo = Some(workflow_list.selected_repo_index);
                            app_state.selected_run = Some(workflow_list.selected_run_index);
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        if app_state.popup.is_none() {
                            let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();
                            workflow_list.previous_run(&app_state.workflow_runs, &repo_names);
                            // Sync app_state with workflow_list selection
                            app_state.selected_repo = Some(workflow_list.selected_repo_index);
                            app_state.selected_run = Some(workflow_list.selected_run_index);
                        }
                    }
                    KeyCode::Char('r') => {
                        if app_state.popup.is_none() {
                            // Force immediate refresh
                            if let Err(e) = app_state.refresh().await {
                                eprintln!("Error refreshing: {}", e);
                            }
                        }
                    }
                    KeyCode::Enter => app_state.handle_key("enter"),
                    KeyCode::Esc => app_state.handle_key("esc"),
                    _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}
