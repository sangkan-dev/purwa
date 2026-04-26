# MVP checklist (PRD §11)

This maps [PRD §11 Definition of Done](../PRD.md) to evidence in the repository. Maintainers tick release-time items when they are true for a given version.

| PRD §11 criterion | Evidence / where to verify |
|-------------------|----------------------------|
| `empu new myapp` + serve; cold build &lt; 30s on typical dev hardware | [docs/getting-started.md](./getting-started.md); timing is **environment-dependent** (CPU, disk, `CARGO_TARGET_DIR`) — document your baseline when claiming the metric. |
| Full CRUD + Svelte/Inertia &lt; 2h for a new developer | Walkthrough: Getting started + Inertia two-terminal flow + `#[resource]` / route patterns ([purwa/tests/resource_routes.rs](../purwa/tests/resource_routes.rs)); formal timed audit is product QA. |
| Session auth (register, login, logout, hash) with `empu make:auth` | `empu make:auth` template; [purwa-auth](../purwa-auth/) + integration tests e.g. [session_flow.rs](../purwa-auth/tests/session_flow.rs). |
| SQLx migrations with `empu migrate` | [purwa-orm](../purwa-orm/), CLI migrate commands, [migrate_integration.rs](../purwa-orm/tests/migrate_integration.rs). |
| `cargo clippy -- -D warnings` on generated and library code | CI [.github/workflows/ci.yml](../.github/workflows/ci.yml); [CONTRIBUTING.md](../CONTRIBUTING.md). |
| `purwa-testing` usable without a running database | [purwa-testing](../purwa-testing/src/lib.rs) HTTP helpers; scaffold [no_db_smoke.rs](../purwa-cli/templates/scaffold/tests/no_db_smoke.rs.txt). |
| `empu route:list` — method, path, handler | `empu route:list` / JSON; registry from [routing.rs](../purwa-core/src/routing.rs). |
| Inertia partial reloads (`X-Inertia-Partial-Data` etc.) | [inertia_protocol.rs](../purwa-inertia/tests/inertia_protocol.rs) (`partial_reload_only_requests_listed_props_plus_errors`); [request.rs](../purwa-inertia/src/request.rs). |
| Documentation: Getting Started (~15 min), Architecture, Escape Hatches | [getting-started.md](./getting-started.md), [architecture.md](./architecture.md), [escape-hatches.md](./escape-hatches.md). |
| README: philosophy in **Indonesian and English** | [README.md](../README.md) § Philosophy. |

## Maintainer security sign-off (Sprint 12 “Done when”)

Before tagging an MVP (or any security-sensitive) release, maintainers should explicitly confirm:

1. **Dependencies** — Run **`cargo audit`** (or equivalent policy: Dependabot, advisory DB). Address critical issues or document accepted risk.
2. **Sessions / cookies** — Review defaults in **purwa-auth** (cookie names, `secure`, `same-site`, store choice) for your deployment environment.
3. **Secrets** — No secrets committed; `.env` remains gitignored in templates; production uses real secret management.
4. **Breaking security changes** — Document in release notes if defaults change (cookie flags, session length, etc.).

Record sign-off in your release process (e.g. GitHub Release notes or internal log).
