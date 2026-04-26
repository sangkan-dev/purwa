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

Merging **inventory-based routes** ([`router_from_inventory`](purwa-core/src/routing.rs)) with a router that uses **`AppState`** (typed `Router<AppState>`) is a composition detail for your `main` (Sprint 4+ may refine helpers); handlers that need config should use `State<Arc<AppConfig>>` with [`AppState`](purwa-core/src/lib.rs) and Axum `FromRef`.

## Routing note

Purwa registers HTTP handlers with the [`inventory`](https://docs.rs/inventory) crate (linker sections). That mechanism is **not supported on `wasm32` targets**; use Purwa on native server/desktop targets only for macro-based routing.

## Philosophy (summary)

*Purwa* (Javanese: beginning / origin) and *Empu* (master forger) express the goal: start from sound architecture and ship durable software. A fuller bilingual note will land with the MVP docs (Sprint 12).

## License

MIT — see [LICENSE](./LICENSE).
