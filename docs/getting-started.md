# Getting started (~15 minutes)

This guide assumes **Linux** or **macOS** (see [README](../README.md) — Windows is not a v1 target for tooling paths).

## Prerequisites

- **Rust** — stable toolchain, **2024 edition** (see workspace `rust-toolchain.toml` / `rust-version` in root `Cargo.toml` for the Purwa repo).
- **PostgreSQL** — optional for this quick path; you need it only if you follow database or auth steps later.
- **Node.js 20+** — only if you use **Svelte + Inertia** (`empu new --inertia` or add `frontend/`).

## Install Empu (CLI)

From a clone of the Purwa workspace:

```bash
cargo install --path purwa-cli
```

Verify:

```bash
empu --help
```

When crates are published, you will be able to use `cargo install purwa-cli` instead (binary name remains **`empu`**).

## Create a project

```bash
empu new my-app --yes --output my-app
cd my-app
```

Use `--purwa-path /path/to/purwa/purwa` if you depend on a local Purwa checkout instead of crates.io.

## Run the server

```bash
cargo run
```

Or:

```bash
empu serve
```

The scaffold listens on the port in `purwa.toml` (default **8080** unless you chose another with `empu new`).

## Check health

```bash
curl -s http://127.0.0.1:8080/health
```

Expect body `ok` (or your scaffold’s health handler output).

## Run tests (no database)

```bash
cargo test
```

The template includes `tests/no_db_smoke.rs` so routing works without Postgres.

## Inertia + Vite (optional)

If you scaffolded with **`--inertia`**:

1. Install frontend deps: `cd frontend && npm install`
2. Terminal A: `cd frontend && npm run dev`
3. Terminal B: `export PURWA_VITE_DEV_ORIGIN=http://127.0.0.1:5173` then `cargo run`

Open the app URL from `purwa.toml` (same host/port as the Rust server). See [README](../README.md) section **Inertia.js** for asset versioning and production builds (`empu build`).

## API reference

After libraries are on [docs.rs](https://docs.rs), search for `purwa`, `purwa-core`, etc., matching the version in your `Cargo.toml`.

## Next steps

- [Architecture](./architecture.md) — how crates fit together.
- [Escape hatches](./escape-hatches.md) — raw Axum `Router`, SQLx, and composition patterns.
- [PRD](../PRD.md) and [TASK](../TASK.md) — product scope and sprint history.
