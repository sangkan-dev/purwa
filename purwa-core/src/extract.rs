//! Validating Axum extractors (JSON and URL-encoded form).

use axum::Json;
use axum::extract::{Form, FromRequest, Request};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::PurwaError;

/// JSON body extractor: deserialize then run [`Validate`].
///
/// Use as the last extractor in the handler when combined with others.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send + 'static,
    S: Send + Sync,
{
    type Rejection = PurwaError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(inner) = Json::<T>::from_request(req, state)
            .await
            .map_err(PurwaError::from_json_rejection)?;
        inner.validate()?;
        Ok(ValidatedJson(inner))
    }
}

/// Form / query extractor: deserialize then run [`Validate`].
///
/// Same semantics as Axum [`Form`] (GET query vs POST `application/x-www-form-urlencoded`).
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate + Send + 'static,
    S: Send + Sync,
{
    type Rejection = PurwaError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(inner) = Form::<T>::from_request(req, state)
            .await
            .map_err(PurwaError::from_form_rejection)?;
        inner.validate()?;
        Ok(ValidatedForm(inner))
    }
}
