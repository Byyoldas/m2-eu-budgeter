//! Shared core library for the ERC Budget and ERC Execution applications.
//!
//! Modules are added here incrementally as they are extracted from
//! `erc-budget` following the strangler-fig plan in
//! `docs/executer/shared-core-roadmap.md`. This crate must never depend on
//! `erc-budget` or `erc-execution`.

pub mod calculation;
pub mod domain;
pub mod error;
pub mod persistence;
pub mod validation;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_links() {
        assert_eq!(2 + 2, 4);
    }
}
