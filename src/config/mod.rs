//! Configuration module for the mockup service

use serde::Deserialize;
use config::{Config, ConfigError, Environment, File};
use std::path::PathBuf;

/// Main application settings
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub templates: TemplateSettings,
    pub cloudinary: CloudinarySettings,
    pub database: DatabaseSettings,
}

/// HTTP server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

/// Template configuration
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateSettings {
    pub path: PathBuf,
}

/// Cloudinary configuration for uploading generated mockups
#[derive(Debug, Clone, Deserialize)]
pub struct CloudinarySettings {
    pub cloud_name: String,
    pub api_key: String,
    pub api_secret: String,
    pub upload_preset: Option<String>,
}

/// Database configuration for PostgreSQL
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: Option<u32>,
}

impl Settings {
    /// Load configuration from files and environment variables
    ///
    /// Configuration priority (highest to lowest):
    /// 1. Environment variables (prefixed with MOCKUP_)
    /// 2. config/local.toml (gitignored)
    /// 3. config/default.toml
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = std::env::var("CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("config"));

        let builder = Config::builder()
            // Start with default configuration
            .add_source(File::from(config_dir.join("default.toml")).required(false))
            // Add local overrides (gitignored)
            .add_source(File::from(config_dir.join("local.toml")).required(false))
            // Add environment variables (MOCKUP_SERVER__PORT, etc.)
            .add_source(
                Environment::with_prefix("MOCKUP")
                    .separator("__")
                    .try_parsing(true)
            );

        builder.build()?.try_deserialize()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            server: ServerSettings {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
            },
            templates: TemplateSettings {
                path: PathBuf::from("assets/templates"),
            },
            cloudinary: CloudinarySettings {
                cloud_name: String::new(),
                api_key: String::new(),
                api_secret: String::new(),
                upload_preset: None,
            },
            database: DatabaseSettings {
                url: String::new(),
                max_connections: Some(10),
            },
        }
    }
}
