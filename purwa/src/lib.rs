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
    AxumRouter, RegisteredRoute, RouteDescriptor, app_router, format_route_table,
    route_descriptors, router_from_inventory,
};
