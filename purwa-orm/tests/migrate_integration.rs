//! Postgres integration: migrations against a real database (testcontainers in CI).

use std::path::PathBuf;

use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/migrations")
}

#[tokio::test]
async fn migrate_up_runs_against_postgres() {
    let container = Postgres::default().start().await.expect("start postgres");
    let host = container.get_host().await.expect("host");
    let port = container.get_host_port_ipv4(5432).await.expect("port");

    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let pool = purwa_orm::connect_pool(&url).await.expect("connect");

    purwa_orm::migrate_up(&pool, &fixtures_dir())
        .await
        .expect("migrate up");

    let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM purwa_s4_smoke")
        .fetch_one(&pool)
        .await
        .expect("query smoke table");
    assert_eq!(n, 0);
}

#[tokio::test]
async fn migrate_rollback_reversible_step() {
    let container = Postgres::default().start().await.expect("start postgres");
    let host = container.get_host().await.expect("host");
    let port = container.get_host_port_ipv4(5432).await.expect("port");

    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    let pool = purwa_orm::connect_pool(&url).await.expect("connect");

    purwa_orm::migrate_up(&pool, &fixtures_dir())
        .await
        .expect("migrate up");

    let (before,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'purwa_s4_reversible'",
    )
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(before, 1);

    purwa_orm::migrate_rollback_one(&pool, &fixtures_dir())
        .await
        .expect("rollback");

    let (after,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'purwa_s4_reversible'",
    )
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(after, 0);

    let (smoke,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'purwa_s4_smoke'",
    )
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(
        smoke, 1,
        "simple migration should remain after one rollback step"
    );
}
