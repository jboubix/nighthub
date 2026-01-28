use nighthub::{
    config::settings::Settings,
    ui::app::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing single repository refresh...");
    
    // Create test settings with single repo
    let settings = Settings::new()?;
    
    // Create app state
    let mut app_state = AppState::new(settings).await?;
    
    println!("Created app state with {} repositories", app_state.repositories.len());
    
    // Test refresh with single repo
    println!("Testing refresh with single repository...");
    let refresh_result = app_state.refresh(true).await;
    
    match refresh_result {
        Ok(_) => println!("Refresh completed successfully"),
        Err(e) => println!("Refresh failed: {}", e),
    }
    
    // Check refreshing state
    {
        let refreshing_repos = app_state.refreshing_repos.read().unwrap();
        println!("Currently refreshing repos: {:?}", refreshing_repos);
        
        // Test UI rendering data
        let repo_names: Vec<String> = app_state.repositories.iter().map(|r| r.full_name.clone()).collect();
        println!("Repository names for UI: {:?}", repo_names);
        
        // Simulate what UI component would see
        let is_any_refreshing = !refreshing_repos.is_empty();
        println!("UI should show refreshing: {}", is_any_refreshing);
        
        for repo_name in &repo_names {
            let is_refreshing = refreshing_repos.contains(repo_name);
            let indicator = if is_refreshing { "🔄 " } else { "" };
            println!("UI display: {}{}: 0", indicator, repo_name);
        }
    }
    
    // Test manual refresh again
    println!("\nTesting second refresh...");
    let refresh_result2 = app_state.refresh(true).await;
    match refresh_result2 {
        Ok(_) => println!("Second refresh completed successfully"),
        Err(e) => println!("Second refresh failed: {}", e),
    }
    
    Ok(())
}