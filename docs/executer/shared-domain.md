# Shared Domain Discovery

**Phase 02 — Shared Domain Discovery**
**Project:** Horizon Europe Project Management Platform
**Date:** 2026-08-03

---

## 1. Purpose

This document identifies every domain concept shared between the Budget Application and the future Project Execution Application. It defines the recommended boundary of a **Shared Core Library** (`erc-core`) that both applications will depend upon, ensuring no business logic is duplicated and that a future unified platform can be assembled from these building blocks.

---

## 2. Shared Domain Concept Map

The diagram below shows the full domain across both applications. Concepts in **bold** are candidates for the Shared Core.

```
┌─────────────────────────────────────────────────────────────────┐
│                    SHARED CORE DOMAIN                           │
│                                                                 │
│  Project ─────────────── ProjectConfig                         │
│      │                        │                                │
│      ├── WorkPackage ──────── WpSchedule                       │
│      │       └── [tasks, deliverables, milestones]*            │
│      │                                                         │
│      ├── PersonnelRole ────── RoleType                         │
│      │       └── [Person]*                                     │
│      │                                                         │
│      ├── EquipmentItem                                         │
│      ├── Trip ────────────── TripType                          │
│      ├── OtherDirectCostItem                                    │
│      ├── Subcontracting                                        │
│      │                                                         │
│      ├── [Partner]*                                            │
│      └── [ReportingPeriod]*                                    │
│                                                                 │
│  RateData ────────────── RateVersion                           │
│                  └────── FlightBand / CountryRate              │
│                                                                 │
│  AppError / FieldError / ValidationErrors                       │
│  FileFormat / Persistence (ProjectFile wrapper)                 │
│  Calculation utilities (salary, depreciation, trip cost)        │
│  CFS threshold constants                                        │
│                                                                 │
│  * = new concepts introduced by Execution Application           │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Entity-by-Entity Analysis

### 3.1 Project

**Status:** Shared Core — extended in Execution Application

The `Project` is the root aggregate for both applications. The Budget Application creates and owns it. The Execution Application reads and enriches it.

| Attribute | Budget App | Execution App | Action |
|---|---|---|---|
| `id: Uuid` | ✅ | ✅ | Keep in core |
| `config: ProjectConfig` | ✅ | Read-only | Keep in core |
| `personnel_roles` | ✅ | Read + track actuals | Keep in core |
| `equipment_items` | ✅ | Read + track actuals | Keep in core |
| `trips` | ✅ | Read + track actuals | Keep in core |
| `other_cost_items` | ✅ | Read + track actuals | Keep in core |
| `subcontracting` | ✅ | Read + track actuals | Keep in core |
| `cfs_warning_dismissed` | ✅ | Carry forward | Keep in core |
| `partners` | ❌ | ✅ future | Introduce in execution layer |
| `reporting_periods` | ❌ | ✅ | Introduce in execution layer |
| `execution_data` | ❌ | ✅ | Add as optional extension block |

---

### 3.2 ProjectConfig

**Status:** Shared Core — read-only in Execution Application

All fields are reused in the execution context for display, calculation, and tracking purposes. The Execution App must not mutate `ProjectConfig`; the Budget App is the source of truth.

Key shared fields:
- `project_title`, `pi_name`, `call_reference` — displayed on all dashboards
- `duration_years` — defines the time horizon for all tracking
- `work_package_count` + `work_package_names` — used in all WP-level views
- `work_package_start_months` + `work_package_end_months` — used to render Gantt and timeline views
- `default_inflation_rate_pct`, `try_eur_rate` — needed if actual vs. planned salary comparisons are made
- `indirect_cost_rate_pct` — used to compute planned overhead
- `rate_version_id` — needed for validating actual travel claims

---

### 3.3 WorkPackage (implicit in ProjectConfig)

**Status:** Shared Core — must be promoted to explicit entity

Currently, Work Packages exist only as indexed positions in arrays within `ProjectConfig`. This is adequate for the Budget Application but insufficient for the Execution Application which needs to attach tasks, deliverables, milestones, actual costs, and responsible persons to each WP.

**Recommended promotion:**

```rust
pub struct WorkPackage {
    pub id: u8,                          // 1-indexed, matches existing WP IDs
    pub name: Option<String>,
    pub start_month: u32,
    pub end_month: u32,
    // New in Execution:
    pub leader_role_id: Option<Uuid>,    // which PersonnelRole leads this WP
    pub description: Option<String>,
    pub objectives: Option<String>,
}
```

The budget arrays in `ProjectConfig` (`work_package_names`, `work_package_start_months`, `work_package_end_months`) can be preserved for backward compatibility and derived from the `WorkPackage` list when constructing the `ProjectConfig` DTO.

---

### 3.4 PersonnelRole

**Status:** Shared Core — extended by a `Person` concept in Execution Application

In the Budget App, a `PersonnelRole` is an anonymous position (e.g., "PostDoc-1"). In the Execution App, each role must be linked to an actual named individual. The role definition (salary, FTE, months, WP) stays in the shared core.

**New concept — `Person`:**

```rust
pub struct Person {
    pub id: Uuid,
    pub full_name: String,
    pub email: Option<String>,
    pub institution: Option<String>,
    pub linked_role_id: Uuid,           // FK to PersonnelRole
    pub actual_start_date: Option<String>,
    pub actual_end_date: Option<String>,
}
```

**Shared from PersonnelRole:**
- `id`, `role_label`, `role_type`, `current_monthly_salary_try`, `fte_fraction`, `inflation_rate_pct`, `start_month`, `end_month`
- Salary projection calculation (CALC-01/02/03) — used for planned vs. actual comparison

**RoleType enum:** Fully shared. Values cover all ERC personnel categories.

---

### 3.5 EquipmentItem

**Status:** Shared Core — extended with procurement tracking in Execution Application

The depreciation formula and all inputs are shared. The Execution App adds procurement tracking on top.

**New in Execution App:**

```rust
pub struct EquipmentProcurement {
    pub equipment_item_id: Uuid,
    pub purchase_date: Option<String>,
    pub actual_purchase_cost_eur: Option<Decimal>,
    pub supplier: Option<String>,
    pub invoice_reference: Option<String>,
    pub delivery_confirmed: bool,
    pub notes: Option<String>,
}
```

---

### 3.6 Trip / TripType

**Status:** Shared Core — extended with actual trip records in Execution Application

The planned trip definition (destination, distance, nights, instances) stays shared. The Execution App tracks whether each instance was actually taken and at what cost.

**New in Execution App:**

```rust
pub struct TripExecution {
    pub trip_id: Uuid,
    pub instance_number: u32,
    pub traveller_role_id: Uuid,
    pub actual_travel_date: Option<String>,
    pub actual_cost_eur: Option<Decimal>,
    pub receipts_uploaded: bool,
    pub approved: bool,
    pub notes: Option<String>,
}
```

---

### 3.7 OtherDirectCostItem

**Status:** Shared Core — extended with procurement/payment tracking in Execution Application

The planned item (name, amount, WP) stays shared. Execution App tracks payment status, invoices, and vendor information.

---

### 3.8 Subcontracting

**Status:** Shared Core — extended with contract details in Execution Application

Currently a single amount field. The Execution App needs a `SubcontractingLine` list with vendor, contract reference, deliverables, and payment milestones.

---

### 3.9 RateData / RateVersion / FlightBand / CountryRate

**Status:** Shared Core — fully shared, read-only

The EU travel rate tables are needed by:
- Budget App: calculating planned trip costs
- Execution App: validating actual travel claims against EU unit rates, computing reimbursement amounts

The embedded JSON files and the `rate_data.rs` loading module should be extracted to the Shared Core unchanged.

---

### 3.10 Error Handling (`error.rs`)

**Status:** Shared Core — fully shared

`AppError`, `FieldError`, `ValidationErrors`, and the `calc_error()` helper are generic infrastructure with no budget-specific logic. They should be the first module extracted.

---

### 3.11 Persistence (`persistence/mod.rs`)

**Status:** Shared Core — the file format wrapper is shared; the execution data extension is new

The `ProjectFile` wrapper (`format_version`, `created_at`, `updated_at`, `project`) is shared. The Execution Application will add an optional `execution_data` block to this wrapper (see Phase 03).

The `save_project()`, `load_project()`, and `auto_save()` functions are shared with minor extension.

---

### 3.12 Calculation Engine (`calculation/`)

**Status:** Partially shared

| Module | Share? | Reason |
|---|---|---|
| `salary_projection.rs` | ✅ Full | Planned vs. actual salary comparison |
| `personnel_cost.rs` | ✅ Full | WP allocation logic reused for actual tracking |
| `equipment_depreciation.rs` | ✅ Full | Actual eligible amount calculation |
| `trip_cost.rs` | ✅ Full | Actual trip cost validation against EU rates |
| `budget_aggregator.rs` | ✅ Full | Actual totals use same aggregation logic |
| `cfs_checker.rs` | ✅ Full | CFS compliance tracking continues in execution |
| `budget_summary.rs` | ⚠️ Partial | Orchestration logic diverges; share sub-functions |
| `wp_budget.rs` | ✅ Full | Per-WP view needed in execution dashboards |

---

### 3.13 Validation Engine (`validation/mod.rs`)

**Status:** Partially shared

| Validator | Share? | Notes |
|---|---|---|
| `validate_project_config` | ✅ | Same rules apply |
| `validate_personnel_role` | ✅ | Reused when editing project config in execution context |
| `validate_equipment_item` | ✅ | Reused when adding items during execution |
| `validate_trip` | ✅ | Reused for new travel entries |
| `validate_other_cost` | ✅ | Reused |
| Actual cost validators | ❌ | New — validate actual amounts vs. planned amounts |
| Progress validators | ❌ | New — validate deliverable/milestone dates |

---

### 3.14 TypeScript Types (`types/index.ts`)

**Status:** Shared Core (TypeScript layer)

All existing types are reused. The Execution App extends them with new DTOs:

| Existing Type | Status in Execution App |
|---|---|
| `RoleType` | Shared — unchanged |
| `ProjectConfigInput` | Shared — read-only in execution forms |
| `PersonnelRoleInput` | Shared — used if editing roles |
| `EquipmentItemInput` | Shared |
| `TripInput` | Shared |
| `OtherCostInput` | Shared |
| `BudgetSummaryDto` | Shared — forms the "Planned" side of all comparisons |
| `WpBudgetDto` | Shared — used in execution WP dashboards |
| `AppError` / `FieldError` | Shared — unchanged |
| `Screen` | Application-specific — each app defines its own |

---

### 3.15 New Concepts Introduced by the Execution Application

These entities do not exist in the Budget Application and will live in the Execution Application's own domain layer (not the Shared Core), unless they eventually prove useful to both apps.

| Entity | Description |
|---|---|
| `ReportingPeriod` | Defines P1, P2, P3 intervals; each period has start/end months and a submission deadline |
| `Milestone` | A named project milestone attached to a WP with a planned month and actual completion date |
| `Deliverable` | A named output attached to a WP with type (Report, Dataset, Software, etc.), planned month, responsible role, and status |
| `Task` | A granular work item within a WP; assigned to one or more roles; has start/end months and completion status |
| `RiskEntry` | Risk register entry: description, probability, impact, mitigation, status, owner |
| `IssueEntry` | Issue log entry: description, raised date, priority, resolution, owner |
| `Meeting` | Meeting record: date, type, attendees, agenda, minutes reference |
| `ActionItem` | Action item from a meeting: description, owner, due date, status |
| `PersonMonthRecord` | Actual person-month consumption: role, period, reported months, approved months |
| `ActualCostEntry` | A submitted actual cost claim: category, amount, reference, period, status |
| `Document` | A stored document reference: type (report, invoice, contract), upload date, path/URL, linked to entity |
| `SubcontractingLine` | Individual subcontract: vendor, amount, contract reference, WP, payment milestones |

---

## 4. Shared Core Library Specification

### 4.1 Recommended Package Name

`erc-core` (Rust crate) + `@erc/core` (TypeScript package, if separated)

### 4.2 Crate Structure

```
erc-core/
├── src/
│   ├── lib.rs
│   ├── error.rs              # AppError, FieldError, ValidationErrors
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entities.rs       # Project, ProjectConfig, WorkPackage, PersonnelRole,
│   │   │                     # EquipmentItem, Trip, OtherDirectCostItem,
│   │   │                     # Subcontracting, RoleType, TripType
│   │   ├── dto.rs            # All shared DTOs
│   │   └── rate_data.rs      # RateData, RateVersion, FlightBand, CountryRate
│   ├── calculation/
│   │   ├── mod.rs
│   │   ├── salary_projection.rs
│   │   ├── personnel_cost.rs
│   │   ├── equipment_depreciation.rs
│   │   ├── trip_cost.rs
│   │   ├── budget_aggregator.rs
│   │   ├── cfs_checker.rs
│   │   └── wp_budget.rs
│   ├── validation/
│   │   └── mod.rs            # Shared validators
│   ├── persistence/
│   │   └── mod.rs            # ProjectFile wrapper + save/load/auto_save
│   └── constants.rs          # CFS_THRESHOLD_EUR, EU_FUNDING_RATE, etc.
├── resources/
│   └── eu_travel_rates/      # Embedded JSON rate tables
└── Cargo.toml
```

### 4.3 Dependency Direction (Law — must not be violated)

```
erc-budget  ──depends on──▶  erc-core
erc-execution ──depends on──▶  erc-core

erc-core must NEVER depend on erc-budget or erc-execution.
```

### 4.4 TypeScript Shared Types Package

A mirrored TypeScript package (`@erc/core-types`) should be maintained alongside the Rust crate, containing:
- All entity interfaces
- All DTO interfaces
- All enum types
- All error types
- All shared Zod schemas

Both applications import from `@erc/core-types`. This prevents divergence between the two apps' TypeScript type definitions.

---

## 5. Concept Ownership Matrix

| Concept | Budget App | Execution App | Shared Core |
|---|---|---|---|
| Project (root) | Creates/Owns | Reads/Enriches | ✅ |
| ProjectConfig | Creates/Owns | Read-only | ✅ |
| WorkPackage | Implicit | Explicit | ✅ (promote) |
| PersonnelRole | Creates/Owns | Reads | ✅ |
| Person (named individual) | ❌ | Creates/Owns | ❌ (execution only) |
| EquipmentItem | Creates/Owns | Reads | ✅ |
| Trip | Creates/Owns | Reads | ✅ |
| OtherDirectCostItem | Creates/Owns | Reads | ✅ |
| Subcontracting | Creates/Owns | Reads | ✅ |
| RateData | Reads | Reads | ✅ |
| AppError / FieldError | Uses | Uses | ✅ |
| Persistence (file format) | Creates | Extends | ✅ |
| Salary Calculation | Uses | Uses | ✅ |
| Depreciation Calculation | Uses | Uses | ✅ |
| Trip Cost Calculation | Uses | Uses | ✅ |
| Budget Aggregation | Uses | Uses | ✅ |
| CFS Logic | Uses | Tracks | ✅ |
| ReportingPeriod | ❌ | Creates/Owns | ❌ (execution only) |
| Milestone | ❌ | Creates/Owns | ❌ |
| Deliverable | ❌ | Creates/Owns | ❌ |
| Task | ❌ | Creates/Owns | ❌ |
| PersonMonthRecord | ❌ | Creates/Owns | ❌ |
| ActualCostEntry | ❌ | Creates/Owns | ❌ |
| RiskEntry / IssueEntry | ❌ | Creates/Owns | ❌ |
| Meeting / ActionItem | ❌ | Creates/Owns | ❌ |

---

## 6. Critical Design Decisions

### Decision 1: The Budget Application is the Source of Truth

The `.ercbudget` file is created and maintained by the Budget Application. The Execution Application reads it and attaches execution data. The Budget Application must never be required to understand execution data. This is a one-way dependency at the data level.

### Decision 2: WorkPackage Must Become an Explicit Entity

The current array-based WP representation must be promoted to a first-class entity. This is required for the Execution App to attach tasks, deliverables, and milestones to WPs. The Budget App can continue using its array-based internal representation but should expose `WorkPackage` structs through the shared domain.

### Decision 3: No Duplication of Calculation Logic

All calculation logic lives in `erc-core`. Neither `erc-budget` nor `erc-execution` implements its own calculation functions. If a new calculation is needed by the Execution App, it is added to `erc-core`.

### Decision 4: Decimal Arithmetic Is Non-Negotiable

All monetary values in both applications must use `rust_decimal::Decimal`. No exceptions. This constraint is enforced by the shared core types.

### Decision 5: The File Format Supports Both Apps

Rather than creating a separate `.ercexecution` file, execution data is stored as an optional `execution_data` block within the existing `.ercbudget` file format. The Budget App ignores this block. The Execution App reads and writes it. (Full specification in Phase 03.)

---

## 7. Open Questions

1. Should `erc-core` be a separate Git repository (true monorepo) or a workspace member of both app repositories?
2. Should the Shared Core Library have its own release cycle and semantic versioning, or be versioned together with the apps?
3. Is the `WorkPackage` promotion backward-compatible with existing `.ercbudget` files? (It should be if existing arrays are treated as the authoritative source during loading.)
4. Should the TypeScript shared types be code-generated from the Rust types (e.g., via `ts-rs` crate) to guarantee synchronization?

---

## 8. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Shared Core changes break Budget App | High | Semantic versioning; comprehensive integration tests in `erc-budget` against `erc-core` API |
| WorkPackage promotion introduces serialization incompatibility | High | Derive WP list from existing arrays during load; write migration tests against real `.ercbudget` files |
| TypeScript types diverge from Rust structs | Medium | Use `ts-rs` or a codegen step; CI gate on type mismatch |
| Both apps released at different cadences creating version skew | Medium | Lock both apps to the same `erc-core` version in CI |

---

## 9. Assumptions

- The Shared Core Library will be a Rust crate within a Cargo workspace that both applications reference
- Both applications will remain Tauri + Rust + React + TypeScript
- The initial Shared Core extraction will happen in Phase 07 (before Execution App implementation begins)

---

## 10. Confidence Level

**90%** — The shared/exclusive boundary is clear from the domain analysis. Minor uncertainty around `WorkPackage` promotion compatibility with existing files and the optimal TypeScript type-sharing mechanism.

---

## 11. Recommended Next Step

**Proceed to Phase 03 — .ercbudget File Specification.**

The file format must be fully specified and its extension strategy defined before the Execution Application can be architected. The format is the contract between both applications.
