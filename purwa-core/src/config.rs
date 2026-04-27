//! Typed configuration from `purwa.toml`, merged with environment variables.
//!
//! # Resolution order
//!
//! 1. Optional `purwa.toml` (or an explicit path from [`AppConfig::load_with_file`]).
//! 2. Environment variables with prefix `PURWA` and nested keys separated by `__`
//!    (e.g. `PURWA_SERVER__PORT=8080`, `PURWA_DATABASE__URL=postgres://...`).
//!
//! After load, [`AppConfig::database_url`] also checks `DATABASE_URL` (no prefix) when
//! `[database].url` is unset. Inertia asset versioning uses [`InertiaSection`] / `PURWA_INERTIA__*`.
//!
//! `dotenvy::dotenv()` runs from [`AppConfig::load`] / [`AppConfig::load_with_file`] so a project
//! `.env` is loaded when present (missing file is ignored).
//!
//! # Router state
//!
//! Use [`crate::AppState`] with Axum `State` and `axum::extract::FromRef` for sub-state extraction
//! (see `purwa-core` integration tests).

use std::path::Path;
use std::sync::Arc;

use config::{Config, Environment, File};
use serde::Deserialize;
use thiserror::Error;

/// Errors while loading or deserializing configuration.
#[derive(Debug, Error)]
pub enum PurwaConfigError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
}

/// Top-level `[app]` section in `purwa.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppSection {
    /// Application display name.
    pub name: String,
}

impl Default for AppSection {
    fn default() -> Self {
        Self {
            name: "purwa-app".to_string(),
        }
    }
}

/// Top-level `[server]` section in `purwa.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
}

impl Default for ServerSection {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
        }
    }
}

/// Top-level `[database]` section in `purwa.toml`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DatabaseSection {
    /// Postgres connection URL (optional if `DATABASE_URL` is set at runtime).
    pub url: Option<String>,
}

/// Top-level `[queue]` section in `purwa.toml` (Phase 2).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct QueueSection {
    /// Redis connection URL (optional if `REDIS_URL` is set at runtime).
    pub redis_url: Option<String>,
    /// Queue name used for key prefixes.
    pub name: String,
}

impl Default for QueueSection {
    fn default() -> Self {
        Self {
            redis_url: None,
            name: "default".to_string(),
        }
    }
}

/// Top-level `[inertia]` section — asset versioning for Inertia.js (Sprint 6).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct InertiaSection {
    /// Bumped when frontend assets change; compared to `X-Inertia-Version` on Inertia requests.
    pub asset_version: String,
}

impl Default for InertiaSection {
    fn default() -> Self {
        Self {
            asset_version: "1".to_string(),
        }
    }
}

/// Framework configuration: `purwa.toml` + env (`PURWA_*`).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub app: AppSection,
    pub server: ServerSection,
    pub database: DatabaseSection,
    pub queue: QueueSection,
    pub inertia: InertiaSection,
}

impl AppConfig {
    /// Load using default discovery: optional `./purwa.toml` (via `config` file name `purwa`) + env.
    pub fn load() -> Result<Arc<Self>, PurwaConfigError> {
        Self::load_with_file(None)
    }

    /// Load from an explicit `purwa.toml` path, or when `None` use `File::with_name("purwa")` in the process CWD.
    pub fn load_with_file(purwa_toml: Option<&Path>) -> Result<Arc<Self>, PurwaConfigError> {
        dotenvy::dotenv().ok();
        let mut builder = Config::builder();
        match purwa_toml {
            Some(path) => {
                builder = builder.add_source(File::from(path).required(true));
            }
            None => {
                builder = builder.add_source(File::with_name("purwa").required(false));
            }
        }
        builder = builder.add_source(
            Environment::with_prefix("PURWA")
                .separator("__")
                .try_parsing(true),
        );
        let cfg = builder.build()?;
        let app: AppConfig = cfg.try_deserialize()?;
        Ok(Arc::new(app))
    }

    /// Resolved database connection URL for SQLx / `PgPool`.
    ///
    /// Order: `[database].url` from config (file + `PURWA_DATABASE__URL`), then `DATABASE_URL`.
    pub fn database_url(&self) -> Option<String> {
        if let Some(ref u) = self.database.url {
            let t = u.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        std::env::var("DATABASE_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Resolved Redis connection URL for queue workers.
    ///
    /// Order: `[queue].redis_url` from config (file + `PURWA_QUEUE__REDIS_URL`), then `REDIS_URL`.
    pub fn queue_redis_url(&self) -> Option<String> {
        if let Some(ref u) = self.queue.redis_url {
            let t = u.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        std::env::var("REDIS_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}
