//! Session stack: [`tower_sessions`] + [`axum_login`].
//!
//! ## Escape hatch
//! - Swap [`MemoryStore`] for a persistent store (e.g. Redis via community crates) by changing
//!   the type passed to [`SessionManagerLayer::new`].
//! - Access the raw session via [`axum_login::AuthSession`] in handlers, or enable
//!   `tower_sessions::Session` through Axum extensions as documented in `tower-sessions`.

pub use axum_login::{
    AuthManagerLayerBuilder, AuthSession, AuthUser, AuthnBackend, AuthzBackend, UserId,
    login_required, permission_required,
};
pub use tower_sessions::{MemoryStore, SessionManagerLayer};

/// Build a [`SessionManagerLayer`] backed by in-memory storage (dev/tests; data lost on restart).
pub fn memory_session_layer() -> SessionManagerLayer<MemoryStore> {
    SessionManagerLayer::new(MemoryStore::default())
}
