//! Re-export of the shared error types. All execution-specific error cases
//! are expressed as existing `erc_core::error::AppError` variants (mainly
//! `Validation`, `Persistence`, `NotFound`, and `NoProject`) — no new
//! variants needed yet.

pub use erc_core::error::{AppError, FieldError, ValidationErrors};
