# Purwa

**The fundamental Rust web framework. Forged by Empu.**

Purwa is an opinionated Rust web framework from **Sangkan**: Laravel-class developer experience (conventions, routing, ORM path, auth, CLI scaffolding) on **Axum** and **Tower**, with **Svelte + Inertia** as the default full-stack story.

## Documentation

- [PRD.md](./PRD.md) — product requirements and architecture
- [TASK.md](./TASK.md) — sprint plan and acceptance criteria
- [AGENT.md](./AGENT.md) — guidelines for contributors and AI agents

## Supported platforms

**Primary:** Linux and macOS. Native Windows development is not a v1 goal (see PRD); paths in tooling may assume Unix.

## Configuration

Settings load from optional **`purwa.toml`** in the working directory and from environment variables prefixed with **`PURWA`**, with nested keys separated by **`__`** (for example `PURWA_SERVER__PORT`). A **`.env`** file is read via [`dotenvy`](https://docs.rs/dotenvy) when present. See [purwa.toml.example](./purwa.toml.example) and [.env.example](./.env.example).

**Database URL:** set `[database].url` in `purwa.toml`, or `PURWA_DATABASE__URL`, or `DATABASE_URL`. Use this for [`PgPool`](https://docs.rs/sqlx/latest/sqlx/type.PgPool.html), [`AppState`](purwa-core/src/lib.rs), and `empu migrate` (Sprint 4).

**Integration tests:** framework crates use **testcontainers** where noted (requires Docker). To run Postgres-backed tests against a fixed instance instead, set **`TEST_DATABASE_URL`** to a disposable database (see [.env.example](./.env.example)). Example migration files: [purwa-orm/tests/fixtures/migrations](./purwa-orm/tests/fixtures/migrations) (copy into your app’s `database/migrations`).

Merging **inventory-based routes** ([`router_from_inventory`](purwa-core/src/routing.rs)) with a router that uses **`AppState`** (typed `Router<AppState>`) is a composition detail for your `main` (Sprint 4+ may refine helpers); handlers that need config should use `State<Arc<AppConfig>>` with [`AppState`](purwa-core/src/lib.rs) and Axum `FromRef`.

## Validation (Sprint 5)

Use the [`validator`](https://docs.rs/validator) crate (`#[derive(Validate)]` on request DTOs) with [`ValidatedJson`](purwa-core/src/extract.rs) or [`ValidatedForm`](purwa-core/src/extract.rs). Failed rules return **422** with JSON:

`{ "message": "Validation failed", "errors": { "field_name": ["…"] } }`

Malformed JSON (**400**) uses `{ "message": "…" }`. [`PurwaError`](purwa-core/src/error.rs) implements `IntoResponse` for use in `Result`-returning handlers. Scaffold a DTO with `empu make:request CreateThing` (writes `src/app/http/requests/create_thing.rs` by default).

## Inertia.js (Sprint 6)

Enable the adapter with **`purwa = { path = "...", features = ["inertia"] }`**. Crate **[`purwa-inertia`](purwa-inertia/src/lib.rs)** implements protocol **v1.3**: `InertiaRequest` extractor, [`InertiaRenderContext`](purwa-inertia/src/request.rs) + [`InertiaRequest::respond`](purwa-inertia/src/request.rs) (JSON vs HTML first load with optional Vite tags via `html_body_injection`), partial reload headers, **409** on asset version mismatch for GET, and shared props middleware. Set **`[inertia].asset_version`** in `purwa.toml` (or `PURWA_INERTIA__ASSET_VERSION`); **`empu build`** can sync it from `public/.vite/manifest.json` after the Vite build. Use **`empu new --inertia`** or **`empu inertia:setup`** for the `frontend/` template.

## Routing note

Purwa registers HTTP handlers with the [`inventory`](https://docs.rs/inventory) crate (linker sections). That mechanism is **not supported on `wasm32` targets**; use Purwa on native server/desktop targets only for macro-based routing.

## Philosophy (summary)

*Purwa* (Javanese: beginning / origin) and *Empu* (master forger) express the goal: start from sound architecture and ship durable software. A fuller bilingual note will land with the MVP docs (Sprint 12).

## License

MIT — see [LICENSE](./LICENSE).
