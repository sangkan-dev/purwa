//! Handler reads config via `State<Arc<AppConfig>>` and `FromRef<AppState>` (Sprint 3).

use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::{Router, routing::get};
use purwa_core::{AppConfig, AppState};
use tower::ServiceExt;

async fn app_name(State(cfg): State<Arc<AppConfig>>) -> String {
    cfg.app.name.clone()
}

#[tokio::test]
async fn handler_reads_config_key_from_toml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("purwa.toml");
    std::fs::write(
        &path,
        r#"
[app]
name = "from-toml-test"

[server]
host = "127.0.0.1"
port = 4000
"#,
    )
    .unwrap();

    let cfg = AppConfig::load_with_file(Some(path.as_path())).unwrap();
    assert_eq!(cfg.server.port, 4000);

    let state = AppState::new(cfg);
    let app = Router::new()
        .route("/name", get(app_name))
        .with_state(state);

    let res = app
        .oneshot(Request::builder().uri("/name").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), b"from-toml-test");
}
