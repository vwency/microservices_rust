// Define the module structure.
pub mod config;  // This is where `AppConfig` and `load_config` are located.
pub mod settings; // This can contain any additional settings or configuration logic.

/// Re-export the `AppConfig` struct and `load_config` function for easy access.
pub use config::AppConfig;
pub use settings::load_config;
