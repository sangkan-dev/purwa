//! Empu — Purwa CLI (Sprint 8).

mod cli;
mod deferred;
mod frontend;
mod generate;
mod migrate_cmd;
mod runners;
mod scaffold;
mod util;

use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::migrate_cmd::{run_migrate, run_migrate_fresh, run_migrate_rollback};
use crate::util::GlobalOpts;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let opts = GlobalOpts {
        verbose: cli.verbose,
        dry_run: cli.dry_run,
    };

    match cli.command {
        Some(Commands::New(args)) => scaffold::run_new(args, opts)?,
        Some(Commands::Serve(args)) => runners::run_serve(args)?,
        Some(Commands::Dev(args)) => runners::run_dev(args)?,
        Some(Commands::Build(args)) => runners::run_build(args)?,
        Some(Commands::RouteList(args)) => runners::run_route_list(args)?,
        Some(Commands::Migrate(args)) => run_migrate(args).await?,
        Some(Commands::MigrateRollback(args)) => run_migrate_rollback(args).await?,
        Some(Commands::MigrateFresh(args)) => run_migrate_fresh(args).await?,
        Some(Commands::MakeRequest(args)) => {
            generate::make_request(&args.name, args.output, opts)?;
        }
        Some(Commands::MakeAuth(args)) => {
            generate::make_auth(&args.migrations_dir, args.output, opts)?;
        }
        Some(Commands::MakeController(args)) => {
            generate::make_controller(&args.name, args.output, opts)?;
        }
        Some(Commands::MakeService(args)) => {
            generate::make_service(&args.name, args.output, opts)?;
        }
        Some(Commands::MakeModel(args)) => {
            generate::make_model(&args.name, args.output, args.sea_orm, opts)?;
        }
        Some(Commands::MakeMigration(args)) => {
            generate::make_migration(&args.name, &args.migrations_dir, opts)?;
        }
        Some(Commands::MakeSeeder(args)) => {
            generate::make_seeder(&args.name, args.output_dir, opts)?;
        }
        Some(Commands::MakeJob(args)) => {
            generate::make_job(&args.name, args.output_dir, opts)?;
        }
        Some(Commands::MakePolicy(_)) => deferred::print_deferred("make:policy"),
        Some(Commands::DbSeed(args)) => runners::run_db_seed(args)?,
        Some(Commands::QueueWork(args)) => runners::run_queue_work(args)?,
        Some(Commands::InertiaSetup(args)) => frontend::run_inertia_setup(args, opts)?,
        None => {
            eprintln!(
                "empu — Purwa CLI. Try: empu new, empu serve, empu route:list, empu migrate, empu make:request, …"
            );
        }
    }
    Ok(())
}
