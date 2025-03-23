// Re-export the AppConfig struct and the load_config function
pub mod config;     // Contains the AppConfig struct.
pub mod settings;   // Contains the logic to load the config.

pub use config::AppConfig;      // Re-export AppConfig for easy access.
pub use settings::load_config;  // Re-export load_config for easy access.
