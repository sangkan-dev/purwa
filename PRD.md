# PURWA — Product Requirements Document

> **The Fundamental Rust Web Framework. Forged by Empu.**

| Field | Value |
|---|---|
| Document Type | Product Requirements Document (PRD) |
| Framework Name | Purwa |
| CLI Tool | Empu |
| Organization | Sangkan |
| Status | Draft v1.0 |
| Target Milestone | MVP (Sprint 1–12) |

> **ꦥꦸꦂꦮ — Philosophy**
>
> Dalam kosmologi Jawa, *Purwa* (dari Purwa–Madya–Wasana) berarti Permulaan atau Awal — resonan langsung dengan "Sangkan" (asal muasal). Membangun dengan Purwa berarti kembali ke akar arsitektur yang benar. CLI tool *Empu* — sang ahli tempa keris tingkat tinggi dari tradisi Jawa — menumbuk besi (Rust) menjadi pusaka (software yang efisien dan bertenaga). Bersama-sama mereka mewujudkan prinsip: mulai dengan benar, tempa dengan baik, bertahan selamanya.

---

## 1. Executive Summary

Purwa is an open-source, opinionated Rust web framework built by the Sangkan organization. It delivers Laravel-class developer experience — convention-over-configuration, expressive routing, ORM, authentication, CLI scaffolding — while preserving Rust's memory safety, fearless concurrency, and near-zero runtime overhead.

The primary target is the full-stack freelance developer and small product team shipping CRUD-heavy web applications, admin dashboards, and SaaS products.

**North-star metric:** A Laravel developer becomes productive in Purwa within one working day.

**Slogan:** *Purwa: The fundamental Rust web framework. Forged by Empu.*

---

## 2. Problem Statement

### 2.1 The Rust Web Framework Gap

Rust's async web ecosystem (Axum, Actix-Web, Warp) provides excellent primitives but minimal convention. Developers must manually compose routing, validation, ORM integration, authentication, and DI — work that Laravel, Rails, or Django handle out-of-the-box. This creates a steep productivity cliff for developers migrating from high-level frameworks.

### 2.2 Existing Solutions Are Insufficient

- **Loco.rs** — offers convention but has limited ergonomics and no first-class Inertia/SSR path.
- **Rocket** — attribute macros but not built on the Tower ecosystem; async limitations.
- **Actix-Web** — performant but deliberately low-level.

No existing framework combines: Tower ecosystem compatibility + Inertia.js-native rendering + Laravel-parity DX.

---

## 3. Goals & Non-Goals

### 3.1 Goals

- **Productivity parity** — A Laravel developer can ship a CRUD app with auth in < 1 day.
- **Zero unsafe user code** — The framework never exposes `unsafe` Rust to application developers.
- **Inertia.js first-class** — Svelte + Inertia is the blessed full-stack path, zero configuration.
- **Single binary** — Applications compile to a single static binary with no runtime dependencies.
- **Escape hatches** — Every high-level abstraction exposes the underlying Axum/SeaORM/SQLx API.

### 3.2 Non-Goals (v1.0)

- GraphQL or gRPC support (HTTP/REST only for MVP).
- Multi-language support (Rust only; no WASM guest plugins in MVP).
- Real-time WebSocket framework (basic Axum support only; opinionated layer deferred).
- Replacing or wrapping Tokio internals.
- Windows native development environment (Linux/macOS primary).

---

## 4. Technical Feasibility Review & Corrections

> *This section validates and corrects the brainstorm document against idiomatic Rust and current crate state (2025).*

### 4.1 Foundation Stack — VALIDATED

| Component | Decision | Rationale |
|---|---|---|
| HTTP Runtime | Axum 0.8 + Tokio 1.x | Tower ecosystem; type-safe extractors; broad community adoption. |
| Macro System | proc-macro2 + syn 2 + quote | Industry standard. syn 2.x required for accurate span diagnostics. |
| Async Executor | Tokio 1.x | De facto standard. Do not abstract over it — tie to Tokio explicitly. |
| Serialization | serde + serde_json | No alternative needed. Inertia props auto-serialized. |

### 4.2 Corrections to Brainstorm

#### 4.2.1 Dependency Injection Model

The brainstorm proposes a "compile-time DI container" inspired by Pavex and shaku. This is partially non-idiomatic:

- Pavex's compile-time DI requires a separate compiler binary — too complex for a framework crate.
- `shaku` uses runtime reflection-like patterns that conflict with Axum's extractor model.

**✅ Correction:** Use Axum's native `State<T>` and `Extension<T>` extractors as the DI mechanism. Services are `Arc`-wrapped and injected via `FromRef` trait bounds. Request-scoped context is passed via Axum extensions in Tower middleware. This is idiomatic, zero-overhead, and requires no separate container crate.

#### 4.2.2 Attribute Macro Routing

The `#[get("/path")]` pattern is feasible but compile times are a real concern. "Auto-registration" is not trivially achievable with proc-macros alone — proc-macros cannot collect across crate boundaries without a build script or `inventory`-style crate.

**✅ Correction:** Use the `inventory` crate for handler auto-registration. Each `#[get]` macro submits the handler to a global registry via linker sections. A `router!()` macro (or build-script codegen) collects them at startup. This is the approach used by Rocket 0.5+ internally.

#### 4.2.3 SeaORM vs. SQLx Choice

SeaORM 1.x (2025) is production-stable, but its dynamic query builder involves runtime schema reflection that conflicts with strict compile-time safety goals. SQLx with compile-time `query!` macros provides stronger guarantees.

**✅ Correction:** Offer a dual-tier model. The `purwa-orm` layer wraps SQLx for raw/compile-time queries (default, safe). SeaORM integration is available as an optional feature flag (`purwa = { features = ["sea-orm"] }`) for developers who prefer an active-record style.

#### 4.2.4 "Hot Reload" Terminology

**✅ Correction:** Rust does not support true hot reload of compiled code. `empu dev` should use `cargo-watch` to trigger recompilation and restart on file change (< 2s incremental builds for typical app code). Set expectations clearly in documentation. Do **not** call it "hot reload" — call it **"fast rebuild watch mode"**.

#### 4.2.5 Inertia.js Adapter — Crate Landscape

The brainstorm references `inertia-rust` and `ferro-inertia`. As of 2025, neither is actively maintained. The `axum-inertia` crate is the most current option but incomplete.

**✅ Decision:** Purwa ships its own first-party `purwa-inertia` crate implementing the Inertia protocol v1.3 spec: `X-Inertia` header detection, partial reload via `X-Inertia-Partial-Data`, shared props via middleware, asset versioning, and SSR fallback. (~600 lines — small but critical enough to own.)

#### 4.2.6 `empu tinker` REPL

**⚠️ Feasibility Note:** A REPL with live app context is aspirational for v1.0. Rust does not have an interpreted mode. The practical implementation is a Tokio-async test harness that spawns the app with a special interactive CLI layer. **Deferred to post-v1.**

---

## 5. Architecture

### 5.1 Crate Workspace Structure

```
purwa/                         # Workspace root
├── purwa/                     # Re-export facade crate (what users import)
├── purwa-core/                # Router, macros, DI, middleware kernel
├── purwa-inertia/             # First-party Inertia.js adapter
├── purwa-orm/                 # SQLx wrapper + SeaORM bridge (feature-gated)
├── purwa-auth/                # Session + token auth, policy engine
├── purwa-queue/               # Redis-backed job queue (post-MVP)
├── purwa-cli/                 # empu binary (clap derive)
└── purwa-testing/             # Test utilities, mock extractors
```

### 5.2 Request Lifecycle

```
Incoming HTTP Request
  │
  ▼
Tower Middleware Stack
  (CORS → Rate Limit → Session → Auth Guard)
  │
  ▼
Axum Router
  (route match → handler fn resolved)
  │
  ▼
Extractor Resolution
  (State<AppState>, Inertia, ValidatedForm<T>, CurrentUser)
  │
  ▼
Controller fn
  (delegates to Arc<dyn ServiceTrait>)
  │
  ▼
Service Layer
  (business logic → Repository / ORM)
  │
  ▼
Response
  (Inertia::render() | Json() | Redirect)
```

### 5.3 Application State Model

A single `AppState` struct — cloned into all handlers via Axum `State<>` — holds all shared resources:

```rust
pub struct AppState {
    pub db:     Arc<SqlxPool>,
    pub config: Arc<AppConfig>,
    pub cache:  Arc<MokaCache<String, Value>>,
}
```

Services are constructed in the service provider layer and stored as `Arc<dyn Trait>` inside `AppState` or injected via the `#[inject]` attribute macro on handler params. **No global mutable state. No `lazy_static` singletons in user-facing code.**

---

## 6. Application Directory Structure

> Generated by: `empu new myapp`

```
myapp/
├── Cargo.toml                  # Workspace root + purwa dep
├── Cargo.lock
├── .env.example
├── purwa.toml                  # Framework configuration
├── src/
│   ├── main.rs                 # Bootstrap: state, router, serve
│   └── app/
│       ├── mod.rs
│       ├── controllers/        # Thin HTTP handlers
│       ├── services/           # Business logic (pure Rust)
│       ├── models/             # SQLx row types / SeaORM entities
│       ├── repositories/       # DB access layer
│       ├── http/
│       │   ├── middleware/     # Tower layers
│       │   └── requests/       # Validated form request structs
│       ├── policies/           # Authorization rules
│       └── providers/          # Service provider bootstrapping
├── config/                     # Typed config modules
├── database/
│   ├── migrations/             # SQL migration files
│   ├── seeders/
│   └── factories/
├── routes/                     # Optional explicit route files
├── frontend/                   # Svelte/Vite + Inertia client
│   ├── src/Pages/              # Inertia page components (.svelte)
│   ├── src/Components/
│   └── vite.config.js
├── public/                     # Built frontend assets
├── storage/                    # Logs, cache (gitignored)
└── tests/                      # Integration & unit tests
```

---

## 7. Feature Roadmap

### 7.1 MVP — Phase 1 (Months 1–6)

> Target: A freelance developer ships an authenticated CRUD dashboard with Svelte frontend in < 1 day.

| Area | Feature | Key Crates |
|---|---|---|
| Routing | `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[resource]` macros; auto-registration via `inventory` | axum 0.8, inventory |
| Controllers | Thin handler fns with extractor-based DI; `empu make:controller` | axum extractors |
| Services | `Arc<dyn Trait>` injection; `empu make:service` | std::sync::Arc |
| Database | SQLx async query wrapper; migration runner; `empu migrate` | sqlx 0.8 |
| ORM (opt-in) | SeaORM 1.x via feature flag; `empu make:model` | sea-orm 1.x |
| Validation | `#[derive(Validated)]` on form request structs; async validators | validator 0.18 |
| Auth | Session-based + API token; `#[auth]` guard extractor; `empu make:auth` | tower-sessions, axum-login |
| Inertia.js | `purwa-inertia`: `Inertia::render()`, shared props middleware, asset versioning | first-party crate |
| Config | `purwa.toml` + `.env`; typed `Config::get()`; dotenvy at startup | dotenvy, config |
| CLI | `empu new`, `make:*`, `migrate`, `serve`, `route:list` | clap 4 derive |
| Error Handling | `PurwaError` enum; `IntoResponse` impl; pretty Inertia error pages | thiserror, anyhow |
| Logging | tracing subscriber; structured JSON (prod) + pretty (dev) output | tracing, tracing-subscriber |
| Testing | `purwa-testing`: test app builder, mock state, DB fixtures | first-party crate |

### 7.2 Post-MVP — Phase 2 (Months 7–12)

- **Queue & Jobs** — Redis-backed (`deadpool-redis`); `#[job]` macro; `empu make:job`. Retry logic, scheduled jobs via cron syntax.
- **Mail & Notifications** — SMTP driver (`lettre` crate); Svelte email templates via Inertia SSR.
- **File Storage** — Local/S3 abstraction via `object_store` crate. `empu make:disk`.
- **Caching Facade** — `Cache::get/put/remember` with Moka (in-memory) and Redis backends.
- **Events & Listeners** — Tokio broadcast channel-based event bus; `#[listen]` macro.
- **API Resources** — Typed JSON transformer structs; pagination (cursor + offset); API versioning prefix.
- **Observability** — OpenTelemetry trace export; Prometheus metrics endpoint.
- **REPL / Tinker** — `empu tinker` — async Tokio shell with app context (post-v1 aspirational).

---

## 8. CLI Tool: Empu

> *"Sang Empu menumbuk besi (Rust) menjadi pusaka (software) yang efisien dan mematikan."*

Empu is the Artisan equivalent for Purwa. It is a separate binary crate (`purwa-cli`) distributed as `cargo install purwa-cli`. All subcommands use `clap 4` derive API with `inquire` for interactive prompts.

### 8.1 Command Reference (MVP)

| Command | Description |
|---|---|
| `empu new <name>` | Scaffold full-stack project with Svelte + Inertia boilerplate. |
| `empu serve` | `cargo run` wrapper with `RUST_LOG=debug` and watch-mode hints. |
| `empu dev` | Fast-rebuild watch mode using `cargo-watch`. NOT hot-reload. |
| `empu build` | Release build + Vite frontend build. |
| `empu route:list` | Pretty table of all registered routes (via inventory registry). |
| `empu make:controller <Name>` | Generate controller with optional `--resource` flag. |
| `empu make:service <Name>` | Generate service trait + impl with DI boilerplate. |
| `empu make:model <Name>` | Generate SQLx row type + optional SeaORM entity. |
| `empu make:request <Name>` | Generate validated form request struct. |
| `empu make:migration <name>` | Generate timestamped SQL migration file. |
| `empu make:seeder <Name>` | Generate database seeder. |
| `empu make:policy <Name>` | Generate authorization policy struct. |
| `empu migrate` | Run pending migrations. |
| `empu migrate:rollback` | Rollback last migration batch. |
| `empu migrate:fresh` | Drop all tables and re-run all migrations. |
| `empu db:seed` | Run database seeders. |
| `empu inertia:setup` | Bootstrap Svelte + Inertia.js frontend. |

---

## 9. Dependency Matrix

| Purpose | Crate (version) | Notes |
|---|---|---|
| HTTP server | axum 0.8 | Do not abstract router — expose Axum `Router` directly. |
| Async runtime | tokio 1.x (full features) | Pinned to Tokio. No runtime-agnostic abstraction. |
| Middleware | tower + tower-http | CORS, compression, tracing middleware from tower-http. |
| Proc-macros | proc-macro2, syn 2, quote | syn 2.x required for accurate diagnostics. |
| Route registration | inventory 0.3 | Linker-section trick for distributed handler collection. |
| Database (core) | sqlx 0.8 (postgres feature) | Compile-time `query!` macros. `PgPool` as `AppState` field. |
| Database (opt-in) | sea-orm 1.x | Feature flag: `purwa = { features = ["sea-orm"] }` |
| Validation | validator 0.18 | `#[derive(Validate)]` on request structs. |
| Serialization | serde 1, serde_json 1 | Automatic for all Inertia props. |
| Auth sessions | tower-sessions 0.x | Cookie-backed sessions via Tower layer. |
| Auth login | axum-login 0.x | User session management layer. |
| Config/Env | dotenvy 0.15, config 0.14 | `.env` loading + typed TOML config. |
| CLI | clap 4 (derive) | All `empu` subcommands. |
| Interactive prompts | inquire 0.7 | `empu new` wizard, confirmations. |
| Error handling | thiserror 1, anyhow 1 | `thiserror` for library; `anyhow` for application layer. |
| Logging/Tracing | tracing + tracing-subscriber | JSON (prod) + pretty (dev) subscriber configs. |
| Cache (in-proc) | moka 0.12 | Thread-safe; async-aware. For session-level caching. |
| Template (CLI gen) | askama 0.12 | Type-safe templates for `empu` code generation. |
| Terminal output | colored 2 | `empu` colorful terminal output. |

---

## 10. Technical Constraints & Risk Register

| Risk | Likelihood | Mitigation |
|---|---|---|
| Proc-macro compile time creep | High | Keep macros minimal; document incremental build workflow; provide cargo feature flags to disable macros. |
| `inventory` crate portability | Medium | `inventory` uses linker sections; does not work on WASM targets. Document this. Desktop/server only. |
| Inertia protocol drift (v2) | Low–Medium | Own the `purwa-inertia` crate; monitor inertiajs/inertia changelog. Semver-gate protocol version. |
| SeaORM API churn | Medium | SeaORM integration is feature-gated and isolated in `purwa-orm`. API changes do not affect core. |
| `axum-login` maintenance | Medium | Auth is isolated in `purwa-auth`. If `axum-login` stalls, swap to `tower-sessions` + custom session store. |
| "Magic" perception | High | Provide `--verbose` mode on all `empu` generators; document macro expansion; provide escape-hatch guide. |
| Windows developer support | Low | Not a v1 goal. `cargo` and `sqlx` work on Windows but `empu` templates assume Unix paths. Document. |

---

## 11. Definition of Done (MVP)

The MVP milestone is complete when **all** of the following criteria are satisfied:

- [ ] `empu new myapp` produces a compiling project with `empu serve` working in < 30 seconds on a cold cargo build.
- [ ] A developer can implement a full CRUD resource (list, create, edit, delete) with Svelte frontend via Inertia in < 2 hours with zero prior Purwa experience.
- [ ] Session-based authentication (register, login, logout, password hash) works out of the box with `empu make:auth`.
- [ ] SQLx migrations run cleanly with `empu migrate`.
- [ ] All generated code passes `cargo clippy -- -D warnings` with no suppression.
- [ ] `purwa-testing` crate provides a test app builder usable without a running database (mock pool).
- [ ] `empu route:list` outputs all registered routes with method, path, and handler name.
- [ ] Inertia partial reloads work correctly (`X-Inertia-Partial-Data` header handling).
- [ ] Documentation covers: Getting Started (15 min tutorial), Architecture Overview, Escape Hatches guide.
- [ ] README includes the project philosophy (Purwa–Empu cosmology) in both Indonesian and English.

---

## 12. Initial Sprint Plan (Months 1–3)

| Sprint | Goal | Key Deliverables |
|---|---|---|
| S1 | Workspace & Foundations | Cargo workspace; CI pipeline; `purwa-core` skeleton; Axum hello-world compiles. |
| S2 | Routing Macros | `#[get/post/put/delete]` proc-macros; inventory registration; `empu route:list`. |
| S3 | Config & State | `purwa.toml` parser; `AppState`; dotenvy loader; typed `Config::get()`. |
| S4 | Database Layer | SQLx `PgPool` integration; `purwa migrate` CLI; `query!` macro helper. |
| S5 | Validation | `#[derive(Validated)]`; `ValidatedForm<T>` extractor; error response format. |
| S6 | Inertia Adapter | `purwa-inertia` crate; `Inertia::render()`; shared props middleware. |
| S7 | Authentication | `tower-sessions` setup; `axum-login` integration; `#[auth]` guard extractor. |
| S8 | Empu CLI Core | `empu new` scaffold; `make:controller`, `make:service`, `make:model` generators. |
| S9 | Frontend Pipeline | Vite + Svelte boilerplate; `empu inertia:setup`; asset versioning. |
| S10 | Error Handling & Logging | `PurwaError`; pretty Inertia error page; tracing subscriber configs. |
| S11 | Testing Crate | `purwa-testing`; mock extractors; DB fixture helpers; integration test examples. |
| S12 | MVP Polish & Docs | Getting Started guide; API docs; `CONTRIBUTING.md`; public release. |

---

## 13. Open Questions

| # | Question | Recommendation |
|---|---|---|
| 1 | **ORM default** — SQLx only, or SeaORM enabled by default? | SQLx default; SeaORM as opt-in feature flag. |
| 2 | **Policy engine** — Simple closures or Policy struct with Casbin? | Struct-based policies first; Casbin integration deferred. |
| 3 | **Frontend adapter** — Svelte-only or adapter-agnostic Inertia? | Svelte as blessed path; React/Vue as community adapters. |
| 4 | **License** — MIT vs. Apache-2.0 vs. dual? | **MIT** (open-source default for Purwa workspace). |
| 5 | **Crate publishing** — Publish `purwa-*` on crates.io immediately or hold until v1.0? | **Multi-crate release at MVP:** publish `purwa`, workspace libraries, and `purwa-cli` (`empu`) with **aligned semver** (e.g. `0.1.0` at MVP); optional `0.1.0-alpha.N` pre-releases; do not defer publication until 1.0. See TASK.md § Resolved decisions Q2. |

---

## Appendix A: Naming & Cultural Reference

| Term | Meaning & Context |
|---|---|
| **Purwa** | Beginning/Origin in Javanese. First of the trilogy Purwa–Madya–Wasana. Represents the correct starting point of any system. |
| **Empu** | Master keris-forger in Javanese tradition. Works in the *besalen* (forge). Transforms raw iron into weapons of power. |
| **Sangkan** | *Asal muasal* — the source, the origin. The umbrella organization whose tools always return to first principles. |
| **Pusaka** | Heirloom weapon of power. Metaphor for software forged to last. |
| **Besalen** | The forge workshop. Can be used as a term for the development environment. |
| **Cantrik** | The Empu's apprentice in wayang. Sister project — the Sangkan CLI AI agent. |

---

> **Final Note to Engineering**
>
> Every technical decision in Purwa should pass two tests:
> 1. Would a Laravel developer find this intuitive?
> 2. Would a Rustacean find this idiomatic?
>
> If both answer yes — ship it. If they conflict — favor Rust idioms and document the difference clearly.

---

*— ꦥꦸꦂꦮ · Forged by Empu · Sangkan Organization —*