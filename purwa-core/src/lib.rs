//! Purwa core — HTTP kernel built on Axum (see workspace `purwa` facade).
//!
//! Applications can take full control of routing by using the [`AxumRouter`] type
//! and Tower services directly (escape hatch; PRD §3.1).

pub mod config;
pub mod routing;

use std::sync::Arc;

use axum::extract::FromRef;
use axum::{Router, routing::get};

pub use config::{AppConfig, AppSection, PurwaConfigError, ServerSection};
pub use routing::{
    RegisteredRoute, RouteDescriptor, format_route_table, route_descriptors, router_from_inventory,
};

/// Shared application state (PRD §5.3). Holds `Arc` resources only — no globals.
///
/// Sprint 4 will add `db: Arc<PgPool>` (or equivalent) alongside [`config`](AppConfig).
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
}

impl FromRef<AppState> for Arc<AppConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

/// Type alias for the default Axum router with unit state — use this when composing custom routes.
pub type AxumRouter = Router;

/// Default application router: Sprint 1 hello on `/` plus any routes registered via Purwa macros
/// ([`router_from_inventory`]).
pub fn app_router() -> AxumRouter {
    Router::new()
        .route("/", get(|| async { "Hello, Purwa" }))
        .merge(router_from_inventory())
}

#[cfg(test)]
mod tests {
    use super::app_router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn root_returns_200() {
        let app = app_router();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
