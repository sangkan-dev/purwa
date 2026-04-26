//! Environment helpers (no optional database dependencies).

/// Reads **`TEST_DATABASE_URL`** if set and non-empty (trimmed).
///
/// Use this for integration tests against a disposable Postgres instance (local or CI) without
/// Docker-based testcontainers. See workspace **README** / **TASK** Q4.
pub fn test_database_url_from_env() -> Option<String> {
    std::env::var("TEST_DATABASE_URL")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}
