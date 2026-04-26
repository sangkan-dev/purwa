# Contributing to Purwa

Thank you for helping improve Purwa and **Empu**. This document is the short path for humans; **[AGENT.md](./AGENT.md)** adds conventions for AI agents and deeper policy.

## Prerequisites

- **Linux** or **macOS** (primary platforms; see [PRD](./PRD.md)).
- **Rust** stable matching the repo’s **`rust-toolchain.toml`** / **`rust-version`**.
- **Docker** optional — needed for some `purwa-orm` integration tests (`testcontainers`).

## Clone and build

```bash
git clone <repository-url>
cd purwa
cargo build --workspace
```

## Run Empu from the workspace

```bash
cargo run -p purwa-cli -- --help
# or after install:
cargo install --path purwa-cli
empu --help
```

## Lint suppressions

Do not add **`#[allow(dead_code)]`** or Clippy allows without maintainer agreement ([AGENT.md](./AGENT.md)). A Sprint 12 audit found **no** `#[allow(...)]` attributes in workspace **`.rs`** sources; keep it that way unless an issue links an exception.

## Quality bar (required before PR)

From the workspace root:

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo clippy -p purwa --features auth -- -D warnings
cargo test --workspace
```

Match existing style in files you touch; avoid drive-by refactors unrelated to your change.

## Dependencies

Prefer **`cargo add`** / **`cargo remove`** (see [AGENT.md](./AGENT.md) §5) so versions and lockfiles stay consistent.

## Pull requests

- Keep changes scoped and described in complete sentences.
- Link issues when applicable.
- If you change user-visible behavior, update **[README](./README.md)** or **[docs/](./docs/)** in the same PR when it matters for adopters.

## Security

Report sensitive issues through the channel your maintainers designate (do not open public issues for undisclosed vulnerabilities). Before release, maintainers follow **[docs/mvp-checklist.md](./docs/mvp-checklist.md)** (security sign-off).

## Publishing crates

Maintainers: see **[RELEASING.md](./RELEASING.md)** for the release train and metadata checklist.
