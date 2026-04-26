//! Per-request shared props merged into every Inertia page object.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use purwa_core::AppConfig;
use serde_json::{Map, Value};

/// JSON object merged into page `props` before the handler’s props (handler wins on key collision).
#[derive(Clone, Default, Debug)]
pub struct SharedProps(pub Map<String, Value>);

impl SharedProps {
    pub fn insert(&mut self, key: impl Into<String>, value: Value) {
        self.0.insert(key.into(), value);
    }
}

/// Axum middleware: ensure [`SharedProps`] exists in request extensions (empty map).
///
/// Add **before** handlers that call [`crate::InertiaRequest::respond`]. Populate via a later layer
/// or inside handlers by extracting `Extension<SharedProps>` with `std::mem::take` patterns as needed.
pub async fn ensure_shared_props(mut req: Request<Body>, next: Next) -> Response {
    if req.extensions().get::<SharedProps>().is_none() {
        req.extensions_mut().insert(SharedProps::default());
    }
    next.run(req).await
}

/// Seed shared props with `app.name` from [`AppConfig`] (typical “flash” of app identity).
pub async fn seed_shared_props_from_config(
    State(cfg): State<Arc<AppConfig>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let mut props = SharedProps::default();
    props.insert("app", serde_json::json!({ "name": cfg.app.name.clone() }));
    req.extensions_mut().insert(props);
    next.run(req).await
}
