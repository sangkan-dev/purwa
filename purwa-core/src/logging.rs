//! Tracing subscriber bootstrap (Sprint 10).
//!
//! Call [`init_tracing`] once at process startup (e.g. in `main` before `tokio::runtime` work).

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

/// Install the global tracing subscriber (pretty in development, JSON when `PURWA_ENV=production`).
///
/// Log levels follow `RUST_LOG` when set; otherwise `default_filter` (e.g. `"info"`).
///
/// # Panics
///
/// Panics if a global subscriber was already set (call at most once).
pub fn init_tracing_with_filter(default_filter: &str) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let is_prod = std::env::var("PURWA_ENV")
        .map(|v| v.eq_ignore_ascii_case("production"))
        .unwrap_or(false);

    if is_prod {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().pretty())
            .init();
    }
}

/// Same as [`init_tracing_with_filter`] with default `"info"`.
pub fn init_tracing() {
    init_tracing_with_filter("info");
}
