//! Empu — Purwa CLI (full generators land in Sprint 8).

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use heck::ToSnakeCase;
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
    /// Generate a validated request DTO (`{{Name}}Request`) under `src/app/http/requests/`
    #[command(name = "make:request")]
    MakeRequest(MakeRequestArgs),
    /// Generate Argon2id session auth: `users` migration + HTTP handlers (`purwa-auth` / `purwa` auth feature)
    #[command(name = "make:auth")]
    MakeAuth(MakeAuthArgs),
}

#[derive(Parser)]
struct MakeAuthArgs {
    /// Directory for SQL migrations (default: `database/migrations`)
    #[arg(long, default_value = "database/migrations")]
    migrations_dir: PathBuf,
    /// Output module path (default: `src/app/http/auth.rs`)
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Parser)]
struct MakeRequestArgs {
    /// Type name prefix; output struct will be `{Name}Request` (e.g. `CreateUser` → `CreateUserRequest`).
    name: String,
    /// Output file path (default: `src/app/http/requests/{snake_case(name)}.rs`)
    #[arg(long)]
    output: Option<PathBuf>,
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
        Some(Commands::MakeRequest(args)) => {
            make_request_file(args)?;
        }
        Some(Commands::MakeAuth(args)) => {
            make_auth_files(args)?;
        }
        None => {
            eprintln!(
                "empu — Purwa CLI. Try: empu migrate, empu make:request, empu make:auth, empu route:list, …"
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

fn make_request_file(
    args: MakeRequestArgs,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prefix = args.name.trim();
    if prefix.is_empty() {
        return Err("name must not be empty".into());
    }
    let struct_name = format!("{prefix}Request");
    let file_stem = prefix.to_snake_case();
    let path = args
        .output
        .unwrap_or_else(|| PathBuf::from("src/app/http/requests").join(format!("{file_stem}.rs")));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = format!(
        r#"//! Generated by `empu make:request {prefix}` (minimal generator; Sprint 5).
//!
//! Use with [`purwa::ValidatedJson`] or [`purwa::ValidatedForm`] in handlers.
//! Ensure your crate depends on `serde`, `validator`, and `purwa`.

use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct {struct_name} {{
    /// Example field — replace with your DTO.
    #[validate(length(min = 1, message = "is required"))]
    pub title: String,
}}
"#
    );
    std::fs::write(&path, content)?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}

fn make_auth_files(args: MakeAuthArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::fs::create_dir_all(&args.migrations_dir)?;
    let ver = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let prefix = format!("{ver:020}_purwa_users");
    let up_path = args.migrations_dir.join(format!("{prefix}.up.sql"));
    let down_path = args.migrations_dir.join(format!("{prefix}.down.sql"));
    if up_path.exists() || down_path.exists() {
        return Err(format!(
            "migration files already exist for prefix {prefix}; remove them or pick a new timestamp"
        )
        .into());
    }
    std::fs::write(
        &up_path,
        r"-- Generated by `empu make:auth` — Purwa Sprint 7 (Argon2id passwords; PHC in password_hash)

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX users_email_idx ON users (email);
",
    )?;
    std::fs::write(
        &down_path,
        r"DROP INDEX IF EXISTS users_email_idx;
DROP TABLE IF EXISTS users;
",
    )?;
    eprintln!("Wrote {}", up_path.display());
    eprintln!("Wrote {}", down_path.display());

    let out = args
        .output
        .unwrap_or_else(|| PathBuf::from("src/app/http/auth.rs"));
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, AUTH_RS_TEMPLATE)?;
    eprintln!("Wrote {}", out.display());
    eprintln!(
        "Cargo.toml: enable `purwa` feature `auth` and depend on `purwa-auth` with feature `postgres`."
    );
    eprintln!(
        "Merge `auth::router(state)` into your Axum `Router` and apply the same `AuthManagerLayer` stack as in `router()` (session + auth layers)."
    );
    Ok(())
}

const AUTH_RS_TEMPLATE: &str = r#"//! Generated by `empu make:auth` — session login with **Argon2id** (`purwa_auth::hash_password`).
//!
//! **Cargo.toml**
//! - `purwa = { path = "…", features = ["auth"] }` (or crates.io version with `auth`)
//! - `purwa-auth = { path = "…", features = ["postgres"] }`
//! - `serde`, `validator`, `axum`, `sqlx` as in your app
//!
//! **`AppState`** must include a public `pool: sqlx::PgPool` field (adjust imports if your type lives elsewhere).

use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::Router;
use purwa::ValidatedForm;
use purwa_auth::{
    insert_user, AuthManagerLayerBuilder, AuthSession, CurrentUser, EmailPasswordCredential,
    PgAuthnBackend, memory_session_layer,
};
use serde::Deserialize;
use validator::Validate;

/// Replace with your real `AppState` import.
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterForm {
    #[validate(email(message = "invalid email"))]
    pub email: String,
    #[validate(length(min = 8, message = "at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginForm {
    #[validate(email(message = "invalid email"))]
    pub email: String,
    pub password: String,
}

pub fn router(state: AppState) -> Router<AppState> {
    let backend = PgAuthnBackend::new(state.pool.clone());
    let session_layer = memory_session_layer();
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .layer(auth_layer)
        .with_state(state)
}

async fn register(
    State(state): State<AppState>,
    ValidatedForm(form): ValidatedForm<RegisterForm>,
) -> impl IntoResponse {
    match insert_user(&state.pool, &form.email, &form.password).await {
        Ok(_) => Redirect::temporary("/login").into_response(),
        Err(_) => (axum::http::StatusCode::CONFLICT, "email taken or db error").into_response(),
    }
}

async fn login(
    mut auth: AuthSession<PgAuthnBackend>,
    ValidatedForm(form): ValidatedForm<LoginForm>,
) -> impl IntoResponse {
    let creds = EmailPasswordCredential {
        email: form.email,
        password: form.password,
    };
    let user = match auth.authenticate(creds).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (axum::http::StatusCode::UNAUTHORIZED, "invalid credentials").into_response();
        }
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "database error",
            )
                .into_response();
        }
    };
    if auth.login(&user).await.is_err() {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "login failed").into_response();
    }
    Redirect::temporary("/").into_response()
}

async fn logout(mut auth: AuthSession<PgAuthnBackend>) -> impl IntoResponse {
    let _ = auth.logout().await;
    Redirect::temporary("/login").into_response()
}

async fn me(CurrentUser(u): CurrentUser<PgAuthnBackend>) -> impl IntoResponse {
    format!("logged in as {} ({})", u.email, u.id)
}
"#;

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
