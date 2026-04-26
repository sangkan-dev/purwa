//! Two `#[get]` handlers register only via macros — no manual `Router::route`.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use purwa::{get, router_from_inventory};
use tower::ServiceExt;

#[get("/ping")]
async fn ping() -> &'static str {
    "pong"
}

#[get("/docs")]
async fn docs() -> &'static str {
    "docs"
}

#[tokio::test]
async fn get_handlers_resolve_without_manual_wiring() {
    let app = router_from_inventory();

    let res = app
        .clone()
        .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = app
        .oneshot(Request::builder().uri("/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[test]
fn route_table_lists_both_handlers() {
    let table = purwa::format_route_table();
    assert!(table.contains("/ping"));
    assert!(table.contains("/docs"));
    assert!(table.contains("GET"));
}
