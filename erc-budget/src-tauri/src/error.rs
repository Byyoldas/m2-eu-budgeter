//! Re-export shim — `error.rs` was extracted to `erc-core` in Milestone 1,
//! Step 1 (see docs/executer/shared-core-roadmap.md §5). Kept as a thin
//! re-export so every existing `crate::error::...` reference in this crate
//! keeps resolving unchanged.
//!
//! `ValidationErrors` and `calc_error` are no longer re-exported here: their
//! only callers in this crate (`validation/mod.rs`, `calculation/*.rs`)
//! moved to `erc-core` in Steps 5 and 6, so nothing in `erc-budget`
//! references them directly any more.

pub use erc_core::error::{AppError, FieldError};
