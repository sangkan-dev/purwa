//! Inertia protocol integration tests (Sprint 6).

use axum::Router;
use axum::body::Body;
use axum::extract::Extension;
use axum::http::header::{self, HOST, HeaderValue};
use axum::http::{Method, Request, StatusCode, Uri};
use axum::middleware::{self, from_fn_with_state};
use axum::routing::get;
use purwa_inertia::headers::{
    X_INERTIA, X_INERTIA_PARTIAL_COMPONENT, X_INERTIA_PARTIAL_DATA, X_INERTIA_VERSION,
};
use purwa_inertia::{
    InertiaRenderContext, InertiaRequest, SharedProps, ensure_shared_props,
    seed_shared_props_from_config,
};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

use purwa_core::AppConfig;

async fn events_page(
    inertia: InertiaRequest,
    Extension(shared): Extension<SharedProps>,
    uri: Uri,
    method: Method,
    headers: axum::http::HeaderMap,
) -> Result<axum::response::Response, std::convert::Infallible> {
    let host = headers.get(HOST).and_then(|h| h.to_str().ok());
    let page = serde_json::json!({
        "events": [1, 2],
        "categories": ["a"],
    });
    let ctx = InertiaRenderContext {
        method: &method,
        request_uri: &uri,
        host_header: host,
        asset_version: "v1",
        html_body_injection: None,
    };
    Ok(inertia
        .respond(&ctx, "Events", page, &shared)
        .unwrap_or_else(|e| {
            axum::response::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(e.to_string()))
                .unwrap()
        }))
}

fn test_router() -> Router {
    Router::new()
        .route("/events", get(events_page))
        // Outer layer runs first: ensure `SharedProps` exists, then test seed.
        .layer(middleware::from_fn(
            |mut req: axum::http::Request<Body>, next: axum::middleware::Next| async move {
                if let Some(props) = req.extensions_mut().get_mut::<SharedProps>() {
                    props.insert("app", serde_json::json!({"name": "test-app"}));
                }
                next.run(req).await
            },
        ))
        .layer(middleware::from_fn(ensure_shared_props))
}

#[tokio::test]
async fn inertia_visit_returns_json_with_vary() {
    let app = test_router();
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("http://127.0.0.1/events")
                .header(HOST, "127.0.0.1")
                .header(header::ACCEPT, "text/html, application/xhtml+xml")
                .header(X_INERTIA, "true")
                .header(X_INERTIA_VERSION, "v1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let ct = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(ct.starts_with("application/json"));
    assert_eq!(
        res.headers().get("x-inertia"),
        Some(&HeaderValue::from_static("true"))
    );
    assert_eq!(
        res.headers().get(header::VARY),
        Some(&HeaderValue::from_static("X-Inertia"))
    );

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["component"], "Events");
    assert_eq!(v["url"], "/events");
    assert_eq!(v["version"], "v1");
    let props = v["props"].as_object().unwrap();
    assert!(props.contains_key("errors"));
    assert_eq!(props["app"]["name"], "test-app");
    assert_eq!(props["events"], serde_json::json!([1, 2]));
}

#[tokio::test]
async fn first_visit_returns_html_with_embedded_page_json() {
    let app = test_router();
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("http://127.0.0.1/events")
                .header(HOST, "127.0.0.1")
                .header(header::ACCEPT, "text/html, application/xhtml+xml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let ct = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(ct.starts_with("text/html"));
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains(r#"data-page="app""#));
    assert!(html.contains("\"component\":\"Events\""));
}

#[tokio::test]
async fn partial_reload_only_requests_listed_props_plus_errors() {
    let app = test_router();
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("http://127.0.0.1/events")
                .header(HOST, "127.0.0.1")
                .header(header::ACCEPT, "text/html, application/xhtml+xml")
                .header(X_INERTIA, "true")
                .header(X_INERTIA_VERSION, "v1")
                .header(X_INERTIA_PARTIAL_COMPONENT, "Events")
                .header(X_INERTIA_PARTIAL_DATA, "events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let props = v["props"].as_object().unwrap();
    assert!(props.contains_key("errors"));
    assert!(props.contains_key("events"));
    assert!(!props.contains_key("categories"));
    assert!(!props.contains_key("app"));
}

#[tokio::test]
async fn asset_version_mismatch_get_returns_409_with_location() {
    let app = test_router();
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("http://127.0.0.1/events")
                .header(HOST, "127.0.0.1")
                .header(X_INERTIA, "true")
                .header(X_INERTIA_VERSION, "stale")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);
    let loc = res.headers().get("x-inertia-location").unwrap();
    assert!(loc.to_str().unwrap().contains("/events"));
}

#[tokio::test]
async fn seed_shared_props_from_config_merges_app_name() {
    let cfg = Arc::new(AppConfig::default());
    let app = Router::new()
        .route("/x", get(events_page))
        .layer(from_fn_with_state(
            cfg.clone(),
            seed_shared_props_from_config,
        ));

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("http://127.0.0.1/x")
                .header(HOST, "127.0.0.1")
                .header(X_INERTIA, "true")
                .header(X_INERTIA_VERSION, "v1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["props"]["app"]["name"], cfg.app.name);
}
