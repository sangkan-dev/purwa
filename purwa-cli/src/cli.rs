//! Clap CLI definitions.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "empu",
    version,
    about = "Empu — Purwa CLI (Artisan-equivalent)"
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        help = "Print files that would be written and their contents"
    )]
    pub dry_run: bool,
    #[arg(long, global = true, help = "Print extra diagnostics")]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scaffold a new Purwa application
    New(NewArgs),
    /// `cargo run` with RUST_LOG=debug if unset
    Serve(ServeArgs),
    /// `cargo watch -x run` (requires `cargo install cargo-watch`)
    Dev(DevArgs),
    /// `cargo build --release` and optional `frontend/` npm build
    Build(BuildArgs),
    /// Print registered routes (runs `purwa-print-routes` binary)
    #[command(name = "route:list")]
    RouteList(RouteListArgs),
    /// Apply pending SQL migrations
    Migrate(MigrateArgs),
    #[command(name = "migrate:rollback")]
    MigrateRollback(MigrateArgs),
    #[command(name = "migrate:fresh")]
    MigrateFresh(MigrateArgs),
    #[command(name = "make:request")]
    MakeRequest(MakeRequestArgs),
    #[command(name = "make:auth")]
    MakeAuth(MakeAuthArgs),
    #[command(name = "make:controller")]
    MakeController(MakeControllerArgs),
    #[command(name = "make:service")]
    MakeService(MakeServiceArgs),
    #[command(name = "make:model")]
    MakeModel(MakeModelArgs),
    #[command(name = "make:migration")]
    MakeMigration(MakeMigrationArgs),
    #[command(name = "make:seeder")]
    MakeSeeder(DeferredArgs),
    #[command(name = "make:policy")]
    MakePolicy(DeferredArgs),
    #[command(name = "db:seed")]
    DbSeed(DeferredArgs),
    #[command(name = "inertia:setup")]
    InertiaSetup(DeferredArgs),
}

#[derive(Parser)]
pub struct NewArgs {
    /// Package name (kebab-case); prompts if omitted
    pub name: Option<String>,
    /// Create project in this directory (default: ./<name>)
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Enable `purwa` feature `auth` in generated Cargo.toml
    #[arg(long)]
    pub auth: bool,
    /// Enable `purwa` feature `inertia`
    #[arg(long)]
    pub inertia: bool,
    /// Use `path = ...` for the `purwa` dependency instead of crates.io version
    #[arg(long)]
    pub purwa_path: Option<PathBuf>,
    /// Server port in generated `purwa.toml` / `main.rs`
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
    /// Skip interactive prompts (use defaults for unspecified options)
    #[arg(long)]
    pub yes: bool,
}

#[derive(Parser)]
pub struct ServeArgs {
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
    #[arg(long, value_name = "NAME")]
    pub bin: Option<String>,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub cargo_args: Vec<OsString>,
}

#[derive(Parser)]
pub struct DevArgs {
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
    /// Argument to `cargo watch -x`
    #[arg(long, default_value = "run")]
    pub watch_cmd: String,
}

#[derive(Parser)]
pub struct BuildArgs {
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
}

#[derive(Parser)]
pub struct RouteListArgs {
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser)]
pub struct MigrateArgs {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Option<String>,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long, default_value = "database/migrations")]
    pub migrations_dir: PathBuf,
}

#[derive(Parser)]
pub struct MakeRequestArgs {
    pub name: String,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Parser)]
pub struct MakeAuthArgs {
    #[arg(long, default_value = "database/migrations")]
    pub migrations_dir: PathBuf,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Parser)]
pub struct MakeControllerArgs {
    pub name: String,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Parser)]
pub struct MakeServiceArgs {
    pub name: String,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Parser)]
pub struct MakeModelArgs {
    pub name: String,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub sea_orm: bool,
}

#[derive(Parser)]
pub struct MakeMigrationArgs {
    pub name: String,
    #[arg(long, default_value = "database/migrations")]
    pub migrations_dir: PathBuf,
}

#[derive(Parser)]
pub struct DeferredArgs {}
