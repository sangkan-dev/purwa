//! Minimal API-token path: bearer header parsing + async store trait.
//!
//! Wire [`ApiTokenStore`] in your handler with [`axum::extract::State`]: parse the token with
//! [`authorization_bearer`], then call [`ApiTokenStore::resolve_token`].

use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;

/// Resolve a raw secret token to a user identifier (e.g. database lookup of a hashed token).
pub trait ApiTokenStore: Send + Sync + 'static {
    type UserId: Clone + Send + Sync + 'static;
    type Error: std::fmt::Display + Send + Sync + 'static;

    fn resolve_token(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<Option<Self::UserId>, Self::Error>> + Send;
}

/// Return the bearer token from `Authorization: Bearer <token>`, if present and well-formed.
pub fn authorization_bearer(headers: &HeaderMap) -> Option<&str> {
    let hdr = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let prefix = "Bearer ";
    let rest = hdr.strip_prefix(prefix)?;
    let token = rest.trim();
    if token.is_empty() {
        return None;
    }
    Some(token)
}
