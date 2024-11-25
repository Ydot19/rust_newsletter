use core::fmt;
use std::error::Error;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(Debug, PartialEq)]
pub enum DomainError {
    NotFound(String),
    Validation { field: String, message: String },
    Internal(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::NotFound(msg) => {
                write!(f, "resource not found: {}", msg)
            }
            DomainError::Validation { field, message } => {
                write!(f, "validation for field {}, reason = {}", field, message)
            }
            DomainError::Internal(msg) => {
                write!(f, "internal error. reason = {}", msg)
            }
        }
    }
}

impl Error for DomainError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DomainError::NotFound(_) => todo!(),
            DomainError::Validation {
                field: _,
                message: _,
            } => {
                todo!()
            }
            DomainError::Internal(_) => todo!(),
        }
    }
}

impl DomainError {
    fn to_response(&self) -> (StatusCode, Json<serde_json::Value>) {
        let (status, msg) = match self {
            DomainError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            DomainError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DomainError::Validation { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        (status, Json(json!({"error": msg})))
    }
}

pub fn error_to_response(err: DomainError) -> impl IntoResponse {
    err.to_response()
}
