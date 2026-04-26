//! Inertia.js **protocol v1.3** adapter for Purwa (first-party; no third-party Inertia crate).
//!
//! # MVP (Sprint 6)
//!
//! - [`InertiaRequest`] extractor: reads `X-Inertia`, version, and partial-reload headers.
//! - [`InertiaRequest::respond`]: returns **JSON** for Inertia visits or an **HTML skeleton** for
//!   the first full-page load (embeds the page object in a `<script type="application/json">`).
//! - **409 Conflict** on **GET** when `X-Inertia-Version` differs from the server asset version
//!   ([`purwa_core::InertiaSection::asset_version`]), with `X-Inertia-Location` set for a full reload.
//! - **Partial reloads**: `X-Inertia-Partial-Component` must match the rendered component; then
//!   `X-Inertia-Partial-Except` wins over `X-Inertia-Partial-Data`; `errors` is always included.
//! - [`SharedProps`] + [`shared::ensure_shared_props`] middleware merge props into every page.
//!
//! Production **Vite + Svelte** wiring is **Sprint 9**; the HTML stub here is intentionally minimal.

pub mod headers;
mod request;
pub mod shared;

pub use request::{InertiaRenderContext, InertiaRequest};
pub use shared::{SharedProps, ensure_shared_props, seed_shared_props_from_config};
