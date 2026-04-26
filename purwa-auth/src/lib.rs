//! Authentication for Purwa: **Argon2id** passwords, [`tower-sessions`] + [`axum_login`], and stubs
//! for API tokens and struct-based policies.
//!
//! # Escape hatches
//! - **Sessions:** use [`session::memory_session_layer`] for dev; swap the store passed to
//!   [`tower_sessions::SessionManagerLayer::new`] for Redis or another backend. Handlers can use
//!   [`axum_login::AuthSession`] directly for full control.
//! - **Password cost:** tune [`password::DEFAULT_M_COST_KIB`] / [`password::hash_password_with`]
//!   for production vs tests ([`password::hash_password_fast`] is for tests only).
//!
//! [`tower-sessions`]: https://docs.rs/tower-sessions
//! [`axum_login`]: https://docs.rs/axum-login

mod error;
mod extract;
mod password;
mod policy;
mod session;
mod token;

#[cfg(feature = "postgres")]
mod pg;

pub use error::PasswordError;
pub use extract::CurrentUser;
pub use password::{
    DEFAULT_M_COST_KIB, DEFAULT_P_COST, DEFAULT_T_COST, argon2_default, argon2_fast, hash_password,
    hash_password_fast, hash_password_with, verify_password, verify_password_with,
};
pub use policy::{AuthzError, Gate, Policy};
pub use session::{
    AuthManagerLayerBuilder, AuthSession, AuthUser, AuthnBackend, AuthzBackend, MemoryStore,
    SessionManagerLayer, UserId, login_required, memory_session_layer, permission_required,
};
pub use token::{ApiTokenStore, authorization_bearer};

#[cfg(feature = "postgres")]
pub use pg::{EmailPasswordCredential, InsertUserError, PgAuthUser, PgAuthnBackend, insert_user};
