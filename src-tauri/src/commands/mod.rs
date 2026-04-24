pub mod config;

// Re-export all commands for easy registration
pub use config::save_config;
pub use config::load_config;
