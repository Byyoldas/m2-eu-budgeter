//! Centralised error types for the ERC Budget application.
//!
//! All IPC commands return `Result<T, AppError>`. `AppError` implements
//! `serde::Serialize` so Tauri can serialise it to JSON and the frontend
//! can deserialise the structured error.

use serde::{Deserialize, Serialize};

/// The top-level error type returned from every Tauri IPC command.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(tag = "kind", content = "detail")]
pub enum AppError {
    /// One or more input fields failed validation.
    #[error("Validation error: {0:?}")]
    Validation(Vec<FieldError>),

    /// A calculation produced an unexpected result (programming error).
    #[error("Calculation error [{code}]: {message}")]
    Calculation { code: String, message: String },

    /// A file-system operation failed.
    #[error("Persistence error: {0}")]
    Persistence(String),

    /// A requested entity was not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// No project is currently open.
    #[error("No project is open. Create or load a project first.")]
    NoProject,

    /// An unexpected internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// A structured validation error for a single form field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    /// The field name as used in the form (matches the DTO field name).
    /// None for entity-level constraints that span multiple fields.
    pub field: Option<String>,
    /// Machine-readable error code (e.g. "INVALID_FTE").
    pub code: String,
    /// Human-readable message shown in the UI.
    pub message: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: Some(field.into()),
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn entity(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: None,
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Convenience builder for collecting multiple field errors before returning.
#[derive(Default)]
pub struct ValidationErrors(Vec<FieldError>);

impl ValidationErrors {
    pub fn push(&mut self, err: FieldError) {
        self.0.push(err);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_result(self) -> Result<(), AppError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(self.0))
        }
    }
}

/// Shorthand to wrap a calculation error.
pub fn calc_error(code: &str, message: impl Into<String>) -> AppError {
    AppError::Calculation {
        code: code.to_string(),
        message: message.into(),
    }
}
