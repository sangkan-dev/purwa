//! `serve`, `dev`, `build`, `route:list`.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::cli::{BuildArgs, DevArgs, RouteListArgs, ServeArgs};

fn cargo_bin() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
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
        eprintln!("Running npm ci && npm run build in {}", frontend.display());
        let st = Command::new("sh")
            .arg("-c")
            .arg("npm ci && npm run build")
            .current_dir(&frontend)
            .status()?;
        if !st.success() {
            return Err("frontend build failed".into());
        }
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
