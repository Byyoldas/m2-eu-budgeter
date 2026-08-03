//! Re-export shim — all DTOs (input and output) were extracted to `erc-core`
//! in Milestone 1, Steps 4 and 6 (see docs/executer/shared-core-roadmap.md
//! §5). Kept as a thin re-export so every existing
//! `crate::domain::dto::...` reference in this crate keeps resolving
//! unchanged.

pub use erc_core::domain::dto::*;
