# Shared Core Refactoring Plan

**Phase 07 — Shared Core Refactoring Plan**
**Project:** Horizon Europe Project Management Platform
**Date:** 2026-08-03

---

## 1. Purpose

This document specifies the exact refactoring required to extract shared components from the existing Budget Application into the `erc-core` Shared Core Library. The Budget Application must continue working identically after every refactoring step. This is a zero-regression migration.

---

## 2. Guiding Principles

- **Strangler fig pattern**: Extract one module at a time. Replace the original with a re-export from `erc-core`. Test after each step.
- **No business logic changes**: This phase is purely structural movement. No calculations are altered.
- **Budget App remains green**: Every extraction is immediately followed by a full test run of the Budget Application.
- **No API changes**: The IPC command signatures, DTO structures, and TypeScript types do not change.
- **Backward file compatibility**: Existing `.ercbudget` files must continue to open without error.

---

## 3. What Gets Extracted

### 3.1 Extraction Inventory

| Module (current location) | Extract to erc-core? | Effort | Risk |
|---|---|---|---|
| `src-tauri/src/error.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/domain/entities.rs` | ✅ Yes — fully | Medium | Medium |
| `src-tauri/src/domain/dto.rs` | ✅ Yes — shared DTOs only | Medium | Low |
| `src-tauri/src/domain/rate_data.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/calculation/salary_projection.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/calculation/personnel_cost.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/calculation/equipment_depreciation.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/calculation/trip_cost.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/calculation/budget_aggregator.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/calculation/cfs_checker.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/calculation/wp_budget.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/calculation/budget_summary.rs` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/validation/mod.rs` | ✅ Yes — shared validators | Medium | Low |
| `src-tauri/src/persistence/mod.rs` | ✅ Yes — extended | Medium | Medium |
| `src-tauri/resources/eu_travel_rates/*.json` | ✅ Yes — fully | Low | Low |
| `src-tauri/src/commands/*.rs` | ❌ No — app-specific | — | — |
| `src/types/index.ts` | ⚠️ Partial — shared types | Medium | Low |
| `src/validators/schemas.ts` | ⚠️ Partial — shared schemas | Low | Low |
| `src/components/*.tsx` | ⚠️ Partial — generic only | Medium | Low |
| `src/export/*.ts` | ❌ No — app-specific | — | — |
| `src/store/*.ts` | ❌ No — app-specific | — | — |
| `src/screens/*.tsx` | ❌ No — app-specific | — | — |

---

## 4. Cargo Workspace Setup

**Step 0 (prerequisite): Create the workspace structure.**

### 4.1 New Repository / Workspace Layout

```
erc-platform/                    ← new workspace root (or existing repo reorganised)
├── Cargo.toml                   ← workspace definition
├── erc-core/                    ← Shared Core Library (new crate)
│   ├── Cargo.toml
│   └── src/
├── erc-budget/                  ← renamed from m2-eu-budgeter-executer
│   ├── src/                     ← React frontend (unchanged)
│   ├── src-tauri/               ← Rust backend (refactored to use erc-core)
│   └── package.json
└── erc-execution/               ← new application (Phase 09+)
    ├── src/
    ├── src-tauri/
    └── package.json
```

### 4.2 Workspace Cargo.toml

```toml
[workspace]
members = [
    "erc-core",
    "erc-budget/src-tauri",
    "erc-execution/src-tauri",
]
resolver = "2"
```

### 4.3 erc-core Cargo.toml

```toml
[package]
name = "erc-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rust_decimal = { version = "1", features = ["serde-str"] }
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
rust_decimal_macros = "1"
```

---

## 5. Extraction Steps — Rust Backend

Each step follows the same procedure:
1. Create the module in `erc-core`
2. Copy the code (do not modify logic)
3. In the Budget App, replace the original file with `pub use erc_core::... ;`
4. Run `cargo test` in the workspace root
5. Run the Budget App and verify it opens/saves a project correctly

---

### Step 1: Extract `error.rs`

**Complexity: Low | Risk: Low**

This is the safest first extraction — no domain types depend on it, but everything depends on it.

**Action in erc-core:**
```
erc-core/src/error.rs    ← copy content of erc-budget/src-tauri/src/error.rs
erc-core/src/lib.rs      ← pub mod error;
```

**Action in erc-budget/src-tauri/src/error.rs:**
```rust
// error.rs — now a re-export shim
pub use erc_core::error::{AppError, FieldError, ValidationErrors, calc_error};
```

**Verification:** `cargo test -p erc-budget` must pass with zero changes to test code.

---

### Step 2: Extract `domain/rate_data.rs`

**Complexity: Low | Risk: Low**

`RateData` is a self-contained struct with no domain entity dependencies.

**Action in erc-core:**
```
erc-core/src/domain/rate_data.rs    ← copy content
erc-core/resources/eu_travel_rates/ ← move all 3 JSON files
```

Update `build.rs` in erc-core to embed the rate files:
```rust
// erc-core/build.rs is not needed; use include_str! in rate_data.rs
```

The `include_str!` macro paths need updating to be relative to the crate root:
```rust
const RATE_V1: &str = include_str!("../resources/eu_travel_rates/v_before_2024_07_31.json");
```

**Action in erc-budget/src-tauri/src/domain/rate_data.rs:**
```rust
pub use erc_core::domain::rate_data::*;
```

---

### Step 3: Extract `domain/entities.rs`

**Complexity: Medium | Risk: Medium**

This is the core domain model. All other backend modules depend on it.

**Action in erc-core:**
```
erc-core/src/domain/entities.rs    ← copy content
erc-core/src/domain/mod.rs         ← pub mod entities; pub mod rate_data;
```

**One change required:** The `WorkPackage` entity should be promoted here as an explicit struct (as specified in Phase 02). This is additive and backward-compatible:

```rust
/// Explicit WorkPackage entity (promoted from ProjectConfig arrays).
/// Derived from ProjectConfig arrays during loading; not stored separately in v1.0 files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackage {
    pub id: u8,
    pub name: Option<String>,
    pub start_month: u32,
    pub end_month: u32,
}

impl ProjectConfig {
    /// Derive explicit WorkPackage structs from the array-based representation.
    pub fn work_packages(&self) -> Vec<WorkPackage> {
        (0..self.work_package_count as usize)
            .map(|i| WorkPackage {
                id: (i + 1) as u8,
                name: self.work_package_names.get(i).cloned().flatten(),
                start_month: self.work_package_start_months.get(i).copied().unwrap_or(1),
                end_month: self.work_package_end_months.get(i)
                    .copied()
                    .unwrap_or(self.duration_years as u32 * 12),
            })
            .collect()
    }
}
```

**Action in erc-budget/src-tauri/src/domain/entities.rs:**
```rust
pub use erc_core::domain::entities::*;
```

---

### Step 4: Extract `domain/dto.rs` (shared portion)

**Complexity: Low | Risk: Low**

Only the DTOs used by both apps are extracted. Budget-App-specific DTOs (e.g., `RoleCostLineDto`, `BudgetSummaryDto`) are moved with the calculation engine in Step 6.

Shared DTOs to extract:
- `ProjectConfigDto` (input)
- `PersonnelRoleInputDto`
- `EquipmentItemInputDto`
- `TripInputDto`
- `OtherCostInputDto`
- `WpCostAmountDto`

All output DTOs (`BudgetSummaryDto`, `PersonnelRoleDetailDto`, etc.) stay with the budget engine for now and move in Step 6.

---

### Step 5: Extract `validation/mod.rs`

**Complexity: Medium | Risk: Low**

All 5 shared validators are extracted. Their signatures don't change.

**Action in erc-core:**
```
erc-core/src/validation/mod.rs    ← copy all 5 validator functions + tests
```

**Action in erc-budget/src-tauri/src/validation/mod.rs:**
```rust
pub use erc_core::validation::*;
// Keep only Budget-App-specific validators here (none currently)
```

All 35+ validation tests move to `erc-core`. The Budget App's test suite should reference `erc-core` validation tests or add integration tests that test the IPC command layer.

---

### Step 6: Extract `calculation/` (all modules)

**Complexity: Low | Risk: Low** (high confidence because these are pure functions)

Extract all 9 calculation modules in a single step (they are tightly coupled and move cleanly together):

```
erc-core/src/calculation/
├── mod.rs
├── salary_projection.rs
├── personnel_cost.rs
├── equipment_depreciation.rs
├── trip_cost.rs
├── budget_aggregator.rs
├── cfs_checker.rs
├── wp_budget.rs
└── budget_summary.rs
```

This also means all output DTOs (`BudgetSummaryDto`, `RoleCostLineDto`, etc.) move to `erc-core/src/domain/dto.rs` (full version).

**Action in erc-budget:** Replace all `calculation/` modules with re-exports:

```rust
// erc-budget/src-tauri/src/calculation/mod.rs
pub use erc_core::calculation::*;
```

All ~100 calculation tests move to `erc-core`. This is a significant test asset — it validates the shared core comprehensively.

---

### Step 7: Extract `persistence/mod.rs`

**Complexity: Medium | Risk: Medium**

The persistence module is extracted with the file format extended to include the optional `execution_data` field (v1.1 support, §5 of Phase 03 spec).

**Key change:** `ProjectFile` struct in `erc-core` now has the optional `execution_data` field. The Budget App continues to write only the `project` block (the field is `None` and `skip_serializing_if` means it is not written).

```rust
// erc-core/src/persistence/mod.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    pub format_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub project: Project,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_data: Option<serde_json::Value>,  // Opaque to erc-core; typed in erc-execution
}
```

The `execution_data` is typed as `Option<serde_json::Value>` in `erc-core` (opaque JSON) and typed as `Option<ExecutionData>` when deserialized by the Execution Application. This avoids a circular dependency where `erc-core` would need to know about execution-specific types.

**Backward compatibility test:** Existing v1.0 `.ercbudget` files must load without error (the field is absent → `None`). A regression test suite should be added with real `.ercbudget` test fixtures.

---

## 6. Extraction Steps — TypeScript / Frontend

### Step 8: Shared TypeScript Types

**Complexity: Medium | Risk: Low**

Create a shared TypeScript types package. Two implementation options:

**Option A (ts-rs — Recommended):** Add `ts-rs` to `erc-core` dev dependencies. Annotate all shared Rust structs with `#[derive(TS)]`. Run `cargo test --features ts-rs` to generate TypeScript files. Place generated files in `erc-core/bindings/`.

Both applications import from `erc-core/bindings/` (referenced via relative path or npm workspace package `@erc/core-types`).

**Option B (Manual):** Manually maintain `@erc/core-types/src/index.ts` containing all shared types. Both apps list it as a workspace dependency.

In either case, the types that stay **app-specific** in each application:
- `Screen` type and `SCREENS` constant
- App-specific state types
- Export-specific types

### Step 9: Shared Zod Schemas

**Complexity: Low | Risk: Low**

Extract schemas that both apps use from `validators/schemas.ts`:
- `projectSetupSchema`
- `budgetSettingsSchema`
- `personnelRoleSchema`
- `equipmentItemSchema`
- `tripSchema`
- `otherCostSchema`

Place in `@erc/core-types/src/schemas.ts`.

### Step 10: Shared UI Components

**Complexity: Medium | Risk: Low**

The following Budget App components are generic enough to share:

| Component | Share? | Notes |
|---|---|---|
| `FormField.tsx` | ✅ | Pure layout wrapper |
| `EmptyStateCard.tsx` | ✅ | Pure display |
| `WarningBanner.tsx` | ✅ | Generic warning display |
| `LivePreviewBox.tsx` | ✅ | Generic preview container |
| `ProgressStepper.tsx` | ❌ | Budget App wizard-specific |
| `BudgetRingChart.tsx` | ❌ | Budget App-specific data shape |
| `BudgetWpBarChart.tsx` | ❌ | Budget App-specific |
| `WorkPackageGanttChart.tsx` | ⚠️ | Shared if extracted to accept generic WP data |
| `UpdateChecker.tsx` | ✅ | Shared (both apps need auto-update) |
| `CFSModal.tsx` | ❌ | Budget App-specific flow |
| `CategoryTotalsPanel.tsx` | ❌ | Budget App sidebar-specific |
| `RoleCard.tsx` | ❌ | Budget App-specific layout |

Generic shared components can live in a `@erc/ui` package or be copied (copy is acceptable for MVP given the small number of components).

---

## 7. Migration Strategy — Step-by-Step Timeline

| Week | Step | Description | Risk Gate |
|---|---|---|---|
| W1 | Step 0 | Create workspace + `erc-core` skeleton | Budget App builds identically |
| W1 | Step 1 | Extract `error.rs` | Full test suite green |
| W1 | Step 2 | Extract `rate_data.rs` + resources | Rate lookup tests green |
| W2 | Step 3 | Extract `domain/entities.rs` | Serialization round-trip test green |
| W2 | Step 4 | Extract shared DTOs | IPC command types unchanged |
| W2 | Step 5 | Extract `validation/mod.rs` | All 35+ validation tests green |
| W3 | Step 6 | Extract `calculation/` (all) | All ~100 calculation tests green |
| W3 | Step 7 | Extract `persistence/mod.rs` | v1.0 file round-trip test green |
| W4 | Step 8 | Shared TypeScript types | Frontend builds identically |
| W4 | Step 9 | Shared Zod schemas | Frontend tests green |
| W4 | Step 10 | Shared UI components | Storybook / manual visual review |
| W4 | — | Budget App full regression | All tests green; manual smoke test |

**Total estimated effort: 2–3 developer-weeks**

---

## 8. Testing Strategy During Migration

### 8.1 Regression Tests to Add Before Starting

Before any extraction begins, add these integration tests to the Budget App to serve as a safety net:

```rust
// src-tauri/tests/regression_tests.rs

#[test]
fn test_existing_v1_0_file_loads_correctly() {
    // Load a real .ercbudget file created by the current app
    // Verify: all entities present, budget summary calculates correctly
}

#[test]
fn test_file_save_and_reload_roundtrip() {
    // Create a project, save it, reload it
    // Verify: all values identical to before save
}

#[test]
fn test_calculation_reference_values() {
    // A set of known input → expected output pairs
    // These values are the regression baseline
    assert_eq!(total_eligible, dec!(578834.75));
    assert_eq!(category_a, dec!(402500.00));
    // etc.
}
```

### 8.2 Per-Step Verification

After each step:
1. `cargo test --workspace` must pass with 0 failures
2. `pnpm test` must pass with 0 failures
3. The Budget App must build and run (manual smoke test: create project, add role, save, reload)

### 8.3 Test Fixtures

Create a `test-fixtures/` directory in the workspace root with:
- `simple_project_v1_0.ercbudget` — a minimal valid v1.0 file
- `full_project_v1_0.ercbudget` — a comprehensive file with all entity types
- Expected calculation output JSON for each fixture

---

## 9. What Remains in erc-budget After Extraction

After all extractions, `erc-budget/src-tauri/src/` contains only:
- `main.rs`, `lib.rs` — application wiring (thin)
- `commands/*.rs` — IPC command handlers (use `erc_core::*` for all business logic)
- `error.rs`, `domain/`, `calculation/`, `validation/`, `persistence/` — re-export shims only

The Budget Application becomes a **thin Tauri shell** over `erc-core`. This is the target state.

---

## 10. erc-core Public API (after full extraction)

```rust
// erc-core/src/lib.rs

pub mod domain {
    pub mod entities;    // Project, ProjectConfig, PersonnelRole, EquipmentItem, Trip, etc.
    pub mod dto;         // All shared DTOs
    pub mod rate_data;   // RateData, RateVersion, FlightBand, CountryRate
}

pub mod calculation {
    pub mod salary_projection;
    pub mod personnel_cost;
    pub mod equipment_depreciation;
    pub mod trip_cost;
    pub mod budget_aggregator;
    pub mod cfs_checker;
    pub mod wp_budget;
    pub mod budget_summary;  // Re-exports calculate_budget_summary
}

pub mod validation;          // All shared validators
pub mod persistence;         // ProjectFile, save_project, load_project, auto_save
pub mod error;               // AppError, FieldError, ValidationErrors, calc_error
pub mod constants;           // CFS_THRESHOLD_EUR, EU_FUNDING_RATE, etc.
```

---

## 11. Version Policy

`erc-core` uses semantic versioning:

| Change type | Version bump |
|---|---|
| New public function, new optional field | PATCH (0.1.x) |
| New public module, new entity type | MINOR (0.x.0) |
| Breaking API change (rename, remove, type change) | MAJOR (x.0.0) |

During the initial extraction period (Phase 07), version is `0.1.0`. First stable release (after Phase 09 MVP) is `1.0.0`.

---

## 12. Open Questions

1. Should the workspace be a new Git repository (`erc-platform`) or should `erc-budget` remain its own repo with `erc-core` as a Git submodule?
2. Should `erc-core` version be locked in both apps (`=0.1.0`) or pinned to a compatible range (`^0.1`)?
3. Should `ts-rs` be adopted immediately for type generation, or deferred to after the Execution App is built?

---

## 13. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Budget App breaks during extraction | High | Regression test suite added before Step 1; revert immediately on red |
| `erc-core` extraction takes longer than 3 weeks, delaying Execution App | Medium | Steps 1–7 (Rust) are the critical path; Steps 8–10 (TypeScript) can proceed in parallel |
| Circular dependency introduced accidentally | High | CI gate: `cargo deny check bans` enforces no circular crate deps |
| Serialization changes break v1.0 files | High | Round-trip regression test with real files added before Step 7 |

---

## 14. Assumptions

- The Budget Application is feature-complete and will not receive new features during the extraction period (breaking the extraction work)
- The development team can work on `erc-core` extraction without blocking active Budget App users (no users in production during Phase 07)
- Rust stable toolchain is used throughout; no nightly features required

---

## 15. Confidence Level

**92%** — The extraction plan is methodical and low-risk because all modules are pure functions or data structures with no hidden global state. The only uncertainty is around the `persistence/mod.rs` extension (v1.1 format) and ensuring existing files are not corrupted.

---

## 16. Recommended Next Step

**Proceed to Phase 08 — Development Roadmap.**

The Shared Core extraction plan is defined. Phase 08 will sequence all work across both applications into a concrete sprint plan with milestones, dependencies, and release targets.
