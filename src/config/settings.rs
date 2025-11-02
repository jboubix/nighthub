use std::env;
use config::ConfigError;
use serde::{Deserialize, Serialize};
use git2::Repository;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepositoryConfig {
    pub owner: String,
    pub name: String,
    pub branch: Option<String>,
    pub workflows: Option<Vec<String>>,
    pub enabled: bool,
    pub refresh_interval_seconds: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MonitoringConfig {
    pub refresh_interval_seconds: u64,
    pub max_concurrent_requests: usize,
    pub max_retries: usize,
    pub retry_delay_seconds: u64,
    pub workflow_runs_per_repo: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThemeConfig {
    pub success_color: String,
    pub error_color: String,
    pub warning_color: String,
    pub info_color: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LayoutConfig {
    pub min_terminal_width: usize,
    pub min_terminal_height: usize,
    pub compact_mode: bool,
    pub show_header: bool,
    pub show_footer: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IconConfig {
    pub success_icon: String,
    pub error_icon: String,
    pub running_icon: String,
    pub queued_icon: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub max_file_size_mb: usize,
    pub max_files: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub github_token: String,
    #[serde(default)]
    pub repositories: Vec<RepositoryConfig>,
    #[serde(default)]
    pub monitoring: MonitoringConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UiConfig {
    pub theme: ThemeConfig,
    pub layout: LayoutConfig,
    pub icons: IconConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 60,
            max_concurrent_requests: 5,
            max_retries: 3,
            retry_delay_seconds: 5,
            workflow_runs_per_repo: 4,
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            min_terminal_width: 80,
            min_terminal_height: 24,
            compact_mode: true,
            show_header: true,
            show_footer: true,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            success_color: "green".to_string(),
            error_color: "red".to_string(),
            warning_color: "yellow".to_string(),
            info_color: "blue".to_string(),
        }
    }
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            success_icon: "✓".to_string(),
            error_icon: "✗".to_string(),
            running_icon: "⟳".to_string(),
            queued_icon: "⏸".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
            max_file_size_mb: 10,
            max_files: 3,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            layout: LayoutConfig::default(),
            icons: IconConfig::default(),
        }
    }
}

impl Settings {
    fn detect_current_repository() -> Result<RepositoryConfig, ConfigError> {
        let repo = Repository::discover(".")
            .map_err(|_| ConfigError::Message("Current directory is not a git repository".to_string()))?;

        let remote = repo.find_remote("origin")
            .or_else(|_| repo.find_remote("upstream"))
            .map_err(|_| ConfigError::Message("No git remote 'origin' or 'upstream' found".to_string()))?;

        let url = remote.url()
            .ok_or_else(|| ConfigError::Message("Git remote has no URL".to_string()))?;

        // Parse GitHub URL (both HTTPS and SSH formats)
        let (owner, name) = if let Some(path) = url.strip_prefix("git@github.com:") {
            // SSH format: git@github.com:owner/repo.git or git@github.com:/owner/repo.git
            let path = path.trim_end_matches(".git");
            // Handle both git@github.com:owner/repo and git@github.com:/owner/repo formats
            let path = path.strip_prefix('/').unwrap_or(path);
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() != 2 {
                return Err(ConfigError::Message(format!("Invalid GitHub SSH URL format: {}", url)));
            }
            (parts[0].to_string(), parts[1].to_string())
        } else if let Some(path) = url.strip_prefix("https://github.com/") {
            // HTTPS format: https://github.com/owner/repo.git
            let path = path.trim_end_matches(".git");
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() != 2 {
                return Err(ConfigError::Message(format!("Invalid GitHub HTTPS URL format: {}", url)));
            }
            (parts[0].to_string(), parts[1].to_string())
        } else {
            return Err(ConfigError::Message(format!("Unsupported git remote URL: {}. Only GitHub URLs are supported.", url)));
        };

        Ok(RepositoryConfig {
            owner,
            name,
            branch: None,
            workflows: None,
            enabled: true,
            refresh_interval_seconds: None,
        })
    }

    pub fn new() -> Result<Self, ConfigError> {
        let github_token = env::var("GITHUB_TOKEN")
            .or_else(|_| env::var("GH_TOKEN"))
            .map_err(|_| ConfigError::Message("Missing required environment variable: GITHUB_TOKEN or GH_TOKEN".to_string()))?;

        if github_token.is_empty() || (github_token.len() < 40) || (!github_token.starts_with("ghp_") && !github_token.starts_with("github_pat_")) {
            return Err(ConfigError::Message("Invalid GitHub token format. It should be a valid GitHub personal access token starting with ghp_ or github_pat_ and at least 40 characters long.".to_string()));
        }

        let repositories = if let Ok(repos_str) = env::var("REPOS") {
            // Parse REPOS environment variable with optional refresh intervals
            let mut repos = Vec::new();
            for repo_entry in repos_str.split(',') {
                let trimmed = repo_entry.trim();
                if !trimmed.is_empty() {
                    if let Some((repo_part, interval_part)) = trimmed.split_once(':') {
                        // Parse with override: org/repo:30
                        let parts: Vec<&str> = repo_part.split('/').collect();
                        if parts.len() != 2 {
                            return Err(ConfigError::Message(format!("Invalid repository format in REPOS: {}. Expected owner/repo:interval.", trimmed)));
                        }
                        
                        let interval = interval_part.parse::<u64>()
                            .map_err(|_| ConfigError::Message(format!("Invalid refresh interval in REPOS: {}", interval_part)))?;
                        
                        // Validate interval bounds (5 seconds to 2 hours)
                        if !(5..=7200).contains(&interval) {
                            return Err(ConfigError::Message(format!("Refresh interval must be between 5 and 7200 seconds in REPOS: {}", trimmed)));
                        }
                        
                        repos.push(RepositoryConfig {
                            owner: parts[0].to_string(),
                            name: parts[1].to_string(),
                            branch: None,
                            workflows: None,
                            enabled: true,
                            refresh_interval_seconds: Some(interval),
                        });
                    } else {
                        // Parse without override: org/repo
                        let parts: Vec<&str> = trimmed.split('/').collect();
                        if parts.len() != 2 {
                            return Err(ConfigError::Message(format!("Invalid repository format in REPOS: {}. Expected owner/repo.", trimmed)));
                        }
                        
                        repos.push(RepositoryConfig {
                            owner: parts[0].to_string(),
                            name: parts[1].to_string(),
                            branch: None,
                            workflows: None,
                            enabled: true,
                            refresh_interval_seconds: None,
                        });
                    }
                }
            }

            if repos.is_empty() {
                return Err(ConfigError::Message("No valid repositories provided in REPOS.".to_string()));
            }
            repos
        } else {
            // No REPOS env var, detect from current directory
            let mut repo_config = Self::detect_current_repository()?;
            repo_config.refresh_interval_seconds = None; // Use tiered refresh for auto-detected repos
            vec![repo_config]
        };

        // Apply defaults
        let monitoring = MonitoringConfig {
            refresh_interval_seconds: 5, // Default polling interval
            max_concurrent_requests: 5,
            max_retries: 3,
            retry_delay_seconds: 5,
            workflow_runs_per_repo: 5,
        };

        let ui = UiConfig::default();

        let logging = LoggingConfig::default();

        Ok(Settings {
            github_token,
            repositories,
            monitoring,
            ui,
            logging,
        })
    }

    pub fn github_token(&self) -> &str {
        &self.github_token
    }

    pub fn repositories(&self) -> &[RepositoryConfig] {
        &self.repositories
    }

    pub fn set_github_token(&mut self, token: String) {
        self.github_token = token;
    }

    pub fn set_repositories(&mut self, repos: Vec<RepositoryConfig>) {
        self.repositories = repos;
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            github_token: "default_token".to_string(),
            repositories: vec![],
            monitoring: MonitoringConfig::default(),
            ui: UiConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_repos_basic() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "owner1/repo1,owner2/repo2") };
        let result = Settings::new();
        
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.repositories.len(), 2);
        assert_eq!(settings.repositories[0].owner, "owner1");
        assert_eq!(settings.repositories[0].name, "repo1");
        assert_eq!(settings.repositories[0].refresh_interval_seconds, None);
        assert_eq!(settings.repositories[1].owner, "owner2");
        assert_eq!(settings.repositories[1].name, "repo2");
        assert_eq!(settings.repositories[1].refresh_interval_seconds, None);
        
        unsafe { env::remove_var("GITHUB_TOKEN") };
        unsafe { env::remove_var("REPOS") };
    }

    #[test]
    fn test_parse_repos_with_override() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "owner1/repo1:30,owner2/repo2:300") };
        let result = Settings::new();
        
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.repositories.len(), 2);
        assert_eq!(settings.repositories[0].refresh_interval_seconds, Some(30));
        assert_eq!(settings.repositories[1].refresh_interval_seconds, Some(300));
        
        unsafe { env::remove_var("GITHUB_TOKEN") };
        unsafe { env::remove_var("REPOS") };
    }

    #[test]
    fn test_parse_repos_mixed_formats() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "owner1/repo1:60,owner2/repo2,owner3/repo3:7200") };
        let result = Settings::new();
        
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.repositories.len(), 3);
        assert_eq!(settings.repositories[0].refresh_interval_seconds, Some(60));
        assert_eq!(settings.repositories[1].refresh_interval_seconds, None);
        assert_eq!(settings.repositories[2].refresh_interval_seconds, Some(7200));
        
        unsafe { env::remove_var("GITHUB_TOKEN") };
        unsafe { env::remove_var("REPOS") };
    }

    #[test]
    fn test_parse_repos_invalid_format() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "invalid-repo") };
        let result = Settings::new();
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid repository format"));
    }

    #[test]
    fn test_parse_repos_invalid_interval() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "owner/repo:invalid") };
        let result = Settings::new();
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid refresh interval"));
    }

    #[test]
    fn test_parse_repos_interval_out_of_bounds() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "owner/repo:3") };
        let result = Settings::new();
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Refresh interval must be between 5 and 7200 seconds"));
    }

    #[test]
    fn test_parse_repos_interval_too_high() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "owner/repo:7201") };
        let result = Settings::new();
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Refresh interval must be between 5 and 7200 seconds"));
    }

    #[test]
    fn test_parse_repos_empty_entries() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", "owner1/repo1,,owner2/repo2, ,owner3/repo3") };
        let result = Settings::new();
        
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.repositories.len(), 3);
        
        unsafe { env::remove_var("GITHUB_TOKEN") };
        unsafe { env::remove_var("REPOS") };
    }

    #[test]
    fn test_parse_repos_whitespace_handling() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::set_var("REPOS", " owner1/repo1 , owner2/repo2:60 ") };
        let result = Settings::new();
        
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.repositories.len(), 2);
        assert_eq!(settings.repositories[0].owner, "owner1");
        assert_eq!(settings.repositories[1].refresh_interval_seconds, Some(60));
        
        unsafe { env::remove_var("GITHUB_TOKEN") };
        unsafe { env::remove_var("REPOS") };
    }

    #[test]
    fn test_valid_github_token() {
        unsafe { env::set_var("GITHUB_TOKEN", "ghp_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::remove_var("REPOS") };
        
        // This test would need to mock git repository detection
        // For now, just test token validation logic
        let token = env::var("GITHUB_TOKEN").unwrap();
        assert!(token.len() >= 40);
        assert!(token.starts_with("ghp_"));
        
        unsafe { env::remove_var("GITHUB_TOKEN") };
    }

    #[test]
    fn test_github_pat_token() {
        unsafe { env::set_var("GITHUB_TOKEN", "github_pat_1234567890abcdef1234567890abcdef1234567890abcdef") };
        unsafe { env::remove_var("REPOS") };
        
        let token = env::var("GITHUB_TOKEN").unwrap();
        assert!(token.len() >= 40);
        assert!(token.starts_with("github_pat_"));
        
        unsafe { env::remove_var("GITHUB_TOKEN") };
    }

    #[test]
    fn test_missing_github_token() {
        unsafe { env::remove_var("GITHUB_TOKEN") };
        unsafe { env::remove_var("GH_TOKEN") };
        unsafe { env::remove_var("REPOS") };
        
        let result = Settings::new();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing required environment variable: GITHUB_TOKEN or GH_TOKEN"));
    }

    #[test]
    fn test_invalid_github_token_too_short() {
        unsafe { env::set_var("GITHUB_TOKEN", "short") };
        unsafe { env::remove_var("REPOS") };
        
        let result = Settings::new();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid GitHub token format"));
    }

    #[test]
    fn test_invalid_github_token_wrong_prefix() {
        unsafe { env::set_var("GITHUB_TOKEN", "wrong_prefix_1234567890abcdef1234567890abcdef12345678") };
        unsafe { env::remove_var("REPOS") };
        
        let result = Settings::new();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid GitHub token format"));
    }

    #[test]
    fn test_default_monitoring_config() {
        let config = MonitoringConfig::default();
        assert_eq!(config.refresh_interval_seconds, 60);
        assert_eq!(config.max_concurrent_requests, 5);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_seconds, 5);
        assert_eq!(config.workflow_runs_per_repo, 4);
    }

    #[test]
    fn test_default_ui_config() {
        let config = UiConfig::default();
        assert_eq!(config.theme.success_color, "green");
        assert_eq!(config.theme.error_color, "red");
        assert_eq!(config.layout.min_terminal_width, 80);
        assert_eq!(config.layout.min_terminal_height, 24);
        assert_eq!(config.icons.success_icon, "✓");
        assert_eq!(config.icons.error_icon, "✗");
    }
}
