//! Extractors built on [`axum_login::AuthSession`].

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_login::{AuthSession, AuthnBackend};
use purwa_core::PurwaError;

/// Authenticated user: fails with **401** if the session has no user.
///
/// Use with routes that already use [`crate::session::AuthManagerLayerBuilder`] / auth layer.
#[derive(Debug, Clone)]
pub struct CurrentUser<B>(pub B::User)
where
    B: AuthnBackend + Clone + Send + Sync + 'static,
    B::User: std::fmt::Debug + Clone + Send + Sync + 'static;

impl<S, B> FromRequestParts<S> for CurrentUser<B>
where
    B: AuthnBackend + Clone + Send + Sync + 'static,
    B::User: std::fmt::Debug + Clone + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = PurwaError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthSession::<B>::from_request_parts(parts, state)
            .await
            .map_err(|_| PurwaError::internal("auth session unavailable"))?;

        let Some(user) = auth.user.clone() else {
            return Err(PurwaError::unauthorized("login required"));
        };

        Ok(CurrentUser(user))
    }
}
