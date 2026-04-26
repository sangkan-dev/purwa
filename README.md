# Purwa

**The fundamental Rust web framework. Forged by Empu.**

Purwa is an opinionated Rust web framework from **Sangkan**: Laravel-class developer experience (conventions, routing, ORM path, auth, CLI scaffolding) on **Axum** and **Tower**, with **Svelte + Inertia** as the default full-stack story.

## Documentation

- [Getting started](./docs/getting-started.md) — ~15 minute first run (`empu new`, `cargo run`, `/health`)
- [Architecture](./docs/architecture.md) — crates, request flow, config, SQLx vs SeaORM
- [Escape hatches](./docs/escape-hatches.md) — raw Axum/Tower, `Router` + state, SQLx
- [MVP checklist](./docs/mvp-checklist.md) — PRD §11 verification pointers
- [PRD.md](./PRD.md) — product requirements and architecture
- [TASK.md](./TASK.md) — sprint plan and acceptance criteria
- [AGENT.md](./AGENT.md) — guidelines for contributors and AI agents
- [CONTRIBUTING.md](./CONTRIBUTING.md) — build, test, PR expectations

## Supported platforms

**Primary:** Linux and macOS. Native Windows development is not a v1 goal (see PRD); paths in tooling may assume Unix.

## Configuration

Settings load from optional **`purwa.toml`** in the working directory and from environment variables prefixed with **`PURWA`**, with nested keys separated by **`__`** (for example `PURWA_SERVER__PORT`). A **`.env`** file is read via [`dotenvy`](https://docs.rs/dotenvy) when present. See [purwa.toml.example](./purwa.toml.example) and [.env.example](./.env.example).

**Database URL:** set `[database].url` in `purwa.toml`, or `PURWA_DATABASE__URL`, or `DATABASE_URL`. Use this for [`PgPool`](https://docs.rs/sqlx/latest/sqlx/type.PgPool.html), [`AppState`](purwa-core/src/lib.rs), and `empu migrate` (Sprint 4).

**Integration tests (TASK Q4 — two layers):**

1. **Without Postgres:** exercise handlers and routing only. Use **[`purwa-testing`](purwa-testing/src/lib.rs)** (`oneshot`, `oneshot_status`, `json_body`, …) with [`router_from_inventory`](purwa-core/src/routing.rs) or a manual `Router<()>`. There is no lightweight mock for `sqlx::PgPool`; keep fast tests free of a real pool (route-only / stub `Extension` types), or use layer (2).
2. **With Postgres:** framework crates (e.g. **`purwa-orm`**) use **testcontainers** where noted (requires Docker). For a fixed instance instead, set **`TEST_DATABASE_URL`** to a disposable database (see [.env.example](./.env.example)). Example migration files: [purwa-orm/tests/fixtures/migrations](./purwa-orm/tests/fixtures/migrations) (copy into your app’s `database/migrations`). Optional: enable **`purwa-testing`** feature **`postgres`** for [`with_testcontainer_postgres`](purwa-testing/src/postgres.rs) — still use **`purwa_orm::connect_pool`** / **`migrate_*`** as the single source of truth for migrations (see [purwa-orm/tests/migrate_integration.rs](./purwa-orm/tests/migrate_integration.rs)).

Scaffolded apps from **`empu new`** include `tests/no_db_smoke.rs` and an ignored `tests/postgres_optional.rs` demonstrating both paths.

Merging **inventory-based routes** ([`router_from_inventory`](purwa-core/src/routing.rs)) with a router that uses **`AppState`** (typed `Router<AppState>`) is a composition detail for your `main` (Sprint 4+ may refine helpers); handlers that need config should use `State<Arc<AppConfig>>` with [`AppState`](purwa-core/src/lib.rs) and Axum `FromRef`.

## Validation (Sprint 5)

Use the [`validator`](https://docs.rs/validator) crate (`#[derive(Validate)]` on request DTOs) with [`ValidatedJson`](purwa-core/src/extract.rs) or [`ValidatedForm`](purwa-core/src/extract.rs). Failed rules return **422** with JSON:

`{ "message": "Validation failed", "errors": { "field_name": ["…"] } }`

Malformed JSON (**400**) uses `{ "message": "…" }`. [`PurwaError`](purwa-core/src/error.rs) implements `IntoResponse` for use in `Result`-returning handlers. Scaffold a DTO with `empu make:request CreateThing` (writes `src/app/http/requests/create_thing.rs` by default).

## Errors & logging (Sprint 10)

[`PurwaError`](purwa-core/src/error.rs) covers validation, malformed JSON/form, **401 / 403 / 404**, **`sqlx::Error`** (row-not-found maps to 404; details are logged, not returned), and generic **500**. Library crates should use **`thiserror`** and surface `PurwaError` at HTTP boundaries; application binaries may use **`anyhow`** in `main` (startup, glue) and convert to `PurwaError` before returning from handlers.

**Inertia:** use [`InertiaRequest::respond_purwa_error`](purwa-inertia/src/request.rs) with the shared **`Error`** page (`INERTIA_ERROR_COMPONENT`, generated as `Pages/Error.svelte`) so X-Inertia JSON and full-page HTML stay aligned.

**Tracing:** call **`purwa::init_tracing()`** once at startup (after `dotenvy::dotenv()`). Uses **pretty** logs by default; set **`PURWA_ENV=production`** for **JSON** lines. Levels follow **`RUST_LOG`** (default `info` via [`init_tracing_with_filter`](purwa-core/src/logging.rs)).

## Inertia.js (Sprint 6)

Enable the adapter with **`purwa = { path = "...", features = ["inertia"] }`**. Crate **[`purwa-inertia`](purwa-inertia/src/lib.rs)** implements protocol **v1.3**: `InertiaRequest` extractor, [`InertiaRenderContext`](purwa-inertia/src/request.rs) + [`InertiaRequest::respond`](purwa-inertia/src/request.rs) (JSON vs HTML first load with optional Vite tags via `html_body_injection`), partial reload headers, **409** on asset version mismatch for GET, and shared props middleware. Set **`[inertia].asset_version`** in `purwa.toml` (or `PURWA_INERTIA__ASSET_VERSION`); **`empu build`** can sync it from `public/.vite/manifest.json` after the Vite build. Use **`empu new --inertia`** or **`empu inertia:setup`** for the `frontend/` template.

## Routing note

Purwa registers HTTP handlers with the [`inventory`](https://docs.rs/inventory) crate (linker sections). That mechanism is **not supported on `wasm32` targets**; use Purwa on native server/desktop targets only for macro-based routing.

## Philosophy

### English

**Purwa** — *the fundamental Rust web framework. Forged by Empu.*

In Javanese cosmology, *Purwa* is the first movement of the **Purwa–Madya–Wasana** cycle: the **beginning**, the right place to start. **Sangkan** (the source) names the organization behind the project. **Empu** is the master smith who tempers raw iron into a *pusaka* — something built to last. The CLI **Empu** is that forge for your app: conventions, generators, and Laravel-class productivity on **Axum** and **Tower**, without hiding the platform when you need it.

The north star: a developer comfortable with **Laravel** should feel productive in **under one day**; the stack stays **idiomatic Rust** (memory safety, async on **Tokio**, escape hatches to Axum and SQLx).

### Bahasa Indonesia

**Purwa** — *kerangka kerja web Rust fundamental. Ditempa oleh Empu.*

Dalam kosmologi Jawa, *Purwa* adalah fase pertama **Purwa–Madya–Wasana**: **permulaan**, titik awal yang benar. **Sangkan** adalah sang sumber — organisasi yang membawa visi ini. **Empu** adalah pandai besi agung yang menempa besi menjadi **pusaka**, perangkat lunak yang tahan lama. CLI **Empu** adalah palu dan dapur tempa untuk aplikasi Anda: konvensi, *scaffolding*, dan pengalaman mirip **Laravel** di atas **Axum** dan **Tower**, tanpa menutup akses ke lapisan bawah ketika Anda membutuhkannya.

Bintang utara: pengembang yang nyaman dengan **Laravel** produktif dalam **kurang dari satu hari**; tetap **Rust yang idiomatis** — aman memori, async **Tokio**, dan jalan keluar ke Axum serta SQLx.

## License

MIT — see [LICENSE](./LICENSE).
