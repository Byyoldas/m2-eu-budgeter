# ERC Budget Tool — Architecture Guide

**Version:** 1.0 (Final — as implemented)  
**Date:** 2026-07-10  
**Status:** Reflects the completed v1.0 implementation  
**Replaces:** TASK-07 architecture draft

---

> ## ⚠ Current Implementation Notes (as of v1.6.0, 2026-07-17)
>
> The layer architecture, data-flow cycle, state management, and persistence sections (§1-3, §6-7) are still accurate. The following have drifted since this was written:
>
> - **§4 (IPC Contract Summary)**: `set_subcontracting` now takes `{ amount_eur, work_package_id }` (a required WP tag was added). `add_cfs_item` now takes a real input, `{ amount_eur, work_package_ids }`, not "none" — it constructs a real C3 item rather than just flagging an existing one. `save_project` takes **no arguments** (saves to the already-known open path); it's `save_project_as(path)` that takes the path — this document's `save_project: { path: string }` row conflates the two. `get_project` (returns the current summary without a mutation) is missing from this table entirely.
> - **§8 (Bundled Rate Data)**: the file lives at `src-tauri/src/domain/rate_data.rs`, not `persistence/rate_data.rs` — there is no `persistence` submodule split, `persistence/mod.rs` is a single file. More importantly, **the JSON example flight bands/rates shown here were always fabricated placeholder numbers**, not the real EU Annex 2a/2b figures — this was discovered and fixed in v1.4.0 for all three bundled rate-version files (~195 countries each, transcribed from the actual EU source document). Don't use the numbers in this section for anything.
> - **§9 (Test Architecture)**: counts and the module breakdown are stale — as of v1.6.0 there are **186 Rust tests** (160 unit + 26 integration) and **116 TypeScript tests**. Notably absent from this table entirely: `calculation/wp_budget.rs` and the WP-allocation portion of `calculation/personnel_cost.rs` (CALC-20/CALC-20a, added after this document was written), `domain/rate_data.rs`'s structural + spot-check tests, and `src/export/excelExporter.test.ts` (which uses the `hyperformula` library to actually *evaluate* the Excel exporter's generated spreadsheet formulas, not just assert their text — the single most distinctive test in the whole suite, and worth reading if you're touching the Excel export).
> - **Not covered here at all**: the Work-Package-based budgeting model (see `docs/developer-guide.md` §8 for the accurate version) and the in-app auto-updater (`docs/developer-guide.md` §14, `docs/deployment-guide.md`).

---

## Contents

1. System Overview
2. Layer Architecture
3. Data Flow — End-to-End Request Cycle
4. IPC Contract Summary
5. Calculation Engine Pipeline
6. State Management
7. Persistence and File Format
8. Bundled Rate Data
9. Test Architecture
10. Key Architecture Decisions

---

## 1. System Overview

The ERC Budget Tool is a cross-platform desktop application built with **Tauri v2**. It runs on macOS 12+ and Windows 10+ with no network dependency. The application is distributed as a native installer (~10 MB) that embeds a small Rust binary and a Vite-compiled React frontend rendered in the OS's native webview.

```
┌──────────────────────────────────────────────────────────┐
│          USER'S DESKTOP (macOS / Windows)                │
│                                                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │             TAURI WINDOW                        │    │
│  │  ┌──────────────┐  ┌────────────────────────┐  │    │
│  │  │  LEFT PANEL  │  │     RIGHT PANEL         │  │    │
│  │  │  Wizard      │  │  Live Budget Dashboard  │  │    │
│  │  │  Steps       │  │  (Charts + Totals)      │  │    │
│  │  └──────────────┘  └────────────────────────┘  │    │
│  │      TypeScript / React (Vite / WebView)        │    │
│  │  ──────────────── IPC boundary ──────────────── │    │
│  │         Rust binary (Tauri commands)            │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
│  .ercbudget file on disk (JSON)                          │
└──────────────────────────────────────────────────────────┘
```

---

## 2. Layer Architecture

Dependencies flow strictly inward. Outer layers may depend on inner layers; inner layers have no knowledge of outer layers.

```
┌─────────────────────────────────────────────────────────────────┐
│  PRESENTATION LAYER                                             │
│  TypeScript 5 · React 18 · Zustand · React Hook Form · Zod     │
│  Recharts · Radix UI · ExcelJS · @react-pdf/renderer            │
│  src/                                                           │
├─────────────────────────────────────────────────────────────────┤
│  APPLICATION LAYER (IPC Command Handlers)                       │
│  Rust · Tauri #[tauri::command] functions                       │
│  src-tauri/src/commands/                                        │
├─────────────────────────────────────────────────────────────────┤
│  DOMAIN LAYER                                                   │
│  Rust structs and enums · rust_decimal · uuid                   │
│  src-tauri/src/domain/                                          │
├────────────────────┬───────────────────────────────────────────┤
│  CALCULATION       │  VALIDATION ENGINE                        │
│  ENGINE            │  Rust: src-tauri/src/validation/          │
│  Rust              │  TypeScript: src/validators/schemas.ts     │
│  src-tauri/src/    │                                           │
│  calculation/      │                                           │
├────────────────────┴───────────────────────────────────────────┤
│  PERSISTENCE LAYER                                             │
│  Rust · std::fs · tauri-plugin-fs · serde_json                 │
│  src-tauri/src/persistence/                                     │
│  Bundled EU rate data: src-tauri/resources/eu_travel_rates/    │
└─────────────────────────────────────────────────────────────────┘
```

### Layer responsibilities

**Presentation Layer** — renders UI; collects user inputs; calls the Application Layer via `invoke()`; displays `BudgetSummaryDto` received from the backend. Contains zero business logic. Never recalculates any financial value.

**Application Layer** — receives IPC calls; orchestrates validation, domain construction, calculation, and persistence; returns a fully-computed `BudgetSummaryDto` on every mutation. The only layer that touches shared application state (the in-memory `Project` and the `RateData`).

**Domain Layer** — defines entities (`Project`, `PersonnelRole`, `EquipmentItem`, `Trip`, `OtherDirectCostItem`, `Subcontracting`) as Rust structs. Enforces invariants in constructors (e.g., FTE must be positive). No I/O, no external dependencies.

**Calculation Engine** — pure functions operating on domain entities. All monetary arithmetic uses `rust_decimal::Decimal` (exact decimal, no floating-point rounding). No side effects.

**Validation Engine** — two tiers: TypeScript/Zod for instant field-level feedback in the browser; Rust for business-rule validation (cross-field, cross-entity) before any entity is persisted.

**Persistence Layer** — serialises/deserialises the `Project` entity tree to/from JSON (`.ercbudget` files). Loads bundled EU rate tables at application startup using `include_str!`.

---

## 3. Data Flow — End-to-End Request Cycle

The following sequence describes what happens when a user saves a new personnel role.

```
1. USER ACTION
   User completes the role form and clicks "Save Role".

2. FIELD VALIDATION (TypeScript / Zod — synchronous, in browser)
   personnelRoleSchema.parse(formData)
   → Field errors displayed immediately if any constraint fails.
   → If all fields pass, proceed.

3. IPC CALL (TypeScript → Rust)
   invoke('add_personnel_role', { role: dto })
   → JSON-serialised PersonnelRoleInput crosses the IPC boundary.

4. COMMAND HANDLER (Rust)
   fn add_personnel_role(role, project_state, rate_data) → Result<BudgetSummaryDto, AppError>

5. BUSINESS VALIDATION (Rust)
   validate_personnel_role(&role, &project)
   → Checks: label unique, only one PI, active years within project duration, inflation in range.
   → If any rule fails: return AppError::Validation(errors) → frontend shows field-level errors.

6. ENTITY CONSTRUCTION (Rust / Domain Layer)
   PersonnelRole::from_input(role)  →  PersonnelRole { id: Uuid::new_v4(), ... }

7. STATE MUTATION (Rust)
   project.personnel_roles.push(entity)

8. AUTO-SAVE (Rust / Persistence Layer)
   Serialise project to JSON → write to temp auto-save path.

9. BUDGET AGGREGATION (Rust / Calculation Engine)
   aggregate_budget(&project, &rate_data)
   → Runs the full CALC-19 pipeline:
      CALC-01: salary_EUR = monthly_TRY ÷ try_eur_rate
      CALC-02: salary_year_N = salary_EUR × (1 + inflation)^(N-1)
      CALC-03: annual_cost_year_N = salary_year_N × 12 × FTE (if active)
      CALC-04: role_total = Σ annual_cost_year_N
      CALC-05: equipment depreciation per item
      CALC-06: cap applied
      CALC-07 to CALC-12: trip costs (flight band + accommodation + subsistence + domestic)
      CALC-13: Category A = Σ personnel costs
      CALC-14: Category C1 = Σ trip costs
      CALC-15: Category C2 = Σ equipment depreciation
      CALC-16: Category C3 = Σ other direct costs
      CALC-17: Category B = subcontracting amount
      CALC-18: CFS threshold check (> €430,000)
      CALC-19: Category E = 25% × (A + C1 + C2 + C3); Total Direct = A+B+C1+C2+C3;
               Total Eligible = Direct + E; Requested EU Contribution = Total Eligible

10. DTO MAPPING (Rust → JSON)
    BudgetSummary → BudgetSummaryDto
    Decimal values rounded to 2 dp; serialised as strings.
    JSON crosses IPC boundary.

11. FRONTEND UPDATE (TypeScript)
    useProjectStore.setState({ summary: dto })
    → React re-renders: ring chart, bar chart, category totals panel.
    → Role card added to personnel list.
    → LivePreviewBox shows updated grand total.

Total elapsed time from click to updated UI: < 50 ms.
```

---

## 4. IPC Contract Summary

All commands are registered in `src-tauri/src/lib.rs` via `tauri::generate_handler![]`.

### Mutation commands (return `BudgetSummaryDto`)

| Command name | Input DTO | Notes |
|---|---|---|
| `create_project` | `ProjectConfigInput` | Initialises in-memory Project; returns initial (zero) summary |
| `update_project_config` | `ProjectConfigInput` | Replaces config; recalculates all costs |
| `add_personnel_role` | `PersonnelRoleInput` | |
| `update_personnel_role` | `{ id, role: PersonnelRoleInput }` | |
| `delete_personnel_role` | `{ id }` | |
| `add_equipment_item` | `EquipmentItemInput` | |
| `update_equipment_item` | `{ id, item: EquipmentItemInput }` | |
| `delete_equipment_item` | `{ id }` | |
| `add_trip` | `TripInput` (discriminated on `trip_kind`) | |
| `update_trip` | `{ id, trip: TripInput }` | |
| `delete_trip` | `{ id }` | |
| `add_other_cost` | `OtherCostInput` | |
| `update_other_cost` | `{ id, item: OtherCostInput }` | |
| `delete_other_cost` | `{ id }` | |
| `set_subcontracting` | `{ amount_eur: string }` | |
| `add_cfs_item` | none | Marks a C3 item as a CFS item; returns updated summary |
| `remove_cfs_item` | none | Unmarks a C3 item as a CFS item; returns updated summary |
| `dismiss_cfs_warning` | none | Dismisses the CFS prompt (user acknowledged); returns updated summary |

### Project lifecycle commands

| Command name | Input | Returns |
|---|---|---|
| `load_project` | `{ path: string }` | `ProjectDto` (full project + current summary) |
| `save_project` | `{ path: string }` | `void` |

### Preview commands (no state mutation)

| Command name | Input | Returns |
|---|---|---|
| `preview_role_cost` | `PersonnelRoleInput` | `RoleCostPreviewDto` (per-year costs) |
| `preview_equipment_depreciation` | `EquipmentItemInput` | `DepreciationPreviewDto` |
| `preview_trip_cost` | `TripInput` | `TripCostPreviewDto` |

### Reference data commands

| Command name | Input | Returns |
|---|---|---|
| `get_rate_versions` | none | `RateVersionSummary[]` |
| `get_countries` | `{ version_id: string }` | `CountryDto[]` |

### Error response format

```json
// Validation errors — mapped to form fields
{ "kind": "Validation", "detail": [
    { "field": "role_label", "code": "DUPLICATE_LABEL", "message": "This label is already in use." }
  ]
}

// No project loaded
{ "kind": "NoProject" }

// Calculation error
{ "kind": "Calculation", "detail": { "code": "DIVISION_BY_ZERO", "message": "..." } }

// Persistence error
{ "kind": "Persistence", "detail": "Failed to write file: ..." }

// Internal error (bug — should never reach production)
{ "kind": "Internal", "detail": "..." }
```

---

## 5. Calculation Engine Pipeline

All 19 calculation rules (CALC-01 to CALC-19) are implemented as pure Rust functions in `src-tauri/src/calculation/`.

| Rule | Module | Formula |
|---|---|---|
| CALC-01 | `salary_projection.rs` | `salary_eur = monthly_salary_try ÷ try_eur_rate` |
| CALC-02 | `salary_projection.rs` | `salary_year_N = salary_eur × (1 + inflation_rate ÷ 100)^(N-1)` |
| CALC-03 | `personnel_cost.rs` | `annual_cost_year_N = salary_year_N × 12 × fte` (if year is active) |
| CALC-04 | `personnel_cost.rs` | `role_total = Σ annual_cost_year_N over active years` |
| CALC-05 | `equipment_depreciation.rs` | `theoretical = (cost ÷ lifetime_months) × (usage_pct ÷ 100) × usage_months` |
| CALC-06 | `equipment_depreciation.rs` | `eligible = min(theoretical, cost × (usage_pct ÷ 100))` |
| CALC-07 | `trip_cost.rs` | Flight band lookup by one-way distance (km → F-01 to F-09; < 400km → €0) |
| CALC-08 | `trip_cost.rs` | `accommodation = nightly_rate_eur × number_of_nights` |
| CALC-09 | `trip_cost.rs` | `subsistence = daily_rate_eur × number_of_days` |
| CALC-10 | `trip_cost.rs` | `per_instance = flight + accommodation + subsistence + domestic_transport` |
| CALC-11 | `trip_cost.rs` | `trip_total = per_instance × number_of_instances` |
| CALC-12 | `trip_cost.rs` | Flat-amount trip: `trip_total = flat_amount × number_of_instances` |
| CALC-13 | `budget_aggregator.rs` | `category_a = Σ role_total over all roles` |
| CALC-14 | `budget_aggregator.rs` | `category_c1 = Σ trip_total over all trips` |
| CALC-15 | `budget_aggregator.rs` | `category_c2 = Σ eligible_depreciation over all equipment items` |
| CALC-16 | `budget_aggregator.rs` | `category_c3 = Σ amount_eur over all other cost items` |
| CALC-17 | `budget_aggregator.rs` | `category_b = subcontracting.amount_eur` |
| CALC-18 | `cfs_checker.rs` | CFS required if `requested_eu_contribution > 430,000`; four-state enum: `NotRequired` / `RequiredAndPresent` / `RequiredButDismissed` / `RequiredAndUnaddressed` |
| CALC-19 | `budget_aggregator.rs` | `indirect_base = A + C1 + C2 + C3`; `category_e = indirect_base × (rate ÷ 100)`; `total_direct = A + B + C1 + C2 + C3`; `total_eligible = total_direct + E`; `requested_eu = total_eligible` |

**Arithmetic contract:** all intermediate values carry full `Decimal` precision. Rounding to 2 decimal places occurs only during DTO mapping for display.

---

## 6. State Management

### Rust application state

A single `AppState` struct is managed by Tauri and injected into every command handler:

```rust
pub struct AppState {
    /// The currently open project. None until the user creates or loads one.
    pub project: Mutex<Option<Project>>,
    /// File-system path of the open .ercbudget file. None for unsaved new projects.
    pub project_path: Mutex<Option<std::path::PathBuf>>,
    /// EU travel rate tables loaded at startup. Read-only for the app's lifetime.
    pub rate_data: RateData,
}
```

`project` is wrapped in `Mutex<Option<…>>` because no project is loaded at startup. Commands that require a project lock the mutex and return `AppError::NoProject` if the `Option` is `None`.

### TypeScript application state (Zustand)

The Zustand store (`src/store/projectStore.ts`) holds only:
- The current screen (wizard step).
- The latest `BudgetSummaryDto` returned by the backend.
- The project file path (for display and Save operations).
- The project config (for display in the wizard header).
- UI state: `isLoading`, `isDirty`, `globalError`.

It does **not** hold the lists of roles, equipment, trips, or cost items. Those are embedded in `BudgetSummaryDto.role_detail`, `equipment_detail`, `trip_detail`, and retrieved from the store each render. This means the frontend never needs to manually synchronise its list state with the backend — the backend is always authoritative.

---

## 7. Persistence and File Format

Projects are saved as `.ercbudget` files (JSON). The file is human-readable and version-stamped.

```json
{
  "format_version": "1.0",
  "created_at": "2026-07-10T10:00:00Z",
  "updated_at": "2026-07-10T14:30:00Z",
  "project": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "config": {
      "project_title": "Topology in Turkish Academia",
      "pi_name": "Prof. Ayşe Demir",
      "call_reference": "ERC-2025-CoG",
      "duration_years": 5,
      "work_package_count": 3,
      "work_package_names": ["Fieldwork", "Analysis", "Dissemination"],
      "default_inflation_rate_pct": "20",
      "try_eur_rate": "38.50",
      "indirect_cost_rate_pct": "25",
      "rate_version_id": "v_from_2025_05_13",
      "call_opening_date": "2025-10-01"
    },
    "personnel_roles": [ ... ],
    "equipment_items": [ ... ],
    "trips": [ ... ],
    "other_cost_items": [ ... ],
    "subcontracting": { "amount_eur": "0", "cfs_acknowledged": false }
  }
}
```

**All `Decimal` values are stored as strings** (e.g., `"38.50"`, not `38.5`). This avoids JSON floating-point representation issues when values have exact decimal representations that cannot be expressed in IEEE 754.

**Auto-save path:** a platform-specific temp directory (resolved via Tauri's `app_local_data_dir()`). The auto-save file is named `erc_budget_autosave.ercbudget`. It is overwritten on every mutation.

**Schema migration:** when the `format_version` field is read, a migration function is called if the version does not match the current application version. This is a stub in v1.0 (no migrations needed yet) but the mechanism is in place.

---

## 8. Bundled Rate Data

The EU Annex 2a/2b travel rate tables are compiled into the application binary using Rust's `include_str!` macro. This makes the application fully offline — no network calls, no external files required.

```rust
// src-tauri/src/persistence/rate_data.rs
const RATE_FILES: &[&str] = &[
    include_str!("../../resources/eu_travel_rates/v_before_2024_07_31.json"),
    include_str!("../../resources/eu_travel_rates/v_2024_07_31_to_2025_05_12.json"),
    include_str!("../../resources/eu_travel_rates/v_from_2025_05_13.json"),
];
```

At application startup, each string is parsed into a `RateVersion` struct and stored in the immutable `RateData` state. Lookup at runtime is an in-memory hash map lookup by `version_id` → country code or flight band.

**Rate data JSON schema:**
```json
{
  "version_id": "v_from_2025_05_13",
  "version_label": "From 13 May 2025",
  "applicable_from": "2025-05-13",
  "flight_bands": [
    { "band_id": "F-01", "min_km": 400, "max_km": 999, "rate_eur": 180 },
    { "band_id": "F-02", "min_km": 1000, "max_km": 1999, "rate_eur": 275 },
    { "band_id": "F-03", "min_km": 2000, "max_km": 2999, "rate_eur": 429 },
    { "band_id": "F-04", "min_km": 3000, "max_km": 3999, "rate_eur": 545 },
    { "band_id": "F-05", "min_km": 4000, "max_km": 4999, "rate_eur": 674 },
    { "band_id": "F-06", "min_km": 5000, "max_km": 7999, "rate_eur": 857 },
    { "band_id": "F-07", "min_km": 8000, "max_km": 9999, "rate_eur": 1100 },
    { "band_id": "F-08", "min_km": 10000, "max_km": 14999, "rate_eur": 1363 },
    { "band_id": "F-09", "min_km": 15000, "max_km": null, "rate_eur": 1826 }
  ],
  "countries": [
    {
      "country_code": "AT",
      "country_name": "Austria",
      "accommodation_per_night_eur": 158,
      "subsistence_per_day_eur": 131
    }
  ]
}
```

---

## 9. Test Architecture

### Rust tests (142 total)

| File | Type | Count | What is tested |
|---|---|---|---|
| `calculation/salary_projection.rs` | Unit (inline) | 13 | CALC-01, CALC-02: TRY→EUR, year-N salary chain |
| `calculation/personnel_cost.rs` | Unit (inline) | 8 | CALC-03, CALC-04: active years, FTE scaling |
| `calculation/equipment_depreciation.rs` | Unit (inline) | 12 | CALC-05, CALC-06: depreciation formula, cap |
| `calculation/trip_cost.rs` | Unit (inline) | 22 | CALC-07 to CALC-12: all flight bands, itemised vs flat, domestic |
| `calculation/cfs_checker.rs` | Unit (inline) | 7 | CALC-18: threshold boundary, 3-state transitions |
| `calculation/budget_aggregator.rs` | Unit (inline) | 14 | CALC-13 to CALC-17: category totals, indirect base |
| `validation/mod.rs` | Unit (inline) | 41 | All 5 validators, field-level and entity-level errors |
| `tests/integration_test.rs` | Integration | 25 | CALC-19 full pipeline; 4 scenarios with workbook-derived expected values |

### TypeScript tests (~107 total)

| File | Type | Count | What is tested |
|---|---|---|---|
| `src/__tests__/validators.test.ts` | Unit | ~80 | All 10 Zod schemas, all field constraints and coercion |
| `src/__tests__/store.test.ts` | Unit | 27 | Zustand store: initial state, all mutations, reset, edge cases |

### Test infrastructure

- **`vitest.config.ts`** — jsdom environment, coverage thresholds (80%/75%/80%/80%), path alias `@` → `src/`.
- **`src/__tests__/setup.ts`** — mocks `@tauri-apps/api/core`'s `invoke` globally so TypeScript tests run without a live Tauri process.
- **Coverage scope** — `src/validators/**`, `src/store/**`, `src/hooks/**`, `src/ipc/**`, `src/export/**`.

---

## 10. Key Architecture Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Desktop framework | Tauri v2 | 10 MB bundle vs 150+ MB Electron; Rust calculation engine; TypeScript frontend |
| Calculation language | Rust | Exact decimal arithmetic via `rust_decimal`; type safety; compiled performance |
| Decimal serialisation | All monetary values as JSON strings | Avoids IEEE 754 representation loss across IPC boundary |
| Budget recalculation trigger | Every mutation → full `BudgetSummaryDto` returned | Frontend always has a consistent, up-to-date view; eliminates synchronisation bugs |
| Frontend state scope | Zustand holds only UI state and `BudgetSummaryDto` | Rust backend is the single source of truth for all domain data |
| Validation approach | Dual-layer (Zod field-level + Rust business rules) | Instant feedback without IPC latency; guaranteed server-side correctness |
| Rate data storage | Embedded JSON compiled into binary via `include_str!` | Fully offline; no network dependency; updatable by replacing JSON files |
| Persistence format | JSON with `.ercbudget` extension | Human-readable; version-stampable; easy schema migration |
| Export engine location | TypeScript (ExcelJS, @react-pdf/renderer) | Best library support for xlsx and PDF in the JS ecosystem |
| IPC wrapper module | All `invoke()` calls in `src/ipc/commands.ts` only | Centralised mocking point for tests; type-checked IPC surface |
| No business logic in frontend | Enforced by code review | Prevents drift between Rust calculation and frontend display values |
