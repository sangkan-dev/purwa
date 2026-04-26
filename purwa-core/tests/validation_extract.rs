//! ValidatedJson / PurwaError response shape (Sprint 5).

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use purwa_core::ValidatedJson;
use serde::Deserialize;
use tower::ServiceExt;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
struct SignUpDto {
    #[validate(email(message = "must be a valid email"))]
    email: String,
}

async fn signup(ValidatedJson(body): ValidatedJson<SignUpDto>) -> &'static str {
    let _ = body;
    "ok"
}

#[tokio::test]
async fn validation_failure_is_422_with_errors_map() {
    let app = Router::new().route("/signup", post(signup));
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"not-an-email"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["message"], "Validation failed");
    let email_errs = v["errors"]["email"].as_array().expect("email errors");
    assert!(!email_errs.is_empty());
}

#[tokio::test]
async fn valid_json_passes_validation() {
    let app = Router::new().route("/signup", post(signup));
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"a@b.co"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn malformed_json_is_400() {
    let app = Router::new().route("/signup", post(signup));
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .body(Body::from("{not json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

use axum::routing::get;
use purwa_core::ValidatedForm;

#[derive(Debug, Deserialize, Validate)]
struct SearchDto {
    #[validate(length(min = 1))]
    q: String,
}

async fn search(ValidatedForm(q): ValidatedForm<SearchDto>) -> &'static str {
    let _ = q;
    "ok"
}

#[tokio::test]
async fn validated_form_query_validation_failure_422() {
    let app = Router::new().route("/search", get(search));
    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/search?q=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
