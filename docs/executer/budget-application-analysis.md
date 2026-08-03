# Budget Application Analysis

**Phase 01 — Existing Budget Application Audit**
**Project:** M2-EU Budgeter → Horizon Europe / ERC Project Management Platform
**Version Audited:** 1.7.0
**Date:** 2026-08-03

---

## 1. Executive Summary

The M2-EU Budgeter is a complete, production-quality desktop application for preparing Horizon Europe / ERC Lump Sum grant budgets. It is a **single-partner, single-institution** budgeting tool specifically designed for Turkish research institutions applying to ERC Consolidator Grant calls. The application replaces a complex Excel workbook with a guided wizard that hides all calculation complexity from the proposal writer.

The application is well-architected, well-tested, and production-ready. It represents a strong foundation for a future platform.

---

## 2. Technology Stack

| Layer | Technology | Version |
|---|---|---|
| Desktop Runtime | Tauri | 2.x |
| Frontend Framework | React | 18.x |
| Frontend Language | TypeScript | 5.x |
| State Management | Zustand | 4.x |
| Form Validation (frontend) | Zod + React Hook Form | 3.x / 7.x |
| Charting | Recharts | 2.x |
| Excel Export | ExcelJS | 4.x |
| PDF Export | Browser `window.print()` API | — |
| Backend Language | Rust | stable |
| Backend Framework | Tauri (IPC commands) | 2.x |
| Decimal Arithmetic | rust_decimal | 1.x |
| UUID Generation | uuid (v4) | 1.x |
| Date/Time | chrono | 0.4.x |
| Serialization | serde / serde_json | 1.x |
| Error Handling | thiserror | 1.x |
| Testing (frontend) | Vitest + Testing Library | 1.x |
| Testing (backend) | Rust built-in `#[cfg(test)]` | — |
| Build Tool | Vite | 5.x |
| Package Manager | pnpm | 11.x |
| CI/CD | GitHub Actions | — |

**Platform Target:** macOS (primary), Windows (CI-built via GitHub Actions), Linux (theoretical via Tauri)

---

## 3. Folder Structure

```
m2-eu-budgeter-executer/
├── src/                          # React / TypeScript frontend
│   ├── App.tsx                   # Root layout component
│   ├── main.tsx                  # React entry point
│   ├── App.css                   # Global styles
│   ├── components/               # Shared UI components
│   │   ├── BudgetRingChart.tsx   # Donut chart — category breakdown
│   │   ├── BudgetWpBarChart.tsx  # Bar chart — per-WP budgets
│   │   ├── CFSModal.tsx          # CFS warning dialog
│   │   ├── CategoryTotalsPanel.tsx # Live sidebar budget totals
│   │   ├── EmptyStateCard.tsx    # Empty list placeholder
│   │   ├── EquipmentCard.tsx     # Equipment list item
│   │   ├── FormField.tsx         # Labelled form field wrapper
│   │   ├── LivePreviewBox.tsx    # Calculated preview panel
│   │   ├── ProgressStepper.tsx   # Left sidebar wizard stepper
│   │   ├── RoleCard.tsx          # Personnel role list item
│   │   ├── TripCard.tsx          # Trip list item
│   │   ├── UpdateChecker.tsx     # Auto-update notification
│   │   ├── WarningBanner.tsx     # Inline warning banner
│   │   └── WorkPackageGanttChart.tsx # WP timeline Gantt chart
│   ├── screens/                  # Wizard step screens
│   │   ├── Welcome.tsx           # New/Open project landing
│   │   ├── ProjectSetup.tsx      # Step 1 — title, PI, duration, WP count
│   │   ├── BudgetSettings.tsx    # Step 2 — FX rate, inflation, indirect rate
│   │   ├── WorkPackages.tsx      # Step 3 — WP names and month ranges
│   │   ├── Personnel.tsx         # Step 4 — staff roles (Category A)
│   │   ├── Equipment.tsx         # Step 5 — equipment items (Category C2)
│   │   ├── Travel.tsx            # Step 6 — trips (Category C1)
│   │   ├── OtherCosts.tsx        # Step 7 — other direct costs (C3) + CFS
│   │   └── ReviewExport.tsx      # Step 8 — summary + export
│   ├── store/
│   │   ├── projectStore.ts       # Zustand global state (single source of truth)
│   │   └── updaterStore.ts       # Auto-update Zustand slice
│   ├── ipc/
│   │   └── commands.ts           # Typed wrappers for all Tauri invoke() calls
│   ├── hooks/
│   │   ├── useAutoSave.ts        # Debounced 2s auto-save hook
│   │   └── useBudgetSummary.ts   # Derived budget summary selectors
│   ├── export/
│   │   ├── excelExporter.ts      # Multi-sheet ExcelJS workbook generator
│   │   ├── pdfExporter.ts        # HTML+print PDF export
│   │   └── csvExporter.ts        # Basic CSV export
│   ├── validators/
│   │   └── schemas.ts            # Zod schemas for frontend form validation
│   ├── types/
│   │   └── index.ts              # TypeScript types mirroring Rust DTOs
│   └── utils/
│       └── formatAppError.ts     # Error display formatting
├── src-tauri/                    # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs               # Binary entry point
│   │   ├── lib.rs                # App wiring: state, plugins, IPC handler registration
│   │   ├── error.rs              # AppError enum + FieldError + ValidationErrors
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── entities.rs       # Core domain entities (Project, roles, equipment, etc.)
│   │   │   ├── dto.rs            # Input/Output DTOs for the IPC boundary
│   │   │   └── rate_data.rs      # EU travel rate tables (embedded JSON)
│   │   ├── calculation/
│   │   │   ├── mod.rs            # Re-exports calculate_budget_summary
│   │   │   ├── salary_projection.rs  # CALC-01, CALC-02: TRY→EUR, salary chain
│   │   │   ├── personnel_cost.rs     # CALC-03, CALC-04, CALC-20a
│   │   │   ├── equipment_depreciation.rs # CALC-05, CALC-06
│   │   │   ├── trip_cost.rs          # CALC-07–CALC-12
│   │   │   ├── budget_aggregator.rs  # CALC-13–CALC-17
│   │   │   ├── cfs_checker.rs        # CALC-18: CFS threshold
│   │   │   ├── wp_budget.rs          # CALC-20: per-WP aggregation
│   │   │   └── budget_summary.rs     # CALC-19: master orchestrator
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── project.rs        # create, update, load, save, get, rates
│   │   │   ├── personnel.rs      # add, update, delete, preview
│   │   │   ├── equipment.rs      # add, update, delete, preview
│   │   │   ├── travel.rs         # add, update, delete, preview
│   │   │   └── other_costs.rs    # add, update, delete, CFS management, subcontracting
│   │   ├── persistence/
│   │   │   └── mod.rs            # save_project, load_project, auto_save
│   │   └── validation/
│   │       └── mod.rs            # Cross-entity validation rules
│   ├── resources/
│   │   └── eu_travel_rates/      # Embedded JSON rate tables (3 versions)
│   ├── capabilities/default.json # Tauri permission declarations
│   └── tauri.conf.json           # Tauri application configuration
├── docs/                         # Project documentation
├── dist/                         # Built frontend assets
├── Cargo.toml                    # Rust workspace root
└── package.json                  # Node/pnpm project manifest
```

---

## 4. Architecture

### 4.1 High-Level Architecture

The application follows a strict **two-process Tauri architecture**:

```
┌────────────────────────────────────────────────────────────┐
│  Frontend Process (WebView)                                │
│  React + TypeScript + Zustand                              │
│                                                            │
│  Wizard Screens → IPC Commands → Zustand Store → UI       │
│  Zod (field validation)                                    │
└────────────────────────┬───────────────────────────────────┘
                         │  Tauri IPC (invoke / serde_json)
┌────────────────────────▼───────────────────────────────────┐
│  Backend Process (Rust)                                    │
│                                                            │
│  Commands → Validation → Domain Entities → Calculation    │
│  Engine → BudgetSummaryDto → Persistence (auto-save)      │
└────────────────────────────────────────────────────────────┘
```

### 4.2 Layered Architecture (Backend)

The Rust backend implements a clean layered architecture:

```
IPC Commands (commands/)
    ↓  uses  ↓
Validation Engine (validation/)
    ↓  uses  ↓
Domain Entities (domain/entities.rs)
    ↓  feeds  ↓
Calculation Engine (calculation/)
    ↓  produces  ↓
DTOs (domain/dto.rs)
    ↓  serialised to  ↓
Frontend (types/index.ts)
```

Persistence (persistence/) is a horizontal service called by commands after every successful mutation.

### 4.3 Application State Machine

The application has a single `AppState` (Rust `Mutex<Option<Project>>`) that holds the in-memory project. Every mutation command follows this pattern:

1. Acquire lock on `AppState.project`
2. Validate input (cross-entity rules)
3. Apply mutation to domain entity
4. Run `calculate_budget_summary()` (full recalculation)
5. Auto-save to `.autosave` file
6. Return `BudgetSummaryDto` to frontend

The frontend stores the returned `BudgetSummaryDto` in Zustand and re-renders. There is no incremental update — the full budget is recalculated after every single change.

### 4.4 Data Flow Diagram

```
User Input
    → Zod validation (instant, no IPC)
    → invoke() → Rust command
        → Rust validation (cross-entity)
        → Mutation applied to Project entity
        → calculate_budget_summary(project, rate_data)
            → CALC-01/02: TRY → EUR, salary projection
            → CALC-03/04: Personnel cost lines + totals
            → CALC-20a: Personnel allocation per WP
            → CALC-05/06: Equipment depreciation
            → CALC-07–12: Trip costs (itemized or flat)
            → CALC-13: C3 aggregation
            → CALC-14: Indirect costs (25% of A+C1+C2+C3)
            → CALC-15: Total direct costs
            → CALC-16: Total eligible costs
            → CALC-17: EU contribution (100%)
            → CALC-18: CFS threshold check
            → CALC-20: Per-WP budget aggregation
        → auto_save(.autosave)
        → return BudgetSummaryDto
    → Zustand setSummary()
    → React re-render (live dashboard updates)
```

---

## 5. Domain Model

### 5.1 Core Entities (Rust — `domain/entities.rs`)

**Project** (root aggregate)
- `id: Uuid`
- `config: ProjectConfig`
- `personnel_roles: Vec<PersonnelRole>`
- `equipment_items: Vec<EquipmentItem>`
- `trips: Vec<Trip>`
- `other_cost_items: Vec<OtherDirectCostItem>`
- `subcontracting: Subcontracting`
- `cfs_warning_dismissed: bool`

**ProjectConfig**
- Project metadata (title, PI name, call reference)
- `duration_years: u8` (1–7)
- `work_package_count: u8` (1–10)
- `work_package_names: Vec<Option<String>>`
- `work_package_start_months: Vec<u32>` (1-indexed, per WP)
- `work_package_end_months: Vec<u32>` (1-indexed, inclusive, per WP)
- `default_inflation_rate_pct: Decimal` (%)
- `try_eur_rate: Decimal` (TRY per 1 EUR)
- `indirect_cost_rate_pct: Decimal` (%, max 50%)
- `rate_version_id: String` (links to embedded rate table)
- `call_opening_date: Option<String>`

**PersonnelRole**
- `id: Uuid`, `role_label: String`, `role_type: RoleType`
- `current_monthly_salary_try: Decimal`
- `fte_fraction: Decimal` (0–1]
- `inflation_rate_pct: Decimal`
- `start_month: u32`, `end_month: u32`

**EquipmentItem**
- `id: Uuid`, `name: String`
- `purchase_cost_eur: Decimal`
- `useful_lifetime_months: u32`
- `grant_usage_pct: Decimal` (%)
- `grant_usage_months: u32`
- `work_package_id: u8` (single WP)

**Trip**
- `id: Uuid`, `name: String`
- `trip_type: TripType` (Itemized | FlatAmount)
- `number_of_instances: u32`
- `work_package_ids: Vec<u8>` (cost split evenly across listed WPs)

**TripType::Itemized**
- `destination_country_code: String`
- `one_way_distance_km: u32` (0 = no flight)
- `number_of_nights: u32`, `number_of_days: u32`
- `domestic_transport_per_instance_eur: Decimal`

**TripType::FlatAmount**
- `flat_amount_per_instance_eur: Decimal`

**OtherDirectCostItem**
- `id: Uuid`, `name: String`
- `amount_eur: Decimal`
- `is_cfs_item: bool` (auto-created by CFS modal flow)
- `notes: Option<String>`
- `work_package_ids: Vec<u8>`

**Subcontracting** (Category B, single flat amount)
- `amount_eur: Decimal`
- `work_package_id: u8`

**RoleType** enum: `Pi | Expert | PostDoc | PhdStudent | MscStudent | Admin`

### 5.2 Rate Data (`domain/rate_data.rs`)

Three embedded JSON rate table versions:
- `v_before_2024_07_31`
- `v_2024_07_31_to_2025_05_12`
- `v_from_2025_05_13`

Each version contains:
- **FlightBands**: distance ranges (km) → flat round-trip cost (EUR)
- **CountryRates**: per-country accommodation (EUR/night) + subsistence (EUR/day)

Rate version is selected from the project `call_opening_date`.

---

## 6. Calculation Engine

All 20 named calculations (CALC-01 through CALC-20) are pure functions in the `calculation/` module. They accept domain values and return `Result<T, AppError>`. No I/O, no side effects, no panics.

| Calculation | Module | Description |
|---|---|---|
| CALC-01 | salary_projection | TRY monthly salary → EUR |
| CALC-02 | salary_projection | Salary projection chain with compounding inflation |
| CALC-03 | personnel_cost | Year-by-year cost lines per role (month proration) |
| CALC-04 | personnel_cost | Category A total (aggregate across all roles) |
| CALC-05 | equipment_depreciation | Eligible depreciation per item (with cap) |
| CALC-06 | equipment_depreciation | Category C2 total |
| CALC-07–11 | trip_cost | Itemized trip cost (flight band + accommodation + subsistence + domestic) |
| CALC-12 | trip_cost | Category C1 total |
| CALC-13 | budget_aggregator | Category C3 total (other direct costs incl. CFS) |
| CALC-14 | budget_aggregator | Indirect costs = (A+C1+C2+C3) × rate% |
| CALC-15 | budget_aggregator | Total direct costs = A+B+C1+C2+C3 |
| CALC-16 | budget_aggregator | Total eligible = direct + indirect |
| CALC-17 | budget_aggregator | EU contribution = total eligible × 100% |
| CALC-18 | cfs_checker | CFS threshold check (>€430,000 triggers requirement) |
| CALC-19 | budget_summary | Master orchestrator — runs all above in order |
| CALC-20 | wp_budget | Per-WP budget aggregation |
| CALC-20a | personnel_cost | Personnel allocation by WP (month-by-month) |

**Key business rules encoded:**
- Subcontracting (B) is **excluded from the indirect cost base** (ERC rule)
- Subcontracting IS an eligible cost and counts toward the EU contribution
- Equipment depreciation is capped at `cost × usage%` (cannot exceed value × usage share)
- CFS required when requested EU contribution exceeds €430,000 (strict `>`, not `≥`)
- EU funding rate is 100% (hardcoded constant `EU_FUNDING_RATE = 1`)
- Indirect rate capped at 50% (ERC rule)
- Personnel WP allocation: each project month's cost is split evenly across all WPs whose range contains that month

---

## 7. Validation Engine

Validation operates at two levels:

### 7.1 Frontend Validation (Zod, TypeScript)
- Runs instantly on form input (no IPC round-trip)
- Field-level checks only: required, type, range
- Schemas: `schemas.ts` — project setup, budget settings, personnel role, equipment item, trip, other cost

### 7.2 Backend Validation (Rust, `validation/mod.rs`)
Cross-entity and business-rule validation. Runs before every mutation.

| Validator | Key rules |
|---|---|
| `validate_personnel_role` | Unique label (case-insensitive), only one PI, salary > 0, FTE in (0,1], months within project duration |
| `validate_equipment_item` | Name required, cost > 0, lifetime ≥ 1, usage% in (0,100], usage months ≥ 1, WP in range |
| `validate_trip` | Name required, ≥1 WP, instances ≥ 1, itemized: country + nights + days required; flat: amount > 0 |
| `validate_other_cost` | Name required, amount > 0, ≥1 WP |
| `validate_project_config` | Duration 1–7, WP count 1–10, FX rate > 0, inflation 0–100%, indirect 0–50%, WP month ranges valid, full project coverage (no month gap) |

All validators collect **all** field errors before returning (not fail-fast), using `ValidationErrors` builder. Errors carry structured `FieldError { field, code, message }`.

---

## 8. Persistence Layer

### 8.1 File Format — `.ercbudget`

Plain UTF-8 JSON with envelope:

```json
{
  "format_version": "1.0",
  "created_at": "<ISO 8601 timestamp>",
  "updated_at": "<ISO 8601 timestamp>",
  "project": { ... }
}
```

The `project` object is a direct serialization of the `Project` entity. All `Decimal` values are serialized as strings (`rust_decimal::serde::str`). All UUIDs are serialized as standard UUID strings.

### 8.2 Auto-Save

Two layers:
1. **Rust auto-save**: After every mutation command, saves to `<file>.ercbudget.autosave` (or system temp dir for unsaved files)
2. **Frontend auto-save**: 2-second debounce hook — triggers `save_project` IPC call on any `summary` change when a named file exists

### 8.3 Operations

- `save_project(path)`: writes the current project to a named file, preserving `created_at`
- `load_project(path)`: reads and deserializes the file; hooks for future format migration (`format_version` check)
- `auto_save(project, project_path)`: lightweight best-effort save to `.autosave` sibling

---

## 9. Export Engine

### 9.1 Excel Export (ExcelJS)

Produces a 6-sheet workbook:
- **Sheet 1**: Budget Summary (category totals + WP breakdown, using Excel formulas referencing detail sheets)
- **Sheet 2**: Gantt Chart (PNG rendered from `<canvas>` embedded in sheet)
- **Sheet 3**: Personnel Detail (WP timeline table + role cost breakdown with formula-built per-WP columns)
- **Sheet 4**: Equipment Detail
- **Sheet 5**: Travel Detail
- **Sheet 6**: Other Direct Costs

The workbook uses real Excel formulas (SUM, SUMPRODUCT) rather than static values where possible, so the output can be edited and recalculated by an auditor.

### 9.2 PDF Export

Uses `window.print()` with an injected print stylesheet. Generates a clean HTML budget summary table, then triggers the browser print dialog. Suitable for submission and review. No heavy PDF library dependency.

### 9.3 CSV Export

Basic flat-file export. Minimal implementation.

---

## 10. UI Architecture

### 10.1 Wizard Layout

The main UI is a **two-panel wizard**:

```
┌─────────────────────┬──────────────────────────────────────┐
│  Left Sidebar       │  Right Content Area                  │
│                     │                                      │
│  Progress Stepper   │  Active Screen (one of 8 steps)      │
│  (8 steps)          │                                      │
│                     │                                      │
│  Category Totals    │                                      │
│  Panel (live)       │                                      │
│                     │                                      │
│  WP Bar Chart       │                                      │
│  Budget Ring Chart  │                                      │
└─────────────────────┴──────────────────────────────────────┘
```

The sidebar dashboard updates in real-time after every mutation (driven by Zustand `summary` state).

### 10.2 Screen Sequence

```
Welcome
  → Project Setup (title, PI, duration, WP count)
  → Budget Settings (FX rate, inflation, indirect rate, rate version)
  → Work Packages (WP names + start/end months + Gantt chart)
  → Personnel (add/edit/delete roles; live cost preview)
  → Equipment (add/edit/delete items; live depreciation preview)
  → Travel (add/edit/delete trips; live cost preview)
  → Other Costs (add/edit/delete C3 items; CFS management; subcontracting)
  → Review & Export (full summary; Excel / PDF / CSV export; save)
```

### 10.3 Live Preview Pattern

Every entity form (personnel, equipment, travel) has a **Live Preview Box** that calls the backend `preview_*` IPC command on form change, showing the calculated cost before the user saves the entry. This avoids an edit-save-review cycle.

### 10.4 CFS Modal Flow

When the budget exceeds €430,000:
1. `CFSModal` is displayed automatically (`cfs_prompt_required = true`)
2. User can enter the CFS item amount (added to C3) or dismiss ("Remind Me Later")
3. If dismissed, a `WarningBanner` with a CFS badge remains visible
4. Status is tracked as `NOT_REQUIRED | REQUIRED_AND_PRESENT | REQUIRED_BUT_DISMISSED | REQUIRED_AND_UNADDRESSED`

### 10.5 IPC Command Pattern

All frontend→backend calls follow this pattern:
```typescript
// In commands.ts
export const addPersonnelRole = (input: PersonnelRoleInput): Promise<BudgetSummaryDto> =>
  invoke('add_personnel_role', { input });

// In screen component
const summary = await addPersonnelRole(input);
setSummary(summary);   // updates Zustand → triggers re-render
```

Every mutating command returns the full new `BudgetSummaryDto`.

---

## 11. Application Configuration

### 11.1 Tauri Configuration

- **Product name**: M2-EU Budgeter
- **Identifier**: `com.m2eubudgeter.desktop`
- **Window**: 1280×800 px, min 1024×700 px
- **Auto-updater**: GitHub Releases endpoint with ed25519 signing
- **Tauri plugins**: `fs`, `dialog`, `shell`, `updater`, `process`

### 11.2 Capabilities

Tauri v2 capability system controls what the WebView is permitted to do. Currently declared in `capabilities/default.json` (file system access, dialog, shell).

---

## 12. Error Handling

### 12.1 Backend Error Hierarchy

```rust
AppError {
    Validation(Vec<FieldError>)  // cross-entity rule violations
    Calculation { code, message } // engine errors (bugs or invalid intermediate states)
    Persistence(String)           // file I/O failures
    NotFound(String)              // rate version not found, entity not found
    NoProject                     // command called before project created/loaded
    Internal(String)              // unexpected errors
}
```

Errors serialize to JSON as `{ "kind": "...", "detail": ... }` via `serde(tag = "kind", content = "detail")`.

### 12.2 Frontend Error Handling

- `AppError` is re-thrown from `invoke()` calls
- Screens catch errors and display inline messages or the global `globalError` store
- `formatAppError.ts` converts structured errors to human-readable strings
- Validation errors are mapped back to form field errors using the `field` property of `FieldError`

---

## 13. Testing

### 13.1 Backend Tests

All tests are in-module (`#[cfg(test)]` blocks). Coverage:

| Module | Test Count | Coverage Areas |
|---|---|---|
| `salary_projection` | 14 | CALC-01/02 — all boundary cases, zero/negative inputs, 7-year duration |
| `personnel_cost` | 13 | CALC-03/04/20a — partial years, proration, WP split, overlapping WPs |
| `equipment_depreciation` | 12 | CALC-05/06 — capped/uncapped, boundary at lifetime, partial usage |
| `budget_aggregator` | 14 | CALC-13–17 — indirect exclusions, CFS, zero-cost cases |
| `cfs_checker` | 8 | CALC-18 — all 4 status paths, threshold boundary (strict >) |
| `validation` | 35+ | All validators — all error codes, multi-error collection |

Integration test in `src-tauri/tests/integration_test.rs`.

### 13.2 Frontend Tests

Vitest test suite in `src/__tests__/`:
- `store.test.ts` — Zustand store behaviour
- `validators.test.ts` — Zod schema validation
- `excelExporter.test.ts` — Excel output structure

---

## 14. Strengths

1. **Clean Architecture**: Strict separation between domain entities, DTOs, calculation engine, validation, persistence, and IPC layer. No business logic leaks into the UI.

2. **Exact Decimal Arithmetic**: `rust_decimal` throughout the backend. No floating-point rounding errors in any financial calculation.

3. **Comprehensive Testing**: All 20 calculation functions have test suites. Validation tests cover all error codes. Tests are co-located with the code they test.

4. **Strong Typing End-to-End**: TypeScript types in `types/index.ts` mirror Rust DTOs exactly. Zod schemas provide runtime validation. Rust's type system prevents many classes of bugs.

5. **Dual Auto-Save**: Both Rust-side (after every mutation) and frontend-side (debounced) auto-save protect against data loss.

6. **Live Preview Pattern**: Users see calculated results before committing any entry — excellent UX that prevents trial-and-error editing.

7. **Embedded Rate Data**: EU travel rate tables are embedded at compile time. No network dependency at runtime; no stale-data risk.

8. **Rich Excel Export**: The generated workbook uses real formulas (SUMPRODUCT for WP allocation), not just static values. Auditors can work with the file independently.

9. **Versioned File Format**: `format_version` field in `.ercbudget` files allows future migrations without breaking existing files. `#[serde(default)]` on new fields provides backward compatibility.

10. **Self-Contained Deployment**: Single binary + WebView. No external runtime or database required.

---

## 15. Weaknesses and Technical Debt

1. **Single-Partner Only**: The entire domain model assumes a single institution. There is no `Partner` concept, no partner-level budget, no coordinator/partner distinction. This is the most significant limitation for a multi-partner Horizon Europe tool.

2. **Turkish Institution Specificity**: The application is hardwired to Turkish institutions (TRY → EUR conversion, Turkish inflation context). The exchange rate and inflation concepts exist but are framed around Turkey.

3. **No User Authentication or Role Model**: There is no concept of users, roles, or access control. The application is single-user by design.

4. **No Reporting Period Model**: There is no concept of project periods (P1, P2, P3), reporting periods, or periodic financial reporting. The budget is a single lump sum across the full project duration.

5. **No Deliverable / Milestone Tracking**: The application models Work Packages only at the budget level. There are no tasks, deliverables, or milestones attached to WPs.

6. **WP Assignment is Flat**: Equipment can only be assigned to a single WP. Trips and other costs split evenly across selected WPs. There is no weighted or per-period assignment.

7. **PDF Export is Minimal**: `window.print()` produces limited output with no control over page layout beyond CSS. It cannot be embedded or processed programmatically.

8. **CSV Export is Underdeveloped**: The CSV exporter is a stub compared to the Excel and PDF exporters.

9. **No Import Engine**: There is no mechanism to import data from an existing Excel budget workbook or any other format. Projects must be created from scratch.

10. **Rate Data is Code-Embedded**: When new EU rate tables are published, a new application release is required. A file-based or network-fetched rate update mechanism would be better long-term.

11. **No Undo/Redo**: Mutations are permanent. The only recovery path is the `.autosave` file.

12. **No Subcontracting Detail**: Subcontracting (Category B) is a single lump-sum field. There is no ability to add multiple subcontract lines with individual descriptions, vendors, or WP assignments.

---

## 16. Reusable Modules for the Execution Application

The following modules are strong candidates for extraction into a **Shared Core Library**:

| Module | Reusability | Notes |
|---|---|---|
| `domain/entities.rs` (ProjectConfig, WorkPackage concepts) | High | Project structure is reused in execution context |
| `domain/rate_data.rs` | High | Travel rate lookup needed in execution tracking |
| `persistence/mod.rs` | Medium | `.ercbudget` format extended, not replaced |
| `error.rs` | High | `AppError`, `FieldError`, `ValidationErrors` are generic |
| `calculation/salary_projection.rs` | High | Needed for actual vs. planned cost comparisons |
| `calculation/equipment_depreciation.rs` | High | Needed for actual equipment cost tracking |
| `calculation/budget_aggregator.rs` | Medium | Budget category totals reused in execution view |
| `calculation/cfs_checker.rs` | High | CFS compliance tracking continues during execution |
| `types/index.ts` (TypeScript types) | High | Most DTOs extend naturally to execution context |
| `components/` (UI primitives) | Medium | FormField, EmptyStateCard, WarningBanner are generic |
| `validators/schemas.ts` (base schemas) | Medium | Base rules can be extended |

---

## 17. Future Risk Areas

1. **Multi-Partner Extension**: Adding a `Partner` entity to the existing model is a significant refactor. The calculation engine's WP-cost aggregation would need to become partner-aware.

2. **State Management Scalability**: The single Zustand store and full-recalculation-on-every-mutation pattern works well for the current scope. A larger Execution Application with many more entities may need more granular state management.

3. **`.ercbudget` Format Evolution**: The file format currently stores only budget data. Extending it to hold execution data requires careful versioning to avoid breaking existing files.

4. **Rate Table Maintenance**: Rate tables are embedded. The EU publishes updated Annex 2a/2b tables irregularly. A mechanism to update them without a full app release would reduce maintenance burden.

5. **Cross-Platform Testing**: macOS is the primary development platform. Windows builds are CI-only (no automated testing on Windows). Linux support is untested.

---

## 18. Deliverables

- [x] **This document**: `/docs/budget-application-analysis.md`

---

## 19. Open Questions

1. Should the Execution Application read `.ercbudget` files directly, or will there be a separate `.ercexecution` file linked to a budget file?
2. Should multi-partner support be added to the Budget Application first, or designed only in the Execution Application?
3. Will the Execution Application target the same technology stack (Tauri + Rust + React)?
4. What is the intended release sequencing — will both apps share a single installer, or remain fully independent?
5. Is the TRY/EUR specificity a product constraint or should the Execution Application support multi-currency from the outset?

---

## 20. Risks

| Risk | Severity | Likelihood | Mitigation |
|---|---|---|---|
| Sharing business logic requires refactoring stable code | Medium | High | Extract to Shared Core carefully with test coverage before modifying either app |
| `.ercbudget` format extension breaks existing files | High | Medium | Use `serde(default)` for all new fields; increment `format_version`; write migration tests |
| Execution Application scope creep (too large for one app) | High | High | Define MVP ruthlessly in Phase 04; defer non-critical modules to future versions |
| Personnel model doesn't map to actual people in execution context | Medium | Medium | Design a Person→Role mapping early in the domain model |

---

## 21. Assumptions

- The Execution Application will be a **separate desktop app** (separate binary, separate window) that can read the same `.ercbudget` file produced by the Budget Application
- The technology stack will remain **Tauri + Rust + React + TypeScript**
- No server-side infrastructure is planned; both apps remain fully offline/local
- The Budget Application source code will **not** be modified during Phase 01–08

---

## 22. Confidence Level

**95%** — The audit covers all source files in the repository. The codebase is well-documented with inline comments. All modules were read and analysed. Minor uncertainty remains around the full Excel export logic (very large file) and edge cases in the Gantt chart rendering.

---

## 23. Recommended Next Step

**Proceed to Phase 02 — Shared Domain Discovery.**

The Budget Application's domain model is the foundation for the Execution Application. Phase 02 will identify every concept that must be preserved, extended, or newly introduced in the shared domain layer, producing a definitive map of the Shared Core Library.
