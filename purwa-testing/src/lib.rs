//! Test helpers for Purwa applications and workspace crates.
//!
//! ## Default (no extra features)
//!
//! Use [`http`] helpers to drive an Axum [`Router`](axum::Router) with Tower’s
//! [`ServiceExt::oneshot`](tower::ServiceExt::oneshot) without boilerplate. This fits
//! **inventory** `router_from_inventory()` setups where the router state is `()` (see the
//! `purwa` / `purwa-core` routing docs).
//!
//! **Note:** There is no lightweight official mock for SQLx **`PgPool`**. Fast
//! tests should avoid constructing a real pool—test handlers that only need routing, or use
//! [`Extension`] / stub types—**or** run against a real disposable Postgres (below).
//!
//! ## Feature **`postgres`**
//!
//! Enables **`with_testcontainer_postgres`** (see the **`postgres`** module). **`test_database_url_from_env`**
//! is always available (see [`mod@env`]). **Do not** duplicate migration logic here: connect with
//! **`purwa_orm::connect_pool`** and run **`purwa_orm::migrate_up`** / rollbacks the same way as in
//! **`purwa-orm`** crate integration tests under `purwa-orm/tests/` (e.g. `migrate_integration.rs`).
//!
//! Default `cargo test -p purwa-testing` does **not** start Docker; optional tests that need a
//! container are `#[ignore]`.
//!
//! [`Extension`]: axum::Extension

#![forbid(unsafe_code)]

pub mod env;
pub mod http;

#[cfg(feature = "postgres")]
pub mod postgres;

pub use env::test_database_url_from_env;

#[cfg(feature = "postgres")]
pub use postgres::with_testcontainer_postgres;

pub use http::{
    JsonBodyError, json_body, oneshot, oneshot_body_bytes, oneshot_status,
    oneshot_status_with_method,
};
