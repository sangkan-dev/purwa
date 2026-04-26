//! Purwa — opinionated Rust web framework (Sangkan).
//!
//! Product requirements and architecture: `PRD.md` at the repository root.

#[doc(hidden)]
pub use axum;
#[doc(hidden)]
pub use inventory;

pub mod routing {
    pub use purwa_core::routing::*;
}

pub use purwa_macros::{delete, get, post, put, resource};

pub use purwa_core::{
    AppConfig, AppSection, AppState, AxumRouter, DatabaseSection, InertiaSection, PgPool,
    PurwaConfigError, PurwaError, RegisteredRoute, RouteDescriptor, ServerSection, ValidatedForm,
    ValidatedJson, ValidationErrorBody, app_router, flatten_validation_errors, format_route_table,
    route_descriptors, router_from_inventory,
};

#[cfg(feature = "inertia")]
pub use purwa_inertia;

#[cfg(feature = "sea-orm")]
pub use purwa_orm;
