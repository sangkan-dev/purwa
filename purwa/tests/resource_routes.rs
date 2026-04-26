//! `#[resource]` registers seven logical routes (Axum merges methods per path).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use purwa::{resource, router_from_inventory};
use tower::ServiceExt;

#[resource("/widgets")]
mod widgets {
    use axum::extract::Path;

    pub async fn index() -> &'static str {
        "index"
    }
    pub async fn create() -> &'static str {
        "create"
    }
    pub async fn store() -> &'static str {
        "store"
    }
    pub async fn show(Path(_id): Path<String>) -> &'static str {
        "show"
    }
    pub async fn edit(Path(_id): Path<String>) -> &'static str {
        "edit"
    }
    pub async fn update(Path(_id): Path<String>) -> &'static str {
        "update"
    }
    pub async fn destroy(Path(_id): Path<String>) -> &'static str {
        "destroy"
    }
}

#[tokio::test]
async fn resource_index_get() {
    let app = router_from_inventory();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/widgets")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn resource_show_with_id() {
    let app = router_from_inventory();
    let res = app
        .oneshot(
            Request::builder()
                .uri("/widgets/42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[test]
fn resource_route_table_has_seven_rows() {
    let n = purwa::route_descriptors().count();
    assert_eq!(n, 7);
}
