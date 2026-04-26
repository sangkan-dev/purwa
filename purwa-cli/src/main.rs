//! Empu — Purwa CLI (full generators land in Sprint 8).

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use purwa_core::AppConfig;
use purwa_orm::PurwaOrmError;

#[derive(Parser)]
#[command(
    name = "empu",
    version,
    about = "Empu — Purwa CLI (Artisan-equivalent)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List registered HTTP routes (full `empu` + app integration in Sprint 8)
    #[command(name = "route:list")]
    RouteList,
    /// Apply pending SQL migrations from `database/migrations` (or `--migrations-dir`)
    Migrate(MigrateArgs),
    /// Roll back the latest reversible migration (`.up.sql` / `.down.sql` pair)
    #[command(name = "migrate:rollback")]
    MigrateRollback(MigrateArgs),
    /// Drop and recreate `public`, then migrate (development only)
    #[command(name = "migrate:fresh")]
    MigrateFresh(MigrateArgs),
}

#[derive(Parser)]
struct MigrateArgs {
    /// Postgres URL (overrides config / `DATABASE_URL`)
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,
    /// Path to `purwa.toml` (default: discover `purwa` in CWD)
    #[arg(long)]
    config: Option<PathBuf>,
    /// Migration SQL directory
    #[arg(long, default_value = "database/migrations")]
    migrations_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::RouteList) => {
            eprintln!("empu route:list — stub (Sprint 2).");
            eprintln!(
                "Routes are collected from your app binary via `inventory`. To print the table:"
            );
            eprintln!("  println!(\"{{}}\", purwa::format_route_table());");
            eprintln!("Sprint 8: this command will delegate to your built application.");
        }
        Some(Commands::Migrate(args)) => {
            run_migrate(args).await?;
        }
        Some(Commands::MigrateRollback(args)) => {
            run_migrate_rollback(args).await?;
        }
        Some(Commands::MigrateFresh(args)) => {
            run_migrate_fresh(args).await?;
        }
        None => {
            eprintln!(
                "empu — Purwa CLI. Try: empu migrate, empu migrate:rollback, empu migrate:fresh, empu route:list"
            );
        }
    }
    Ok(())
}

fn resolve_database_url(
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

async fn run_migrate(args: MigrateArgs) -> Result<(), PurwaOrmError> {
    let url = resolve_database_url(args.database_url, args.config.as_deref())?;
    let pool = purwa_orm::connect_pool(&url).await?;
    purwa_orm::migrate_up(&pool, &args.migrations_dir).await?;
    eprintln!("Applied migrations from {}", args.migrations_dir.display());
    Ok(())
}

async fn run_migrate_rollback(args: MigrateArgs) -> Result<(), PurwaOrmError> {
    let url = resolve_database_url(args.database_url, args.config.as_deref())?;
    let pool = purwa_orm::connect_pool(&url).await?;
    purwa_orm::migrate_rollback_one(&pool, &args.migrations_dir).await?;
    eprintln!("Rollback step finished (see docs for reversible migrations).");
    Ok(())
}

async fn run_migrate_fresh(args: MigrateArgs) -> Result<(), PurwaOrmError> {
    let url = resolve_database_url(args.database_url, args.config.as_deref())?;
    let pool = purwa_orm::connect_pool(&url).await?;
    purwa_orm::migrate_fresh(&pool, &args.migrations_dir).await?;
    eprintln!(
        "Fresh migrations applied from {}",
        args.migrations_dir.display()
    );
    Ok(())
}
