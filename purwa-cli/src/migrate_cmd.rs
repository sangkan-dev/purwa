//! Database migration commands.

use purwa_core::AppConfig;
use purwa_orm::PurwaOrmError;

use crate::cli::MigrateArgs;

pub fn resolve_database_url(
    cli_url: Option<String>,
    config_path: Option<&std::path::Path>,
) -> Result<String, PurwaOrmError> {
    if let Some(u) = cli_url {
        let t = u.trim();
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    let cfg = AppConfig::load_with_file(config_path)?;
    purwa_orm::database_url_from_config(&cfg)
}

pub async fn run_migrate(args: MigrateArgs) -> Result<(), PurwaOrmError> {
    let url = resolve_database_url(args.database_url, args.config.as_deref())?;
    let pool = purwa_orm::connect_pool(&url).await?;
    purwa_orm::migrate_up(&pool, &args.migrations_dir).await?;
    eprintln!("Applied migrations from {}", args.migrations_dir.display());
    Ok(())
}

pub async fn run_migrate_rollback(args: MigrateArgs) -> Result<(), PurwaOrmError> {
    let url = resolve_database_url(args.database_url, args.config.as_deref())?;
    let pool = purwa_orm::connect_pool(&url).await?;
    purwa_orm::migrate_rollback_one(&pool, &args.migrations_dir).await?;
    eprintln!("Rollback step finished (see docs for reversible migrations).");
    Ok(())
}

pub async fn run_migrate_fresh(args: MigrateArgs) -> Result<(), PurwaOrmError> {
    let url = resolve_database_url(args.database_url, args.config.as_deref())?;
    let pool = purwa_orm::connect_pool(&url).await?;
    purwa_orm::migrate_fresh(&pool, &args.migrations_dir).await?;
    eprintln!(
        "Fresh migrations applied from {}",
        args.migrations_dir.display()
    );
    Ok(())
}
