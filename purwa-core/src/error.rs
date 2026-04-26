//! Application errors (Sprint 5 stub; extended in Sprint 10).
//!
//! # Validation JSON (422)
//!
//! Failed [`validator::Validate`] checks return **422 Unprocessable Entity** with a body suitable
//! for Svelte / Inertia forms:
//! `{ "message": "Validation failed", "errors": { "field": ["…"] } }`.

use std::collections::HashMap;

use axum::Json;
use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

/// JSON body for validation failures (Laravel-style `errors` map).
#[derive(Debug, Serialize)]
pub struct ValidationErrorBody {
    pub message: &'static str,
    pub errors: HashMap<String, Vec<String>>,
}

/// Central framework error type (minimal in S5; grows in S10).
#[derive(Debug, Error)]
pub enum PurwaError {
    /// Field validation failed ([`Validate`](validator::Validate)); responds with **422**.
    #[error("validation failed")]
    Validation(#[from] ValidationErrors),
    /// JSON body could not be parsed or did not match the DTO; responds with **400**.
    #[error("invalid JSON body: {0}")]
    MalformedJson(String),
    /// Form or query deserialization failed; responds with **400**.
    #[error("invalid form data: {0}")]
    MalformedForm(String),
}

impl PurwaError {
    /// Convert Axum's [`JsonRejection`] into **400** [`PurwaError::MalformedJson`].
    pub fn from_json_rejection(rejection: JsonRejection) -> Self {
        Self::MalformedJson(rejection.to_string())
    }

    /// Convert Axum's [`FormRejection`] into **400** [`PurwaError::MalformedForm`].
    pub fn from_form_rejection(rejection: FormRejection) -> Self {
        Self::MalformedForm(rejection.to_string())
    }
}

impl IntoResponse for PurwaError {
    fn into_response(self) -> Response {
        match self {
            PurwaError::Validation(errors) => {
                let body = ValidationErrorBody {
                    message: "Validation failed",
                    errors: flatten_validation_errors(&errors),
                };
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }
            PurwaError::MalformedJson(msg) => {
                let body = serde_json::json!({ "message": msg });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }
            PurwaError::MalformedForm(msg) => {
                let body = serde_json::json!({ "message": msg });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }
        }
    }
}

/// Flatten [`ValidationErrors`] to dot-path keys and message lists (nested structs / lists supported).
pub fn flatten_validation_errors(errors: &ValidationErrors) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    flatten_recursive(errors, "", &mut out);
    out
}

fn flatten_recursive(
    errors: &ValidationErrors,
    prefix: &str,
    out: &mut HashMap<String, Vec<String>>,
) {
    for (field, kind) in errors.errors() {
        let path = if prefix.is_empty() {
            field.to_string()
        } else {
            format!("{prefix}.{field}")
        };
        match kind {
            ValidationErrorsKind::Field(errs) => {
                let msgs: Vec<String> = errs.iter().map(validation_error_message).collect();
                out.entry(path).or_default().extend(msgs);
            }
            ValidationErrorsKind::Struct(inner) => {
                flatten_recursive(inner, &path, out);
            }
            ValidationErrorsKind::List(list) => {
                for (idx, inner) in list {
                    let p = format!("{path}.{idx}");
                    flatten_recursive(inner, &p, out);
                }
            }
        }
    }
}

fn validation_error_message(e: &ValidationError) -> String {
    e.message
        .as_ref()
        .map(|m| m.to_string())
        .unwrap_or_else(|| e.code.to_string())
}
