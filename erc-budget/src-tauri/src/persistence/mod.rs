//! Re-export shim — `persistence/mod.rs` was extracted to `erc-core` in
//! Milestone 1, Step 7 (see docs/executer/shared-core-roadmap.md §5), which
//! also added the optional `execution_data` field (format v1.1) to
//! `ProjectFile`. The Budget Application always saves with
//! `execution_data: None`, so it continues to write plain v1.0 files.

pub use erc_core::persistence::*;
