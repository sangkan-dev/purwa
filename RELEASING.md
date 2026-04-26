# Releasing Purwa (crates.io release train)

Aligned with [TASK.md](./TASK.md) **Resolved Q2**: multiple workspace crates share **the same semver** (e.g. **`0.1.0`** at MVP, or **`0.1.0-alpha.N`** if you need pre-releases).

## Before the first publish

1. Add **Cargo metadata** to each publishable `Cargo.toml`:
   - `description`, `license` (MIT), `repository`, `readme` (often `"../README.md"` or a crate-specific README), `documentation` (e.g. `https://docs.rs/<crate>/<version>` after first publish, or repo URL until then).
2. Ensure **`publish = false`** is **not** set unless the crate is intentionally private.
3. Confirm **`purwa`** names on [crates.io](https://crates.io) (see TASK **crates.io naming**).

## Publish order

Publish **dependencies before dependents**. A workable order for this workspace:

1. **`purwa-macros`** — proc-macros; no internal path deps on other `purwa-*` crates.
2. **`purwa-core`**
3. **`purwa-inertia`**, **`purwa-orm`**, **`purwa-auth`**, **`purwa-testing`** (any order among these once their deps are on crates.io).
4. **`purwa`** (facade) — update **`purwa-macros`** (and other path deps) to **version-only** entries matching what you just published, e.g. `purwa-macros = "0.1.0"`.
5. **`purwa-cli`** — depends on **`purwa-core`**, **`purwa-orm`**; set those to published versions before publishing the CLI.

After publishing **`purwa`**, run **`cargo publish -p purwa --dry-run`** and a real publish only when dry-run succeeds.

## Commands (per crate)

From the repo root:

```bash
cargo publish -p <crate-name> --dry-run
cargo publish -p <crate-name>
```

Use **`--allow-dirty`** only if your policy explicitly allows it (usually avoid).

## Version bump

Bump **`[workspace.package] version`** in the root **`Cargo.toml`** (members use `version.workspace = true`) so all crates stay aligned. Commit the lockfile changes **`Cargo.lock`** if any.

## Tagging

Tag the repo after a successful train, e.g. **`v0.1.0`**, and record notes (changelog or GitHub Release) as your project prefers.

## Binary **`empu`**

**`purwa-cli`** exposes the **`empu`** binary. Users install with:

```bash
cargo install purwa-cli
```

Document this in [docs/getting-started.md](./docs/getting-started.md) once the crate is live.
