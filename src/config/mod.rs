//! Configuration module for the mockup service

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

/// Main application settings
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub templates: TemplateSettings,
    pub cloudinary: CloudinarySettings,
    pub database: DatabaseSettings,
    #[serde(default)]
    pub r2: Option<R2Settings>,
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

/// Cloudflare R2 configuration for POD asset storage
#[derive(Debug, Clone, Deserialize)]
pub struct R2Settings {
    /// Cloudflare account ID
    pub account_id: String,
    /// R2 Access Key ID
    pub access_key_id: String,
    /// R2 Secret Access Key
    pub secret_access_key: String,
    /// Bucket name for POD assets
    pub bucket_name: String,
    /// Public URL prefix for assets (optional, for CDN)
    pub public_url_prefix: Option<String>,
}

impl Settings {
    /// Load configuration from files and environment variables
    ///
    /// Configuration priority (highest to lowest):
    /// 1. Environment variables (prefixed with MOCKUP)
    /// 2. config/local.toml (gitignored)
    /// 3. config/default.toml
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = std::env::var("CONFIG_PATH")
            .or_else(|_| std::env::var("CONFIG_DIR"))
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
                    .try_parsing(true),
            );

        let mut settings: Settings = builder.build()?.try_deserialize()?;

        if settings.database.url.is_empty() {
            if let Some(database_url) = first_env(&[
                "DATABASE_URL",
                "MOCKUP_DATABASE__URL",
                "MOCKUP__DATABASE__URL",
            ]) {
                settings.database.url = database_url;
            }
        }

        if settings.database.max_connections.is_none() {
            if let Some(max_connections) = first_env(&["MOCKUP_DATABASE__MAX_CONNECTIONS"]) {
                if let Ok(parsed) = max_connections.parse::<u32>() {
                    settings.database.max_connections = Some(parsed);
                }
            }
        }

        settings.r2 = resolve_r2_settings(settings.r2);

        Ok(settings)
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
            r2: None,
        }
    }
}

fn first_env(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| std::env::var(key).ok())
}

fn resolve_r2_settings(existing: Option<R2Settings>) -> Option<R2Settings> {
    let account_id = first_env(&["MOCKUP_R2__ACCOUNT_ID", "R2_ACCOUNT_ID"]);
    let access_key_id = first_env(&["MOCKUP_R2__ACCESS_KEY_ID", "R2_ACCESS_KEY_ID"]);
    let secret_access_key = first_env(&["MOCKUP_R2__SECRET_ACCESS_KEY", "R2_SECRET_ACCESS_KEY"]);
    let bucket_name = first_env(&["MOCKUP_R2__BUCKET_NAME", "R2_BUCKET_NAME"]);
    let public_url_prefix = first_env(&["MOCKUP_R2__PUBLIC_URL_PREFIX", "R2_PUBLIC_URL_PREFIX"]);

    match existing {
        Some(mut settings) => {
            if let Some(value) = account_id {
                settings.account_id = value;
            }
            if let Some(value) = access_key_id {
                settings.access_key_id = value;
            }
            if let Some(value) = secret_access_key {
                settings.secret_access_key = value;
            }
            if let Some(value) = bucket_name {
                settings.bucket_name = value;
            }
            if public_url_prefix.is_some() {
                settings.public_url_prefix = public_url_prefix;
            }
            Some(settings)
        }
        None => match (account_id, access_key_id, secret_access_key) {
            (Some(account_id), Some(access_key_id), Some(secret_access_key)) => Some(R2Settings {
                account_id,
                access_key_id,
                secret_access_key,
                bucket_name: bucket_name.unwrap_or_else(default_r2_bucket_name),
                public_url_prefix,
            }),
            _ => None,
        },
    }
}

pub fn service_name() -> String {
    first_env(&["MOCKUP_SERVICE__NAME", "SERVICE_NAME"])
        .unwrap_or_else(|| "r-image-magic".to_string())
}

pub fn pricing_url() -> String {
    first_env(&["MOCKUP_SERVICE__PRICING_URL", "PRICING_URL"])
        .unwrap_or_else(|| "https://r-image-magic.com/pricing".to_string())
}

pub fn service_user_agent() -> String {
    first_env(&["MOCKUP_SERVICE__USER_AGENT", "SERVICE_USER_AGENT"])
        .unwrap_or_else(|| format!("{}/{}", service_name(), env!("CARGO_PKG_VERSION")))
}

pub fn default_r2_bucket_name() -> String {
    first_env(&["MOCKUP_SERVICE__R2_BUCKET_DEFAULT", "R2_BUCKET_DEFAULT"])
        .unwrap_or_else(|| "r-image-magic-pod-assets".to_string())
}
