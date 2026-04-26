//! Authentication errors.

use thiserror::Error;

/// Password hashing or verification failure.
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("password hashing failed: {0}")]
    Hash(String),
    #[error("invalid password hash in storage")]
    InvalidHash,
    #[error("password verification failed")]
    Verify,
}
