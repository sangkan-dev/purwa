//! `empu new` — project scaffold.

use std::path::PathBuf;

use askama::Template;
use heck::ToTitleCase;
use inquire::Confirm;

use crate::cli::NewArgs;
use crate::frontend::write_frontend_tree;
use crate::generate::{
    ScaffoldCargoToml, ScaffoldEnvExample, ScaffoldHealth, ScaffoldLibRs, ScaffoldMainRs,
    ScaffoldPrintRoutes, ScaffoldPurwaToml, ScaffoldReadme, ScaffoldWelcome, crate_package_name,
    features_csv, purwa_dep_toml, rust_lib_name_from_package,
};
use crate::util::{GlobalOpts, write_output};

pub fn run_new(
    args: NewArgs,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut name = args.name;
    if name.as_deref().map(str::trim).unwrap_or("").is_empty() && !args.yes {
        name = Some(inquire::Text::new("Project name (kebab-case)?").prompt()?);
    }
    let name = name.ok_or("project name required (pass as arg or use prompt)")?;
    let name = name.trim();
    if name.is_empty() {
        return Err("project name must not be empty".into());
    }

    let pkg = crate_package_name(name);
    let lib = rust_lib_name_from_package(&pkg);
    let root = args.output.clone().unwrap_or_else(|| PathBuf::from(&pkg));

    let mut auth = args.auth;
    let mut inertia = args.inertia;
    let mut purwa_path = args.purwa_path.clone();
    let port = args.port;

    if !args.yes {
        if !args.auth
            && Confirm::new("Enable auth feature (purwa/auth)?")
                .with_default(false)
                .prompt()?
        {
            auth = true;
        }
        if !args.inertia
            && Confirm::new("Enable inertia feature (purwa/inertia)?")
                .with_default(false)
                .prompt()?
        {
            inertia = true;
        }
        if purwa_path.is_none()
            && Confirm::new("Use local path for `purwa` dependency (instead of crates.io 0.1.0)?")
                .with_default(false)
                .prompt()?
        {
            let p: String =
                inquire::Text::new("Path to purwa facade crate (directory containing Cargo.toml)?")
                    .with_default("../purwa/purwa")
                    .prompt()?;
            purwa_path = Some(PathBuf::from(p.trim()));
        }
    }

    let mut feats: Vec<&str> = Vec::new();
    if auth {
        feats.push("auth");
    }
    if inertia {
        feats.push("inertia");
    }
    let features_csv = features_csv(&feats);
    let dep_inner = purwa_dep_toml(purwa_path.as_deref());

    let title = pkg.to_title_case();
    let app_name = lib.clone();

    let cargo = ScaffoldCargoToml {
        crate_name: &pkg,
        purwa_dep: &dep_inner,
        features_csv: &features_csv,
        inertia,
    }
    .render()?;
    let lib_rs = ScaffoldLibRs {
        title: &title,
        inertia,
    }
    .render()?;
    let main_rs = ScaffoldMainRs {
        title: &title,
        rust_lib_name: &lib,
        port,
        inertia,
    }
    .render()?;
    let print_rs = ScaffoldPrintRoutes {
        rust_lib_name: &lib,
        inertia,
    }
    .render()?;
    let health_rs = ScaffoldHealth.render()?;
    let purwa_toml = ScaffoldPurwaToml {
        title: &title,
        app_name: &app_name,
        port,
        inertia,
    }
    .render()?;
    let env_ex = ScaffoldEnvExample {
        app_name: &app_name,
    }
    .render()?;
    let readme = ScaffoldReadme {
        title: &title,
        port,
        inertia,
    }
    .render()?;

    let welcome_rs = if inertia {
        Some(ScaffoldWelcome.render()?)
    } else {
        None
    };

    let mut paths: Vec<(PathBuf, String)> = vec![
        (root.join("Cargo.toml"), cargo),
        (root.join("src/lib.rs"), lib_rs),
        (root.join("src/main.rs"), main_rs),
        (root.join("src/bin/purwa-print-routes.rs"), print_rs),
        (root.join("src/routes/health.rs"), health_rs),
        (root.join("purwa.toml"), purwa_toml),
        (root.join(".env.example"), env_ex),
        (root.join("README.md"), readme),
    ];
    if let Some(w) = welcome_rs {
        paths.push((root.join("src/routes/welcome.rs"), w));
        paths.push((root.join("public/.gitkeep"), String::new()));
    }

    for (path, content) in paths {
        write_output(&path, &content, opts)?;
    }

    let gitkeep = root.join("database/migrations/.gitkeep");
    write_output(&gitkeep, "", opts)?;

    if inertia {
        write_frontend_tree(&root.join("frontend"), port, opts)?;
    }

    if !opts.dry_run {
        eprintln!("Created Purwa app at {}", root.display());
        eprintln!("Next: cd {} && cargo build", root.display());
    }

    Ok(())
}
