//! Askama templates (paths relative to `purwa-cli/templates/`).

use askama::Template;

#[derive(Template)]
#[template(path = "make/request.txt", escape = "none")]
pub struct MakeRequestTpl<'a> {
    pub prefix: &'a str,
    pub struct_name: &'a str,
}

#[derive(Template)]
#[template(path = "scaffold/Cargo.toml.txt", escape = "none")]
pub struct ScaffoldCargoToml<'a> {
    pub crate_name: &'a str,
    pub purwa_dep: &'a str,
    pub features_csv: &'a str,
    pub inertia: bool,
}

#[derive(Template)]
#[template(path = "scaffold/lib.rs.txt", escape = "none")]
pub struct ScaffoldLibRs<'a> {
    pub title: &'a str,
    pub inertia: bool,
}

#[derive(Template)]
#[template(path = "scaffold/main.rs.txt", escape = "none")]
pub struct ScaffoldMainRs<'a> {
    pub title: &'a str,
    /// Library crate name (snake_case) for `use {{name}} as _`.
    pub rust_lib_name: &'a str,
    pub port: u16,
    pub inertia: bool,
}

#[derive(Template)]
#[template(path = "scaffold/print_routes.rs.txt", escape = "none")]
pub struct ScaffoldPrintRoutes<'a> {
    pub rust_lib_name: &'a str,
    pub inertia: bool,
}

#[derive(Template)]
#[template(path = "scaffold/routes_health.rs.txt", escape = "none")]
pub struct ScaffoldHealth;

#[derive(Template)]
#[template(path = "scaffold/routes_welcome.rs.txt", escape = "none")]
pub struct ScaffoldWelcome;

#[derive(Template)]
#[template(path = "scaffold/purwa.toml.txt", escape = "none")]
pub struct ScaffoldPurwaToml<'a> {
    pub title: &'a str,
    pub app_name: &'a str,
    pub port: u16,
    pub inertia: bool,
}

#[derive(Template)]
#[template(path = "scaffold/env.example.txt", escape = "none")]
pub struct ScaffoldEnvExample<'a> {
    pub app_name: &'a str,
}

#[derive(Template)]
#[template(path = "scaffold/README.md.txt", escape = "none")]
pub struct ScaffoldReadme<'a> {
    pub title: &'a str,
    pub port: u16,
    pub inertia: bool,
}

#[derive(Template)]
#[template(path = "frontend/package.json.txt", escape = "none")]
pub struct FrontendPackageJson;

#[derive(Template)]
#[template(path = "frontend/vite.config.js.txt", escape = "none")]
pub struct FrontendViteConfig {
    pub port: u16,
}

#[derive(Template)]
#[template(path = "frontend/svelte.config.js.txt", escape = "none")]
pub struct FrontendSvelteConfig;

#[derive(Template)]
#[template(path = "frontend/src/app.js.txt", escape = "none")]
pub struct FrontendAppJs;

#[derive(Template)]
#[template(path = "frontend/src/Pages/Welcome.svelte.txt", escape = "none")]
pub struct FrontendWelcomeSvelte;

#[derive(Template)]
#[template(path = "frontend/.gitignore.txt", escape = "none")]
pub struct FrontendGitignore;

#[derive(Template)]
#[template(path = "make/controller.txt", escape = "none")]
pub struct MakeControllerTpl<'a> {
    pub name: &'a str,
}

#[derive(Template)]
#[template(path = "make/service.txt", escape = "none")]
pub struct MakeServiceTpl<'a> {
    pub name: &'a str,
    pub trait_name: &'a str,
    pub impl_name: &'a str,
}

#[derive(Template)]
#[template(path = "make/model.txt", escape = "none")]
pub struct MakeModelTpl<'a> {
    pub struct_name: &'a str,
}

#[derive(Template)]
#[template(path = "make/migration_up.txt", escape = "none")]
pub struct MakeMigrationUp<'a> {
    pub stem: &'a str,
}

#[derive(Template)]
#[template(path = "make/migration_down.txt", escape = "none")]
pub struct MakeMigrationDown<'a> {
    pub stem: &'a str,
}
