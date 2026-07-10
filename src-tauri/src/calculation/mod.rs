//! Calculation Engine — pure functions, no side effects, no I/O.
//!
//! All monetary arithmetic uses `rust_decimal::Decimal` for exact decimal
//! representation. No `f32` or `f64` values are used anywhere in this module.
//!
//! Functions return `Result<T, AppError>`. They never panic.
//! Every error condition is explicitly handled and mapped to a named error code.

pub mod salary_projection;
pub mod personnel_cost;
pub mod equipment_depreciation;
pub mod trip_cost;
pub mod budget_aggregator;
pub mod cfs_checker;
pub mod budget_summary;

// Re-export the master orchestration function for convenience.
pub use budget_summary::calculate_budget_summary;
