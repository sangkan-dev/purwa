# Releasing Purwa (crates.io release train)

Aligned with [TASK.md](./TASK.md) **Resolved Q2**: multiple workspace crates share **the same semver** (e.g. **`0.1.0`** at MVP, or **`0.1.0-alpha.N`** if you need pre-releases).

## Repository URL in manifests

`[workspace.package] repository` in the root **[Cargo.toml](./Cargo.toml)** must match your Git host (default: `https://github.com/sangkan-dev/purwa`). Fix it **before** the first publish if your canonical repo differs.

## One-time: crates.io login

```bash
cargo login
```

Paste an API token from [crates.io/settings/tokens](https://crates.io/settings/tokens). Alternatively set **`CARGO_REGISTRY_TOKEN`** in the environment for CI (keep it secret).

You must **own** (or be an owner of) every crate name on first publish. After the first upload, add co-owners with:

```bash
cargo owner -a <username> -p <crate-name>
```

## Before each publish

1. **Commit** a clean tree (`cargo publish` refuses uncommitted changes unless you pass `--allow-dirty`, which you should avoid for real releases).
2. Run **`cargo publish -p <crate> --dry-run`** for the next crate in the list below; fix any errors.
3. Metadata is already set per crate: `description`, `license`, `repository`, `readme`, `documentation` (docs.rs), `keywords` / `categories` where useful.
4. Internal path dependencies use **`version = "0.1.0"` + `path`** so local workspace builds keep working; the **published** `Cargo.toml` uses only the version.

## v0.1.0 — publish order

Publish **dependencies before dependents**. Wait for each crate to finish indexing on crates.io (usually under a minute) before the next, or the next `--dry-run` / publish may fail.

```bash
# From repository root, after commit:
cargo publish -p purwa-macros
cargo publish -p purwa-core
cargo publish -p purwa-orm
cargo publish -p purwa-inertia
cargo publish -p purwa-auth
cargo publish -p purwa-testing
cargo publish -p purwa
cargo publish -p purwa-cli
```

**Binary:** users install the CLI with:

```bash
cargo install purwa-cli
```

The executable name is **`empu`**.

If a step fails with “no matching package named `purwa-…`”, the dependency was not published or not yet visible — retry after the upstream crate appears on [crates.io](https://crates.io).

## After the train

1. Tag the repo: **`v0.1.0`** (or match your version).
2. Optional: GitHub Release notes, `cargo audit`, and the security checklist in [docs/mvp-checklist.md](./docs/mvp-checklist.md).

## Version bump (future releases)

Bump **`[workspace.package] version`** in the root **`Cargo.toml`** (members use `version.workspace = true`) so all crates stay aligned. Run **`cargo build --workspace`** and commit **`Cargo.lock`** if it changes.

## Dry-run note

`cargo publish -p purwa-orm --dry-run` **before** `purwa-core` exists on crates.io will fail — that is expected. Verify leaf crates (`purwa-macros`, `purwa-core`) anytime; verify the rest **after** their dependencies are live, or rely on a full publish sequence in a clean checkout.

## Binary **`empu`**

**`purwa-cli`** exposes the **`empu`** binary. [docs/getting-started.md](./docs/getting-started.md) documents install paths for workspace and for crates.io.
