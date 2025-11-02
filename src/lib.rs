pub mod config;
pub mod error;
pub mod github;
pub mod ui;
pub mod utils;

pub fn setup_logging() {
    // Simple logging setup without external dependencies
    env_logger::init();
}