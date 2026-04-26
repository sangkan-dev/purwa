//! Database layer for Purwa — SQLx-first; optional SeaORM behind the `sea-orm` feature.

#[cfg(feature = "sea-orm")]
pub mod sea;

use std::path::{Path, PathBuf};

use purwa_core::{AppConfig, PurwaConfigError};
use sqlx::Executor;
use sqlx::PgPool;
use sqlx::migrate::{Migrate, MigrateError, Migrator};
use thiserror::Error;

/// Errors from connection, migrations, or missing configuration.
#[derive(Debug, Error)]
pub enum PurwaOrmError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migrate(#[from] MigrateError),
    #[error(transparent)]
    Config(#[from] PurwaConfigError),
    #[error(
        "database URL is not set (purwa.toml [database].url, PURWA_DATABASE__URL, or DATABASE_URL)"
    )]
    DatabaseUrlMissing,
}

/// Default migrations directory relative to the process working directory (PRD §6).
pub fn default_migrations_dir() -> PathBuf {
    PathBuf::from("database/migrations")
}

/// Open a pool using a Postgres URL (typically from [`AppConfig::database_url`]).
pub async fn connect_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

/// Resolve the database URL from loaded config (no `DATABASE_URL` fallback beyond [`AppConfig::database_url`]).
pub fn database_url_from_config(cfg: &AppConfig) -> Result<String, PurwaOrmError> {
    cfg.database_url().ok_or(PurwaOrmError::DatabaseUrlMissing)
}

/// Apply all pending migrations from `dir` (files named `VERSION_description.sql` per SQLx).
pub async fn migrate_up(pool: &PgPool, dir: &Path) -> Result<(), PurwaOrmError> {
    let m = Migrator::new(dir).await?;
    m.run(pool).await?;
    Ok(())
}

/// Undo applied migrations with version greater than `target` (SQLx `Migrator::undo`).
pub async fn migrate_undo(pool: &PgPool, dir: &Path, target: i64) -> Result<(), PurwaOrmError> {
    let m = Migrator::new(dir).await?;
    m.undo(pool, target).await?;
    Ok(())
}

/// Roll back the latest migration step. Only affects **reversible** migrations (paired `.up.sql` / `.down.sql`).
///
/// Simple single-file `.sql` migrations have no down phase; in that case this returns successfully
/// without changing the schema.
pub async fn migrate_rollback_one(pool: &PgPool, dir: &Path) -> Result<(), PurwaOrmError> {
    let Some(target) = rollback_target(pool).await? else {
        return Ok(());
    };
    migrate_undo(pool, dir, target).await
}

async fn rollback_target(pool: &PgPool) -> Result<Option<i64>, PurwaOrmError> {
    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table().await?;
    if let Some(v) = conn.dirty_version().await? {
        return Err(MigrateError::Dirty(v).into());
    }
    let applied = conn.list_applied_migrations().await?;
    if applied.is_empty() {
        return Ok(None);
    }
    let mut versions: Vec<i64> = applied.into_iter().map(|a| a.version).collect();
    versions.sort();
    let target = if versions.len() <= 1 {
        0_i64
    } else {
        versions[versions.len() - 2]
    };
    Ok(Some(target))
}

/// **Development only:** drop and recreate the `public` schema, then run all migrations from `dir`.
pub async fn migrate_fresh(pool: &PgPool, dir: &Path) -> Result<(), PurwaOrmError> {
    pool.execute("DROP SCHEMA IF EXISTS public CASCADE")
        .await
        .map_err(PurwaOrmError::Sqlx)?;
    pool.execute("CREATE SCHEMA public")
        .await
        .map_err(PurwaOrmError::Sqlx)?;
    pool.execute("GRANT ALL ON SCHEMA public TO PUBLIC")
        .await
        .map_err(PurwaOrmError::Sqlx)?;
    migrate_up(pool, dir).await
}
