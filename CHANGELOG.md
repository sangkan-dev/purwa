# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Workspace crates (`purwa`, `purwa-core`, `purwa-macros`, `purwa-inertia`, `purwa-orm`, `purwa-auth`, `purwa-testing`, `purwa-cli`) share the **same version** for each release (see [RELEASING.md](./RELEASING.md)).

## [Unreleased]

## [0.1.1] - 2026-04-27

### Added

- `empu make:seeder <Name>` generator for `database/seeders/` (with deterministic markers in `mod.rs`).
- `empu db:seed` wrapper (runs `cargo run --bin seed`), plus scaffolded `src/bin/seed.rs`.
- PRD §8.1 command table now includes a **Status** column (Available vs Planned).

## [0.1.0] - 2026-04-26

First public **MVP** on [crates.io](https://crates.io): aligned **0.1.0** release train and **Empu** (`purwa-cli`).

### Added

- **Core:** Axum/Tower app kernel (`purwa-core`), `purwa.toml` + env config, `AppState` / `PgPool`, validation (`ValidatedJson` / `ValidatedForm`), unified `PurwaError` + tracing helpers.
- **Routing:** `#[get]` / `#[post]` / `#[put]` / `#[delete]` / `#[resource]`, `inventory` registry, `router_from_inventory()`, `empu route:list`.
- **Data:** `purwa-orm` — SQLx pool, migrations, `empu migrate`; optional SeaORM feature.
- **Full-stack:** `purwa-inertia` (Inertia protocol v1.3-oriented), Svelte + Vite scaffold, `empu build` / asset versioning.
- **Auth:** `purwa-auth` — sessions, `axum-login`, Argon2id, `#[auth]`, `empu make:auth`.
- **CLI:** `empu` — `new`, `serve`, `dev`, `build`, generators (`make:request`, `make:controller`, …), migrations.
- **Testing:** `purwa-testing` — HTTP one-shot helpers; optional Postgres / `TEST_DATABASE_URL`; scaffold integration examples.
- **Docs:** `docs/getting-started.md`, `architecture.md`, `escape-hatches.md`, `mvp-checklist.md`; `CONTRIBUTING.md`, bilingual README philosophy (ID/EN).

### Published crates

`purwa-macros`, `purwa-core`, `purwa-orm`, `purwa-inertia`, `purwa-auth`, `purwa-testing`, `purwa`, `purwa-cli` at **v0.1.0**.

[Unreleased]: https://github.com/sangkan-dev/purwa/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/sangkan-dev/purwa/releases/tag/v0.1.1
[0.1.0]: https://github.com/sangkan-dev/purwa/releases/tag/v0.1.0
