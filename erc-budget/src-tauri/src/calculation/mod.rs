//! Re-export shim — the entire calculation engine (all 8 modules) was
//! extracted to `erc-core` in Milestone 1, Step 6 (see
//! docs/executer/shared-core-roadmap.md §5). Submodule paths like
//! `crate::calculation::salary_projection::project_salary_chain` keep
//! resolving unchanged because a glob re-export of a module also re-exports
//! its public submodules.

pub use erc_core::calculation::*;
