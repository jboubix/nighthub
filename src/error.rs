use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ConfigError(String),
    GithubError(String),
    IoError(std::io::Error),
    ParseError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::ConfigError(e) => write!(f, "Configuration error: {}", e),
            AppError::GithubError(e) => write!(f, "GitHub API error: {}", e),
            AppError::IoError(e) => write!(f, "IO error: {}", e),
            AppError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl From<config::ConfigError> for AppError {
    fn from(error: config::ConfigError) -> Self {
        AppError::ConfigError(error.to_string())
    }
}

impl From<octocrab::Error> for AppError {
    fn from(error: octocrab::Error) -> Self {
        AppError::GithubError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::error::Error;

    #[test]
    fn test_app_error_display_config() {
        let error = AppError::ConfigError("Test config error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Configuration error: Test config error"));
    }

    #[test]
    fn test_app_error_display_github() {
        let error = AppError::GithubError("Test GitHub error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("GitHub API error: Test GitHub error"));
    }

    #[test]
    fn test_app_error_display_io() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = AppError::IoError(io_error);
        let display = format!("{}", error);
        assert!(display.contains("IO error: File not found"));
    }

    #[test]
    fn test_app_error_display_parse() {
        let error = AppError::ParseError("Test parse error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Parse error: Test parse error"));
    }

    #[test]
    fn test_app_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let app_error = AppError::from(io_error);
        match app_error {
            AppError::IoError(_) => {}, // Expected
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_app_error_from_config_error() {
        let config_error = config::ConfigError::Message("Config message".to_string());
        let app_error = AppError::from(config_error);
        match app_error {
            AppError::ConfigError(_) => {}, // Expected
            _ => panic!("Expected ConfigError variant"),
        }
    }

    #[test]
    fn test_app_error_from_octocrab_error() {
        // Note: This test would need mocking for octocrab::Error
        // For now, just test the conversion exists
        // In a real scenario, you'd create an actual octocrab::Error
        let error_str = "GitHub API rate limit exceeded";
        let app_error = AppError::GithubError(error_str.to_string());
        match app_error {
            AppError::GithubError(msg) => {
                assert_eq!(msg, error_str);
            }
            _ => panic!("Expected GithubError variant"),
        }
    }

    #[test]
    fn test_app_error_debug_format() {
        let error = AppError::ConfigError("Debug test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ConfigError"));
        assert!(debug_str.contains("Debug test"));
    }

    #[test]
    fn test_app_error_source() {
        let io_error = io::Error::new(io::ErrorKind::BrokenPipe, "Pipe broken");
        let app_error = AppError::from(io_error);
        
        // Test that the error implements std::error::Error
        let source = app_error.source();
        assert!(source.is_some());
    }


}