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
use tokio::signal;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{enable_raw_mode, disable_raw_mode, Clear},
    execute,
};


#[derive(Parser)]
#[command(name = "nighthub")]
#[command(about = "A terminal monitor for GitHub Actions")]
struct Args {
    #[arg(long)]
    fixed: bool,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn Error>> {
    let _args = Args::parse();
    setup_logging();

    let settings = Settings::new()?;
    let mut app_state = AppState::new(settings).await?;

    enable_raw_mode()?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    execute!(std::io::stdout(), Clear(crossterm::terminal::ClearType::All))?;

    let workflow_list = WorkflowListComponent::new();

    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_clone = Arc::clone(&should_exit);

    // Set up graceful shutdown signal handler
    let shutdown_handle = tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to setup SIGTERM handler");
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("Failed to setup SIGINT handler");
        
        tokio::select! {
            _ = sigterm.recv() => {
                should_exit_clone.store(true, Ordering::Relaxed);
            }
            _ = sigint.recv() => {
                should_exit_clone.store(true, Ordering::Relaxed);
            }
        }
    });

    loop {
        if should_exit.load(Ordering::Relaxed) {
            break;
        }

        // Auto refresh when timer reaches 0
        if app_state.seconds_until_refresh() == 0 {
            let _ = app_state.refresh(false).await;
        }

        terminal.draw(|f| {
            // Create a list of repository names for UI component
            let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();

            // Render workflow list component with timer
            workflow_list.render(f, f.area(), &app_state.workflow_runs, &repo_names, app_state.seconds_until_refresh(), app_state.is_refreshing);

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
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') | KeyCode::Down => {
                            if app_state.popup.is_none() {
                                app_state.next_repo();
                            } else if let Some(nighthub::ui::app::PopupType::ContextMenu) = app_state.popup {
                                app_state.context_menu.next();
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if app_state.popup.is_none() {
                                app_state.previous_repo();
                            } else if let Some(nighthub::ui::app::PopupType::ContextMenu) = app_state.popup {
                                app_state.context_menu.previous();
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            if app_state.popup.is_none() {
                                app_state.next_run();
                            }
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            if app_state.popup.is_none() {
                                app_state.previous_run();
                            }
                        }
                        KeyCode::Char('f') => {
                            if app_state.popup.is_none() {
                                // Force immediate refresh of ALL repos (manual refresh)
                                let _ = app_state.refresh(true).await;
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

    // Abort the shutdown handler task
    shutdown_handle.abort();
    
    disable_raw_mode()?;
    Ok(())
}
