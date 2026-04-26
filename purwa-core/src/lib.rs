//! Purwa core — HTTP kernel built on Axum (see workspace `purwa` facade).
//!
//! Applications can take full control of routing by using the [`AxumRouter`] type
//! and Tower services directly (escape hatch; PRD §3.1).

pub mod config;
pub mod error;
pub mod extract;
pub mod routing;

use std::sync::Arc;

use axum::extract::FromRef;
use axum::{Router, routing::get};

pub use config::{
    AppConfig, AppSection, DatabaseSection, InertiaSection, PurwaConfigError, ServerSection,
};
pub use error::{PurwaError, ValidationErrorBody, flatten_validation_errors};
pub use extract::{ValidatedForm, ValidatedJson};
pub use routing::{
    RegisteredRoute, RouteDescriptor, format_route_table, route_descriptors, router_from_inventory,
};
pub use sqlx::PgPool;

/// Shared application state (PRD §5.3). Holds `Arc` resources only — no globals.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Arc<PgPool>,
}

impl AppState {
    pub fn new(config: Arc<AppConfig>, db: Arc<PgPool>) -> Self {
        Self { config, db }
    }
}

impl FromRef<AppState> for Arc<AppConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db.as_ref().clone()
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
