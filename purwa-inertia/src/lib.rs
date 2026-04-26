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
//! **Vite + Svelte:** use [`vite_manifest`] for full-page `<script>` / `<link>` tags and
//! [`InertiaRenderContext::html_body_injection`](crate::InertiaRenderContext::html_body_injection) in [`InertiaRequest::respond`](crate::InertiaRequest::respond).
//!
//! **Errors (Sprint 10):** [`InertiaRequest::respond_purwa_error`](crate::InertiaRequest::respond_purwa_error) renders [`purwa_core::PurwaError`] as the shared [`INERTIA_ERROR_COMPONENT`] page (e.g. `Pages/Error.svelte`).

pub mod headers;
mod request;
pub mod shared;
pub mod vite_manifest;

pub use request::{INERTIA_ERROR_COMPONENT, InertiaRenderContext, InertiaRequest};
pub use shared::{SharedProps, ensure_shared_props, seed_shared_props_from_config};
