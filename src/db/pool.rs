//! Database connection pool management

use deadpool_postgres::{Config, Pool, Runtime, ManagerConfig, RecyclingMethod};
use tokio_postgres::NoTls;
use thiserror::Error;
use tracing::info;

/// Database-related errors
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::CreatePoolError),
    #[error("Query error: {0}")]
    Query(#[from] tokio_postgres::Error),
    #[error("Pool get error: {0}")]
    PoolGet(#[from] deadpool_postgres::PoolError),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Database connection pool wrapper
#[derive(Clone)]
pub struct DbPool {
    pool: Pool,
}

impl DbPool {
    /// Create a new database pool from a connection string
    pub fn new(database_url: &str) -> Result<Self, DbError> {
        // Parse the connection URL
        let url = url::Url::parse(database_url)
            .map_err(|e| DbError::Config(format!("Invalid database URL: {}", e)))?;

        let host = url.host_str()
            .ok_or_else(|| DbError::Config("Missing host in DATABASE_URL".to_string()))?;
        let port = url.port().unwrap_or(5432);
        let user = url.username();
        let password = url.password().unwrap_or("");
        let dbname = url.path().trim_start_matches('/');

        let mut cfg = Config::new();
        cfg.host = Some(host.to_string());
        cfg.port = Some(port);
        cfg.user = Some(user.to_string());
        cfg.password = Some(password.to_string());
        cfg.dbname = Some(dbname.to_string());

        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

        info!(
            host = %host,
            port = %port,
            dbname = %dbname,
            "Database pool created"
        );

        Ok(DbPool { pool })
    }

    /// Get the underlying pool reference
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Get a connection from the pool
    pub async fn get(&self) -> Result<deadpool_postgres::Object, DbError> {
        Ok(self.pool.get().await?)
    }

    /// Test the database connection
    pub async fn test_connection(&self) -> Result<(), DbError> {
        let client = self.get().await?;
        client.query_one("SELECT 1", &[]).await?;
        info!("Database connection test successful");
        Ok(())
    }
}
