//! Optional Postgres helpers behind the **`postgres`** feature.
//!
//! For migrations and pool setup, use **`purwa-orm`** (`connect_pool`, `migrate_up`, …) like
//! `purwa-orm/tests/migrate_integration.rs` — this crate only supplies container lifecycle.
//! For **`TEST_DATABASE_URL`**, see [`crate::test_database_url_from_env`].

use std::future::Future;

use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

/// Starts a Postgres **testcontainers** instance, builds a connection URL, runs `f(url)`, then
/// drops the container.
///
/// Default image credentials match `purwa-orm` integration tests (`postgres` / `postgres`,
/// database `postgres`).
pub async fn with_testcontainer_postgres<F, Fut, T>(f: F) -> T
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = T>,
{
    let container = Postgres::default()
        .start()
        .await
        .expect("start postgres container");
    let host = container.get_host().await.expect("container host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("container port");
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    f(url).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Requires Docker; run with `cargo test -p purwa-testing --features postgres -- --ignored`.
    #[tokio::test]
    #[ignore = "needs Docker (testcontainers)"]
    async fn testcontainer_connects_via_purwa_orm() {
        with_testcontainer_postgres(|url| async move {
            let pool = purwa_orm::connect_pool(&url).await.expect("connect_pool");
            let (n,): (i64,) = sqlx::query_as("SELECT 1::bigint")
                .fetch_one(&pool)
                .await
                .expect("SELECT 1");
            assert_eq!(n, 1);
        })
        .await;
    }
}
