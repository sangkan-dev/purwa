//! `empu make:*` code generation.

mod templates;

use std::path::{Path, PathBuf};

use askama::Template;
use heck::{ToKebabCase, ToPascalCase, ToSnakeCase};
pub use templates::*;

use crate::util::GlobalOpts;
use crate::util::write_output;

const AUTH_RS: &str = include_str!("../../templates/make/auth.rs.txt");
const AUTH_UP_SQL: &str = include_str!("../../templates/make/auth_up.sql");
const AUTH_DOWN_SQL: &str = include_str!("../../templates/make/auth_down.sql");
const SEEDERS_MOD_RS: &str = include_str!("../../templates/scaffold/database_seeders_mod.rs.txt");
const JOB_MOD_RS: &str = r#"//! Application jobs.

#![forbid(unsafe_code)]

// purwa:jobs-mods
// purwa:end-jobs-mods

pub fn register_all(mut builder: purwa_queue::JobRegistryBuilder) -> purwa_queue::JobRegistryBuilder {
    // purwa:jobs-register
    // purwa:end-jobs-register
    builder
}
"#;

pub fn make_request(
    name: &str,
    output: Option<PathBuf>,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prefix = name.trim();
    if prefix.is_empty() {
        return Err("name must not be empty".into());
    }
    let struct_name = format!("{prefix}Request");
    let file_stem = prefix.to_snake_case();
    let tpl = MakeRequestTpl {
        prefix,
        struct_name: &struct_name,
    };
    let content = tpl.render()?;
    let path = output
        .unwrap_or_else(|| PathBuf::from("src/app/http/requests").join(format!("{file_stem}.rs")));
    write_output(&path, &content, opts)?;
    if !opts.dry_run && !opts.verbose {
        eprintln!("Wrote {}", path.display());
    }
    Ok(())
}

pub fn make_auth(
    migrations_dir: &Path,
    output: Option<PathBuf>,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !opts.dry_run {
        std::fs::create_dir_all(migrations_dir)?;
    }
    let ver = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let prefix = format!("{ver:020}_purwa_users");
    let up_path = migrations_dir.join(format!("{prefix}.up.sql"));
    let down_path = migrations_dir.join(format!("{prefix}.down.sql"));
    if !opts.dry_run && (up_path.exists() || down_path.exists()) {
        return Err(format!(
            "migration files already exist for prefix {prefix}; remove them or pick a new timestamp"
        )
        .into());
    }
    write_output(&up_path, AUTH_UP_SQL, opts)?;
    write_output(&down_path, AUTH_DOWN_SQL, opts)?;
    let out = output.unwrap_or_else(|| PathBuf::from("src/app/http/auth.rs"));
    write_output(&out, AUTH_RS, opts)?;
    if !opts.dry_run {
        eprintln!(
            "Cargo.toml: enable `purwa` feature `auth` and depend on `purwa-auth` with feature `postgres`."
        );
        eprintln!(
            "Merge `auth::router(state)` into your Axum `Router` and apply the same `AuthManagerLayer` stack as in `router()` (session + auth layers)."
        );
    }
    Ok(())
}

pub fn make_controller(
    name: &str,
    output: Option<PathBuf>,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name must not be empty".into());
    }
    let pascal = name.to_pascal_case();
    let tpl = MakeControllerTpl { name: &pascal };
    let content = tpl.render()?;
    let stem = name.to_snake_case();
    let path = output
        .unwrap_or_else(|| PathBuf::from("src/app/http/controllers").join(format!("{stem}.rs")));
    write_output(&path, &content, opts)?;
    if !opts.dry_run && !opts.verbose {
        eprintln!("Wrote {}", path.display());
    }
    Ok(())
}

pub fn make_service(
    name: &str,
    output: Option<PathBuf>,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name must not be empty".into());
    }
    let pascal = name.to_pascal_case();
    let trait_name = format!("{pascal}Service");
    let impl_name = format!("{pascal}ServiceImpl");
    let tpl = MakeServiceTpl {
        name: &pascal,
        trait_name: &trait_name,
        impl_name: &impl_name,
    };
    let content = tpl.render()?;
    let stem = name.to_snake_case();
    let path =
        output.unwrap_or_else(|| PathBuf::from("src/app/services").join(format!("{stem}.rs")));
    write_output(&path, &content, opts)?;
    if !opts.dry_run && !opts.verbose {
        eprintln!("Wrote {}", path.display());
    }
    Ok(())
}

pub fn make_model(
    name: &str,
    output: Option<PathBuf>,
    _sea_orm: bool,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name must not be empty".into());
    }
    let pascal = name.to_pascal_case();
    let tpl = MakeModelTpl {
        struct_name: &pascal,
    };
    let content = tpl.render()?;
    let stem = name.to_snake_case();
    let path = output.unwrap_or_else(|| PathBuf::from("src/app/models").join(format!("{stem}.rs")));
    write_output(&path, &content, opts)?;
    if !opts.dry_run && !opts.verbose {
        eprintln!("Wrote {}", path.display());
    }
    if _sea_orm && !opts.dry_run {
        eprintln!(
            "Note: `--sea-orm` entity stub deferred — add SeaORM entity by hand or track Sprint 9+."
        );
    }
    Ok(())
}

pub fn make_seeder(
    name: &str,
    output_dir: Option<PathBuf>,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name must not be empty".into());
    }
    let stem = name.to_snake_case();
    let out_dir = output_dir.unwrap_or_else(|| PathBuf::from("src/database/seeders"));

    let tpl = MakeSeederTpl { name };
    let content = tpl.render()?;
    let file_path = out_dir.join(format!("{stem}.rs"));
    if !opts.dry_run && file_path.exists() {
        return Err(format!("seeder already exists at {}", file_path.display()).into());
    }
    write_output(&file_path, &content, opts)?;

    let mod_path = out_dir.join("mod.rs");
    if !opts.dry_run && !mod_path.exists() {
        write_output(&mod_path, SEEDERS_MOD_RS, opts)?;
    }
    if !opts.dry_run {
        let src = std::fs::read_to_string(&mod_path)
            .map_err(|e| format!("{}: {}", mod_path.display(), e))?;
        let updated = seeders_mod_insert(&src, &stem)?;
        if updated != src {
            std::fs::write(&mod_path, updated)?;
            if opts.verbose {
                eprintln!("Updated {}", mod_path.display());
            }
        }
    }

    if !opts.dry_run && !opts.verbose {
        eprintln!("Wrote {}", file_path.display());
    }
    Ok(())
}

pub fn make_job(
    name: &str,
    output_dir: Option<PathBuf>,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name must not be empty".into());
    }
    let pascal = name.to_pascal_case();
    let stem = name.to_snake_case();
    let out_dir = output_dir.unwrap_or_else(|| PathBuf::from("src/app/jobs"));

    let type_string = stem.to_kebab_case();
    let tpl = MakeJobTpl {
        name: &pascal,
        type_string: &type_string,
    };
    let content = tpl.render()?;
    let file_path = out_dir.join(format!("{stem}.rs"));
    if !opts.dry_run && file_path.exists() {
        return Err(format!("job already exists at {}", file_path.display()).into());
    }
    write_output(&file_path, &content, opts)?;

    let mod_path = out_dir.join("mod.rs");
    if !opts.dry_run && !mod_path.exists() {
        write_output(&mod_path, JOB_MOD_RS, opts)?;
    }
    if !opts.dry_run {
        let src = std::fs::read_to_string(&mod_path)
            .map_err(|e| format!("{}: {}", mod_path.display(), e))?;
        let updated = jobs_mod_insert(&src, &stem)?;
        if updated != src {
            std::fs::write(&mod_path, updated)?;
            if opts.verbose {
                eprintln!("Updated {}", mod_path.display());
            }
        }
    }

    if !opts.dry_run && !opts.verbose {
        eprintln!("Wrote {}", file_path.display());
    }
    Ok(())
}

fn jobs_mod_insert(
    src: &str,
    stem: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    const MOD_BEGIN: &str = "// purwa:jobs-mods";
    const MOD_END: &str = "// purwa:end-jobs-mods";
    const REG_BEGIN: &str = "// purwa:jobs-register";
    const REG_END: &str = "// purwa:end-jobs-register";

    if !(src.contains(MOD_BEGIN)
        && src.contains(MOD_END)
        && src.contains(REG_BEGIN)
        && src.contains(REG_END))
    {
        return Err(format!(
            "jobs mod.rs is missing Purwa markers ({MOD_BEGIN}, {MOD_END}, {REG_BEGIN}, {REG_END})"
        )
        .into());
    }

    let mod_line = format!("pub mod {stem};");
    let reg_line = format!("    builder = {stem}::register(builder);");

    let mut out = src.to_string();
    if !out.contains(&mod_line) {
        out = out.replace(MOD_BEGIN, &format!("{MOD_BEGIN}\n{mod_line}"));
    }
    if !out.contains(&reg_line) {
        out = out.replace(REG_BEGIN, &format!("{REG_BEGIN}\n{reg_line}"));
    }
    Ok(out)
}

fn seeders_mod_insert(
    src: &str,
    stem: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    const MOD_BEGIN: &str = "// purwa:seeders-mods";
    const MOD_END: &str = "// purwa:end-seeders-mods";
    const RUN_BEGIN: &str = "// purwa:seeders-run";
    const RUN_END: &str = "// purwa:end-seeders-run";

    if !(src.contains(MOD_BEGIN)
        && src.contains(MOD_END)
        && src.contains(RUN_BEGIN)
        && src.contains(RUN_END))
    {
        return Err(format!(
            "seeders mod.rs is missing Purwa markers ({MOD_BEGIN}, {MOD_END}, {RUN_BEGIN}, {RUN_END})"
        )
        .into());
    }

    let mod_line = format!("mod {stem};");
    let run_line = format!("    {stem}::run(pool).await?;");

    let mut out = src.to_string();
    if !out.contains(&mod_line) {
        out = out.replace(MOD_BEGIN, &format!("{MOD_BEGIN}\n{mod_line}"));
    }
    if !out.contains(&run_line) {
        out = out.replace(RUN_BEGIN, &format!("{RUN_BEGIN}\n{run_line}"));
    }
    Ok(out)
}

pub fn make_migration(
    name: &str,
    migrations_dir: &Path,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stem = name.trim().to_snake_case();
    if stem.is_empty() {
        return Err("migration name must not be empty".into());
    }
    if !opts.dry_run {
        std::fs::create_dir_all(migrations_dir)?;
    }
    let ver = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let prefix = format!("{ver:020}_{stem}");
    let up = migrations_dir.join(format!("{prefix}.up.sql"));
    let down = migrations_dir.join(format!("{prefix}.down.sql"));
    if !opts.dry_run && (up.exists() || down.exists()) {
        return Err("migration files for this timestamp exist; retry in 1s or remove files".into());
    }
    let up_tpl = MakeMigrationUp { stem: &stem };
    let down_tpl = MakeMigrationDown { stem: &stem };
    write_output(&up, &up_tpl.render()?, opts)?;
    write_output(&down, &down_tpl.render()?, opts)?;
    if !opts.dry_run && !opts.verbose {
        eprintln!("Wrote {}", up.display());
        eprintln!("Wrote {}", down.display());
    }
    Ok(())
}

/// Feature list for `Cargo.toml` e.g. `"default", "auth"` — each item quoted in output.
pub fn features_csv(features: &[&str]) -> String {
    features
        .iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// `purwa` dependency line inner (after `purwa = { `, before `, features` ).
pub fn purwa_dep_toml(purwa_path: Option<&Path>) -> String {
    if let Some(p) = purwa_path {
        format!(r#"path = "{}""#, p.display())
    } else {
        r#"version = "0.1.0""#.to_string()
    }
}

/// `purwa-testing` dev-dependency line inner (after `purwa-testing = { `, before ` }`).
pub fn purwa_testing_dep_toml(purwa_path: Option<&Path>) -> String {
    sibling_crate_dep(purwa_path, "purwa-testing")
}

/// `purwa-orm` dev-dependency line inner for optional DB integration tests.
pub fn purwa_orm_dep_toml(purwa_path: Option<&Path>) -> String {
    sibling_crate_dep(purwa_path, "purwa-orm")
}

/// `purwa-queue` dependency line inner for scaffolded apps.
pub fn purwa_queue_dep_toml(purwa_path: Option<&Path>) -> String {
    sibling_crate_dep(purwa_path, "purwa-queue")
}

fn sibling_crate_dep(purwa_path: Option<&Path>, crate_dir: &str) -> String {
    if let Some(p) = purwa_path
        && let Some(parent) = p.parent()
    {
        let path = parent.join(crate_dir);
        format!(r#"path = "{}""#, path.display())
    } else {
        r#"version = "0.1.0""#.to_string()
    }
}

/// Kebab-case package name from user input.
pub fn crate_package_name(raw: &str) -> String {
    raw.trim().to_kebab_case()
}

/// Rust library identifier (hyphens → underscores).
pub fn rust_lib_name_from_package(pkg: &str) -> String {
    pkg.replace('-', "_")
}
