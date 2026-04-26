# Purwa — Instructions for AI Agents & Contributors

This repository is **Purwa**: an opinionated Rust web framework (Sangkan) with **Empu** CLI, targeting Laravel-class DX on **Axum / Tower**, **SQLx** (default) + optional **SeaORM**, and **Svelte + Inertia** as the blessed full-stack path.

---

## 1. Sources of truth (read before changing scope)

| Document | Use |
|----------|-----|
| [PRD.md](./PRD.md) | Product scope, architecture, stack, non-goals, Definition of Done (MVP). |
| [TASK.md](./TASK.md) | Sprint order (S1–S12), acceptance criteria, resolved decisions (license, crates.io, `#[resource]`, testing). |
| [docs/getting-started.md](./docs/getting-started.md), [docs/architecture.md](./docs/architecture.md), [docs/escape-hatches.md](./docs/escape-hatches.md) | User-facing quickstart, architecture, escape hatches. |
| [CONTRIBUTING.md](./CONTRIBUTING.md), [RELEASING.md](./RELEASING.md), [docs/mvp-checklist.md](./docs/mvp-checklist.md) | Contributor workflow, crates.io train, PRD §11 evidence + security sign-off. |

**Do not** expand scope into PRD **§3.2 Non-Goals** or **§7.2 Post-MVP** unless the user explicitly updates the PRD/TASK.

---

## 2. North star & constraints

- **DX:** A developer comfortable with Laravel should become productive in **&lt; 1 day**; full CRUD + Svelte/Inertia in **&lt; 2 hours** with docs (PRD §11).
- **Rust:** No **`unsafe`** in **application-facing** APIs; prefer **idiomatic** patterns over “magic.”
- **DI:** Use **Axum `State` / `FromRef` / `Extension`** — no separate compile-time DI compiler (PRD §4.2.1).
- **Routing registration:** **`inventory`** — document **no WASM** for this path (PRD §10).
- **Inertia:** **First-party `purwa-inertia`** — do not depend on unmaintained third-party Inertia crates for the core path.
- **ORM:** **SQLx default**; **SeaORM** only behind **`purwa` feature flag** (PRD §4.2.3).
- **Dev workflow:** **`empu dev`** = fast rebuild/watch — **never** call it “hot reload” (PRD §4.2.4).
- **License:** **MIT**; **crates.io:** multi-crate **aligned semver**, first release at MVP (**`purwa`** name verified free).

---

## 3. Sprint discipline

1. Follow **[TASK.md](./TASK.md)** sprint **dependencies** (S1 → … → S12). Do not skip foundations (e.g. routing before DB) without explicit maintainer approval.
2. **`#[resource]`** is **MVP-required** — full REST set, not a stub (TASK § Resolved Q3).
3. Prefer **small, reviewable changes** that map to one sprint slice or one logical task group.
4. Update **TASK.md** checkboxes when work is **merged and verified**, not when merely drafted.

---

## 4. Code quality (non-negotiable)

- Run **`cargo fmt`** and **`cargo clippy -- -D warnings`** on touched crates; **no clippy suppressions** in generated or hand-written code without a linked issue and maintainer agreement (PRD §11).
- **No dead code** in `main`, examples, or public API: remove unused items or gate behind **`#[cfg(test)]` / features** with a short comment referencing an issue if temporarily needed.
- **No redundant helpers:** one implementation per concern — put shared logic in the **owning crate** (`purwa-core`, `purwa-inertia`, etc.), re-export from **`purwa`** facade only when intentional.
- **Escape hatches:** Expose underlying **Axum `Router`**, **SQLx**, **SeaORM** where PRD promises — do not hide platform types unnecessarily.
- Match **existing** naming, module layout, and error style in the file you edit.

---

## 5. Dependencies & manifests (use CLI, not hand-edits)

Adding or removing packages should go through the **tooling** so versions, features, and **lockfiles** stay consistent. **Do not** paste new deps into `Cargo.toml` / `package.json` by hand unless there is no alternative (then run the matching sync command and verify locks).

### Rust (workspace)

- **Add:** `cargo add <crate>` from the crate directory, or **`cargo add -p <workspace-member> <crate>`** from the workspace root (e.g. `cargo add -p purwa-core serde`).
- **Features / optional deps:** `cargo add serde --features derive`, `cargo add sqlx --optional`, etc. (see `cargo add --help`).
- **Dev-only:** `cargo add --dev <crate>` (or `--dev -p mycrate ...`).
- **Remove:** `cargo remove <crate>` / `cargo remove -p <member> <crate>`.
- **Lockfile:** Let **`cargo`** update **`Cargo.lock`** — do not hand-edit `Cargo.lock`.
- **Manual `Cargo.toml` edits** are only for non-dependency concerns (e.g. `[workspace.package]`, metadata, `[[bin]]` names). After any dependency change, run **`cargo check`** (or `cargo test`) on affected crates.

### Frontend (`frontend/` or scaffold JS/TS)

- Use the **package manager the project already uses** (check `package.json` for `packageManager` or existing lockfile: `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`).
- **Add:** `npm install <pkg>`, **`pnpm add <pkg>`**, or **`yarn add <pkg>`** — not manual `"dependencies": { }` edits.
- **Do not** hand-edit **lockfiles** (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`); the install command must refresh them.

### Other ecosystems

- If you introduce tools with their own manifests (e.g. **pnpm** for Vite, **cargo** for WASM helpers), apply the same rule: **use the ecosystem’s add/install command**, then commit the updated lockfile.

---

## 6. Testing strategy (TASK Q4)

| Layer | Expectation |
|-------|-------------|
| **Framework / DB integration** | **Real Postgres** in CI via **testcontainers** (or CI service); exercises migrations and SQLx (`purwa-orm` tests are the reference). |
| **`purwa-testing` / app tests** | **No-DB path:** HTTP helpers + `Router<()>` / inventory routes; avoid requiring **`PgPool`** in the same test when you want speed — there is no official lightweight pool mock. **DB path:** **`TEST_DATABASE_URL`** or **`purwa-testing`** feature **`postgres`** (testcontainers helper) + **`purwa_orm::connect_pool`** / **`migrate_*`**; do not duplicate migration logic in `purwa-testing`. |
| **Local** | Document **`TEST_DATABASE_URL`** for contributors without Docker. |

Do not replace integration tests with mocks only, or vice versa — **both** layers are required.

---

## 7. Stack reference (MVP)

Align versions with **PRD §9** (Axum 0.8, Tokio 1.x, SQLx 0.8, syn 2, `inventory`, `tower-sessions`, `axum-login`, etc.). When in doubt, **PRD wins**; if PRD is stale, propose a PRD edit in the same change as the dependency bump.

---

## 8. Frontend (Svelte + Inertia)

When the repo contains **`frontend/`**:

- Prefer **Svelte** patterns consistent with the scaffold; keep **Inertia** protocol behavior in **`purwa-inertia`**, not duplicated in app JS.
- Do not introduce **React/Vue** in the default scaffold (PRD §13 #3 — community adapters later).

Use **Svelte MCP / project Svelte skills** when editing **`.svelte`** files if available in the agent environment.

---

## 9. Verification before claiming “done”

1. **`cargo test`** (workspace or affected crates).
2. **`cargo clippy --workspace -- -D warnings`** (or scoped to changed crates if CI matches).
3. **`cargo fmt --check`**.
4. For DB-related changes: confirm **CI or local** path covers **Postgres** integration where applicable.

Do not assert completion without running these or citing CI evidence.

---

## 10. CLI & naming

- Binary crate: **`purwa-cli`**, command name: **`empu`** (PRD §8).
- Generators should support **`--verbose`** / clear output (PRD risk register — “magic” perception).

---

## 11. If something is ambiguous

1. **PRD.md** → **TASK.md** → existing code.
2. If still unclear, **document the assumption** in the PR description or a short comment near the code; do not invent large features.

---

*Purwa · Empu · Sangkan — agent instructions aligned with PRD & TASK.*
