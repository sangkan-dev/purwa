//! Application errors (Sprint 5 validation; Sprint 10 central enum + HTTP mapping).
//!
//! # Validation JSON (422)
//!
//! Failed [`validator::Validate`] checks return **422 Unprocessable Entity** with a body suitable
//! for Svelte / Inertia forms:
//! `{ "message": "Validation failed", "errors": { "field": ["…"] } }`.
//!
//! # Libraries vs applications
//!
//! Use **`thiserror`** and [`PurwaError`] in Purwa library crates. Application binaries may use
//! **`anyhow`** at startup and convert to [`PurwaError`] at HTTP boundaries (see workspace README).

use std::collections::HashMap;

use axum::Json;
use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::{Map, Value};
use thiserror::Error;
use tracing::error;
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

/// JSON body for validation failures (Laravel-style `errors` map).
#[derive(Debug, Serialize)]
pub struct ValidationErrorBody {
    pub message: &'static str,
    pub errors: HashMap<String, Vec<String>>,
}

/// Central framework error type.
#[derive(Debug, Error)]
pub enum PurwaError {
    /// Field validation failed ([`validator::Validate`]); **422**.
    #[error("validation failed")]
    Validation(#[from] ValidationErrors),
    /// JSON body could not be parsed or did not match the DTO; **400**.
    #[error("invalid JSON body: {0}")]
    MalformedJson(String),
    /// Form or query deserialization failed; **400**.
    #[error("invalid form data: {0}")]
    MalformedForm(String),
    /// Session or credentials missing; **401**.
    #[error("{message}")]
    Unauthorized { message: String },
    /// Authenticated but not allowed; **403**.
    #[error("{message}")]
    Forbidden { message: String },
    /// Resource not found; **404**.
    #[error("{message}")]
    NotFound { message: String },
    /// Database layer failure; **500** (details logged, not echoed).
    #[error("database error")]
    Database(#[source] sqlx::Error),
    /// Generic server error; **500** (`message` is safe for clients).
    #[error("{message}")]
    Internal { message: String },
}

impl PurwaError {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Convert Axum's [`JsonRejection`] into **400** [`PurwaError::MalformedJson`].
    pub fn from_json_rejection(rejection: JsonRejection) -> Self {
        Self::MalformedJson(rejection.to_string())
    }

    /// Convert Axum's [`FormRejection`] into **400** [`PurwaError::MalformedForm`].
    pub fn from_form_rejection(rejection: FormRejection) -> Self {
        Self::MalformedForm(rejection.to_string())
    }

    /// HTTP status for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            PurwaError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            PurwaError::MalformedJson(_) | PurwaError::MalformedForm(_) => StatusCode::BAD_REQUEST,
            PurwaError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            PurwaError::Forbidden { .. } => StatusCode::FORBIDDEN,
            PurwaError::NotFound { .. } => StatusCode::NOT_FOUND,
            PurwaError::Database(e) => database_status(e),
            PurwaError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// JSON value merged into Inertia page `props` for the shared **`Error`** page component.
    pub fn inertia_error_props(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            "status".to_string(),
            Value::Number(self.status_code().as_u16().into()),
        );
        map.insert(
            "errors".to_string(),
            Value::Object(self.validation_errors_map_json()),
        );
        let msg = self.safe_client_message();
        map.insert("message".to_string(), Value::String(msg));
        Value::Object(map)
    }

    /// Flat `errors` object for Inertia (empty unless validation).
    pub fn validation_errors_map_json(&self) -> Map<String, Value> {
        match self {
            PurwaError::Validation(e) => {
                let flat = flatten_validation_errors(e);
                let mut m = Map::new();
                for (k, v) in flat {
                    m.insert(k, Value::Array(v.into_iter().map(Value::String).collect()));
                }
                m
            }
            _ => Map::new(),
        }
    }

    fn safe_client_message(&self) -> String {
        match self {
            PurwaError::Validation(_) => "Validation failed".to_string(),
            PurwaError::MalformedJson(_) | PurwaError::MalformedForm(_) => {
                "The request could not be processed".to_string()
            }
            PurwaError::Unauthorized { message } => message.clone(),
            PurwaError::Forbidden { message } => message.clone(),
            PurwaError::NotFound { message } => message.clone(),
            PurwaError::Database(_) => "A database error occurred".to_string(),
            PurwaError::Internal { message } => message.clone(),
        }
    }
}

fn database_status(e: &sqlx::Error) -> StatusCode {
    match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl From<sqlx::Error> for PurwaError {
    fn from(value: sqlx::Error) -> Self {
        if matches!(value, sqlx::Error::RowNotFound) {
            return Self::NotFound {
                message: "Record not found".to_string(),
            };
        }
        Self::Database(value)
    }
}

impl IntoResponse for PurwaError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let safe = self.safe_client_message();
        match self {
            PurwaError::Validation(errors) => {
                let body = ValidationErrorBody {
                    message: "Validation failed",
                    errors: flatten_validation_errors(&errors),
                };
                (status, Json(body)).into_response()
            }
            PurwaError::Database(e) => {
                error!(error = %e, "database error");
                (status, Json(serde_json::json!({ "message": safe }))).into_response()
            }
            PurwaError::Internal { message } => {
                error!(%message, "internal error");
                (status, Json(serde_json::json!({ "message": message }))).into_response()
            }
            _ => (status, Json(serde_json::json!({ "message": safe }))).into_response(),
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
