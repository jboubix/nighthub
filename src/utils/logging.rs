use tokio::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
}

pub struct AsyncLogger {
    sender: mpsc::UnboundedSender<LogEntry>,
}

impl AsyncLogger {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<LogEntry>();
        
        // For terminal UI, we don't output logs to avoid interfering with the UI
        // Just consume the log entries without printing them
        tokio::spawn(async move {
            while let Some(_entry) = receiver.recv().await {
                // Silently consume log entries
            }
        });
        
        Self { sender }
    }
    
    pub fn log(&self, level: LogLevel, message: String) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let entry = LogEntry {
            timestamp,
            level,
            message,
        };
        
        let _ = self.sender.send(entry);
    }
    
    pub fn error(&self, message: String) {
        self.log(LogLevel::Error, message);
    }
    
    pub fn warn(&self, message: String) {
        self.log(LogLevel::Warn, message);
    }
    
    pub fn info(&self, message: String) {
        self.log(LogLevel::Info, message);
    }
    
    pub fn debug(&self, message: String) {
        self.log(LogLevel::Debug, message);
    }
}

impl Default for AsyncLogger {
    fn default() -> Self {
        Self::new()
    }
}

// Global async logger instance
lazy_static::lazy_static! {
    pub static ref ASYNC_LOGGER: AsyncLogger = AsyncLogger::new();
}

// Convenience functions for global logger
pub fn log_error(message: String) {
    ASYNC_LOGGER.error(message);
}

pub fn log_warn(message: String) {
    ASYNC_LOGGER.warn(message);
}

pub fn log_info(message: String) {
    ASYNC_LOGGER.info(message);
}

pub fn log_debug(message: String) {
    ASYNC_LOGGER.debug(message);
}