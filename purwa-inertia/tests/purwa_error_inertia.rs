//! PurwaError + Inertia mapping (Sprint 10).

use axum::Router;
use axum::body::Body;
use axum::extract::Extension;
use axum::http::header::{self, HOST};
use axum::http::{Method, Request, StatusCode, Uri};
use axum::routing::get;
use purwa_core::PurwaError;
use purwa_inertia::headers::{X_INERTIA, X_INERTIA_VERSION};
use purwa_inertia::{
    INERTIA_ERROR_COMPONENT, InertiaRenderContext, InertiaRequest, SharedProps, ensure_shared_props,
};
use serde_json::Value;
use std::borrow::Cow;
use std::convert::Infallible;
use tower::ServiceExt;
use validator::{ValidationError, ValidationErrors};

async fn boom(
    inertia: InertiaRequest,
    Extension(shared): Extension<SharedProps>,
    uri: Uri,
    method: Method,
    headers: axum::http::HeaderMap,
) -> Result<axum::response::Response, Infallible> {
    let host = headers.get(HOST).and_then(|h| h.to_str().ok());
    let ctx = InertiaRenderContext {
        method: &method,
        request_uri: &uri,
        host_header: host,
        asset_version: "v1",
        html_body_injection: None,
    };
    let err = PurwaError::unauthorized("not allowed here");
    Ok(inertia
        .respond_purwa_error(&ctx, err, &shared, INERTIA_ERROR_COMPONENT)
        .unwrap())
}

async fn validation_boom(
    inertia: InertiaRequest,
    Extension(shared): Extension<SharedProps>,
    uri: Uri,
    method: Method,
    headers: axum::http::HeaderMap,
) -> Result<axum::response::Response, Infallible> {
    let host = headers.get(HOST).and_then(|h| h.to_str().ok());
    let ctx = InertiaRenderContext {
        method: &method,
        request_uri: &uri,
        host_header: host,
        asset_version: "v1",
        html_body_injection: None,
    };
    let mut errors = ValidationErrors::new();
    errors.add(
        "email",
        ValidationError::new("email").with_message(Cow::Borrowed("invalid email")),
    );
    let err = PurwaError::Validation(errors);
    Ok(inertia
        .respond_purwa_error(&ctx, err, &shared, INERTIA_ERROR_COMPONENT)
        .unwrap())
}

fn app() -> Router {
    Router::new()
        .route("/boom", get(boom))
        .layer(axum::middleware::from_fn(ensure_shared_props))
}

fn app_validation() -> Router {
    Router::new()
        .route("/v", get(validation_boom))
        .layer(axum::middleware::from_fn(ensure_shared_props))
}

#[tokio::test]
async fn inertia_visit_purwa_error_json_shape() {
    let res = app()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("http://127.0.0.1/boom")
                .header(HOST, "127.0.0.1")
                .header(header::ACCEPT, "application/json")
                .header(X_INERTIA, "true")
                .header(X_INERTIA_VERSION, "v1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers().get("x-inertia"),
        Some(&axum::http::HeaderValue::from_static("true"))
    );
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["component"], "Error");
    assert_eq!(v["props"]["message"], "not allowed here");
    assert_eq!(v["props"]["status"], 401);
    assert!(v["props"]["errors"].is_object());
}

#[tokio::test]
async fn first_visit_purwa_error_html_contains_error_component() {
    let res = app()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("http://127.0.0.1/boom")
                .header(HOST, "127.0.0.1")
                .header(header::ACCEPT, "text/html, application/xhtml+xml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains(r#"data-page="app""#));
    assert!(html.contains("\"component\":\"Error\""));
    assert!(html.contains("not allowed here"));
}

#[tokio::test]
async fn validation_error_inertia_json_includes_field_errors() {
    let res = app_validation()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("http://127.0.0.1/v")
                .header(HOST, "127.0.0.1")
                .header(X_INERTIA, "true")
                .header(X_INERTIA_VERSION, "v1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["component"], "Error");
    let email_errs = v["props"]["errors"]["email"].as_array().unwrap();
    assert!(
        email_errs
            .iter()
            .any(|x| x.as_str() == Some("invalid email"))
    );
}
