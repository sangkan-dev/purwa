//! `frontend/` tree for Inertia + Vite + Svelte (Sprint 9).

use std::path::Path;

use askama::Template;

use crate::cli::InertiaSetupArgs;
use crate::generate::{
    FrontendAppJs, FrontendErrorSvelte, FrontendGitignore, FrontendPackageJson,
    FrontendSvelteConfig, FrontendViteConfig, FrontendWelcomeSvelte,
};
use crate::util::{GlobalOpts, write_output};

/// Write the standard `frontend/` layout (idempotent for nested dirs).
pub fn write_frontend_tree(
    root: &Path,
    vite_backend_port: u16,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let files: [(&Path, String); 8] = [
        (&root.join("package.json"), FrontendPackageJson.render()?),
        (
            &root.join("vite.config.js"),
            FrontendViteConfig {
                port: vite_backend_port,
            }
            .render()?,
        ),
        (
            &root.join("svelte.config.js"),
            FrontendSvelteConfig.render()?,
        ),
        (&root.join("src/app.js"), FrontendAppJs.render()?),
        (
            &root.join("src/Pages/Welcome.svelte"),
            FrontendWelcomeSvelte.render()?,
        ),
        (
            &root.join("src/Pages/Error.svelte"),
            FrontendErrorSvelte.render()?,
        ),
        (&root.join("src/Components/.gitkeep"), String::new()),
        (&root.join(".gitignore"), FrontendGitignore.render()?),
    ];
    for (path, content) in files {
        write_output(path, &content, opts)?;
    }
    if !opts.dry_run && !opts.verbose {
        eprintln!("Wrote frontend tree under {}", root.display());
    }
    Ok(())
}

/// `empu inertia:setup` — add or refresh `frontend/` in an existing project.
pub fn run_inertia_setup(
    args: InertiaSetupArgs,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pkg = args.output.join("package.json");
    if pkg.is_file() && !args.force {
        return Err(format!(
            "{} already exists; pass --force to overwrite",
            pkg.display()
        )
        .into());
    }
    write_frontend_tree(&args.output, args.backend_port, opts)?;
    if !opts.dry_run {
        eprintln!(
            "Next: enable `purwa` feature `inertia`, add `tower-http` with feature `fs`, use \
             `ServeDir::new(\"public\")` as a fallback route, add `[inertia]` to purwa.toml, and a \
             `Welcome` route — or run `empu new --inertia` for a full example."
        );
    }
    Ok(())
}
