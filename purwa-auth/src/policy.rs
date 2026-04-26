//! Struct-based authorization stub (PRD §13 #2 — Casbin deferred).
//!
//! Applications can implement [`Policy`] with their own action/resource types and call
//! [`Policy::authorize`] before mutating state.

use std::fmt;

use thiserror::Error;

/// Authorization failure (minimal stub).
#[derive(Debug, Error)]
#[error("forbidden: {0}")]
pub struct AuthzError(pub String);

/// Gate carrying the current subject; extend with resource-specific checks.
pub struct Gate<U> {
    pub user: U,
}

impl<U> Gate<U> {
    pub fn new(user: U) -> Self {
        Self { user }
    }

    /// Stub: always ok. Replace with real rules (roles, ownership, etc.).
    pub fn authorize(&self, _action: &str) -> Result<(), AuthzError> {
        Ok(())
    }
}

/// Optional trait for richer policy objects.
pub trait Policy {
    type User;
    type Action: fmt::Debug + ?Sized;

    fn authorize(&self, user: &Self::User, action: &Self::Action) -> Result<(), AuthzError>;
}
