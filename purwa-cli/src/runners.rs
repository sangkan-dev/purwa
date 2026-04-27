//! `serve`, `dev`, `build`, `route:list`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;
use toml_edit::DocumentMut;

use crate::cli::{
    BuildArgs, DbSeedArgs, DevArgs, QueueCronArgs, QueueWorkArgs, RouteListArgs, ServeArgs,
};

fn cargo_bin() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

fn validate_vite_outdir_public(
    frontend: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = frontend.join("vite.config.js");
    if !cfg.is_file() {
        return Ok(());
    }
    let s = std::fs::read_to_string(&cfg)?;
    if !s.contains("../public") {
        return Err(
            "frontend/vite.config.js must set build.outDir to '../public' (see `empu inertia:setup`)."
                .into(),
        );
    }
    Ok(())
}

fn sync_inertia_asset_version_from_manifest(
    project_root: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest_path = project_root.join("public/.vite/manifest.json");
    let raw = std::fs::read_to_string(&manifest_path)
        .map_err(|_| format!("expected {} after vite build", manifest_path.display()))?;
    let v: Value = serde_json::from_str(&raw)?;
    let file = v
        .get("src/app.js")
        .and_then(|e| e.get("file"))
        .and_then(|x| x.as_str())
        .ok_or("vite manifest missing entry for \"src/app.js\"")?;
    let version = std::path::Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file)
        .to_string();
    let purwa_path = project_root.join("purwa.toml");
    let src = std::fs::read_to_string(&purwa_path)
        .map_err(|e| format!("{}: {}", purwa_path.display(), e))?;
    let mut doc: DocumentMut = src
        .parse()
        .map_err(|e: toml_edit::TomlError| e.to_string())?;
    if doc.get("inertia").is_none() || !doc["inertia"].is_table_like() {
        doc["inertia"] = toml_edit::table();
    }
    doc["inertia"]["asset_version"] = toml_edit::value(version);
    std::fs::write(&purwa_path, doc.to_string())?;
    eprintln!(
        "Updated [inertia].asset_version in {}",
        purwa_path.display()
    );
    Ok(())
}

pub fn run_serve(args: ServeArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut c = Command::new(cargo_bin());
    c.arg("run");
    if let Some(m) = &args.manifest_path {
        c.args([
            "--manifest-path",
            m.to_str().ok_or("manifest path must be UTF-8")?,
        ]);
    }
    if let Some(b) = &args.bin {
        c.args(["--bin", b]);
    }
    for a in &args.cargo_args {
        c.arg(a);
    }
    if std::env::var_os("RUST_LOG").is_none() {
        c.env("RUST_LOG", "debug");
    }
    let st = c.status()?;
    if !st.success() {
        return Err("cargo run failed".into());
    }
    Ok(())
}

pub fn run_dev(args: DevArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let watch_cmd = args.watch_cmd;
    let mut c = Command::new(cargo_bin());
    c.args(["watch", "-x", &watch_cmd]);
    if let Some(m) = &args.manifest_path {
        c.args([
            "--manifest-path",
            m.to_str().ok_or("manifest path must be UTF-8")?,
        ]);
    }
    match c.status() {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => Err("cargo watch failed".into()),
        Err(_) => {
            Err("`cargo watch` failed to start. Install with: cargo install cargo-watch".into())
        }
    }
}

pub fn run_build(args: BuildArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut c = Command::new(cargo_bin());
    c.args(["build", "--release"]);
    if let Some(m) = &args.manifest_path {
        c.args([
            "--manifest-path",
            m.to_str().ok_or("manifest path must be UTF-8")?,
        ]);
    }
    let st = c.status()?;
    if !st.success() {
        return Err("cargo build --release failed".into());
    }
    let root = args
        .manifest_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let frontend = root.join("frontend");
    if frontend.join("package.json").is_file() {
        validate_vite_outdir_public(&frontend)?;
        eprintln!("Running npm ci && npm run build in {}", frontend.display());
        let st = Command::new("sh")
            .arg("-c")
            .arg("npm ci && npm run build")
            .current_dir(&frontend)
            .status()?;
        if !st.success() {
            return Err("frontend build failed".into());
        }
        sync_inertia_asset_version_from_manifest(&root)?;
    }
    Ok(())
}

pub fn run_route_list(args: RouteListArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest = args
        .manifest_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("Cargo.toml"));
    let manifest_s = manifest.to_str().ok_or("manifest path must be UTF-8")?;
    let mut cmd = Command::new(cargo_bin());
    cmd.args([
        "run",
        "--quiet",
        "--bin",
        "purwa-print-routes",
        "--manifest-path",
        manifest_s,
    ]);
    if args.json {
        cmd.arg("--");
        cmd.arg("--json");
    }
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let st = cmd.status()?;
    if !st.success() {
        return Err(
            "route:list failed (is this a Purwa app with `purwa-print-routes` binary?).".into(),
        );
    }
    Ok(())
}

pub fn run_db_seed(args: DbSeedArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cmd = Command::new(cargo_bin());
    cmd.args(["run", "--bin", "seed"]);
    if let Some(m) = &args.manifest_path {
        cmd.args([
            "--manifest-path",
            m.to_str().ok_or("manifest path must be UTF-8")?,
        ]);
    }
    for a in &args.cargo_args {
        cmd.arg(a);
    }
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let st = cmd.status()?;
    if !st.success() {
        return Err("db:seed failed (expected a `seed` bin in this project)".into());
    }
    Ok(())
}

pub fn run_queue_work(args: QueueWorkArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cmd = Command::new(cargo_bin());
    cmd.args(["run", "--bin", "queue-worker"]);
    if let Some(m) = &args.manifest_path {
        cmd.args([
            "--manifest-path",
            m.to_str().ok_or("manifest path must be UTF-8")?,
        ]);
    }
    for a in &args.cargo_args {
        cmd.arg(a);
    }
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let st = cmd.status()?;
    if !st.success() {
        return Err("queue:work failed (expected a `queue-worker` bin in this project)".into());
    }
    Ok(())
}

pub fn run_queue_cron(args: QueueCronArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cmd = Command::new(cargo_bin());
    cmd.args(["run", "--bin", "queue-cron"]);
    if let Some(m) = &args.manifest_path {
        cmd.args([
            "--manifest-path",
            m.to_str().ok_or("manifest path must be UTF-8")?,
        ]);
    }
    for a in &args.cargo_args {
        cmd.arg(a);
    }
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let st = cmd.status()?;
    if !st.success() {
        return Err("queue:cron failed (expected a `queue-cron` bin in this project)".into());
    }
    Ok(())
}
