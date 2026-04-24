pub mod config;

// Re-export all commands for easy registration
pub use config::save_config;
pub use config::load_config;
pub use config::get_default_config;
pub use config::get_config_status;
