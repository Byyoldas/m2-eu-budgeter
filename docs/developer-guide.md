# M2-EU Budgeter — Developer Guide

**Version:** 1.6.0
**Applies to:** M2-EU Budgeter v1.6.0 (Tauri v2 + Rust + React/TypeScript)
**Audience:** Engineers maintaining or extending the application
**Date:** 2026-07-17

> This guide describes the application **as it actually exists in the repository today**. Earlier planning documents in this folder (`project-overview.md`, `excel-analysis.md`, `business-rules.md`, `domain-model.md`, `calculation-engine.md`, `input-catalog.md`, `ux-design.md`, `development-plan.md`, `architecture.md`) were written *before* implementation and describe an initial design that has since diverged in a few significant ways — most importantly, cost categories are now organised by **Work Package (WP)** instead of **project year**. See the "Current Implementation Notes" callout near the top of each of those documents for what changed. This guide and `deployment-guide.md` are kept in sync with the code and can be trusted as-is.

---

## Contents

1. Prerequisites and Environment Setup
2. Repository Layout
3. Running the Application in Development
4. Running the Test Suite
5. Frontend Architecture (TypeScript / React)
6. Backend Architecture (Rust)
7. The IPC Contract
8. Work-Package-Based Budgeting (the core architectural decision)
9. Adding a New Budget Category
10. Adding a New Rate Table Version
11. Adding a New Validation Rule
12. Updating the Calculation Engine
13. The Excel Export Engine
14. The Auto-Updater
15. Code Style and Conventions
16. Dependency Management

---

## 1. Prerequisites and Environment Setup

| Tool | Version | Purpose |
|---|---|---|
| Rust (stable) | ≥ 1.78 | Backend compilation |
| Node.js | ≥ 20 LTS | Frontend toolchain |
| pnpm | ≥ 9 | Package management (the project uses **pnpm**, not npm — a `pnpm-lock.yaml` is committed) |
| Tauri CLI v2 | 2.11.x | Development server + bundler (installed as a dev dependency, invoked via `pnpm tauri`) |
| poppler (`pdftotext`/`pdfinfo`) | any | Only needed if you're transcribing a new EU rate table from a PDF source document — not required to build or run the app |

**Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-apple-darwin aarch64-apple-darwin   # macOS
rustup target add x86_64-pc-windows-msvc                     # Windows (cross)
```

**Install frontend dependencies:**
```bash
cd erc-budget
pnpm install
```

**Verify Tauri CLI:**
```bash
pnpm tauri --version   # tauri-cli 2.11.x
```

**macOS additional requirement:** Xcode Command Line Tools (`xcode-select --install`).

**Windows additional requirement:** Microsoft C++ Build Tools (Visual Studio Build Tools 2022 with "Desktop development with C++"). In practice this project builds Windows installers via GitHub Actions (`.github/workflows/windows-build.yml`), not locally — see `deployment-guide.md`.

---

## 2. Repository Layout

```
erc-budget/
├── src/                              # TypeScript / React frontend
│   ├── screens/                      # One file per wizard step, in wizard order
│   │   ├── Welcome.tsx               # New/Open project + manual "Check for Updates"
│   │   ├── ProjectSetup.tsx
│   │   ├── BudgetSettings.tsx
│   │   ├── WorkPackages.tsx          # Defines WP count + Start/End Month per WP (Gantt chart)
│   │   ├── Personnel.tsx             # Runs AFTER WorkPackages — WP timelines must exist first
│   │   ├── Equipment.tsx
│   │   ├── Travel.tsx
│   │   ├── OtherCosts.tsx            # C3 items + Subcontracting (B) + CFS
│   │   └── ReviewExport.tsx
│   ├── components/
│   │   ├── ProgressStepper.tsx
│   │   ├── CategoryTotalsPanel.tsx
│   │   ├── BudgetWpBarChart.tsx      # Per-WP bar chart (NOT per-year — renamed from BudgetYearBarChart)
│   │   ├── BudgetRingChart.tsx
│   │   ├── WorkPackageGanttChart.tsx
│   │   ├── RoleCard.tsx / EquipmentCard.tsx / TripCard.tsx   # List-item cards with Edit + Delete
│   │   ├── CFSModal.tsx
│   │   ├── WarningBanner.tsx
│   │   ├── EmptyStateCard.tsx
│   │   ├── LivePreviewBox.tsx
│   │   ├── FormField.tsx
│   │   └── UpdateChecker.tsx         # Silent background update check + "Update Available" modal
│   ├── store/
│   │   ├── projectStore.ts           # Zustand store — screen, summary, projectConfig, rate data
│   │   └── updaterStore.ts           # Shared updater-check state (used by UpdateChecker + Welcome's manual button)
│   ├── hooks/
│   │   ├── useBudgetSummary.ts       # Wraps an IPC mutation: loading/error state + store update
│   │   └── useAutoSave.ts
│   ├── validators/
│   │   └── schemas.ts                # All Zod schemas
│   ├── ipc/
│   │   └── commands.ts               # Typed wrappers around invoke() — see §7
│   ├── export/
│   │   ├── excelExporter.ts          # ExcelJS, formula-based Personnel sheet — see §13
│   │   ├── excelExporter.test.ts     # HyperFormula-based formula-evaluation tests
│   │   ├── csvExporter.ts
│   │   └── pdfExporter.ts            # window.print() + print stylesheet, not a PDF library
│   ├── types/
│   │   └── index.ts                  # TypeScript types mirroring Rust DTOs (kept in lockstep by hand)
│   ├── utils/
│   │   └── formatAppError.ts
│   ├── App.tsx                       # Root component / screen router / STEP_ORDER
│   ├── App.css                       # All application styles (single global stylesheet)
│   └── main.tsx
│
├── src-tauri/                        # Rust backend
│   ├── src/
│   │   ├── lib.rs                    # Crate root; registers plugins, AppState, invoke_handler
│   │   ├── main.rs
│   │   ├── commands/                 # Application layer (IPC handlers), one file per category
│   │   │   ├── mod.rs
│   │   │   ├── project.rs            # create/update/load/save project, get_rate_versions, get_countries
│   │   │   ├── personnel.rs
│   │   │   ├── equipment.rs
│   │   │   ├── travel.rs
│   │   │   └── other_costs.rs        # C3 items, subcontracting, CFS
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── entities.rs           # Project, PersonnelRole, EquipmentItem, Trip, etc.
│   │   │   ├── dto.rs                # Input/Detail DTOs (JSON-serialisable across IPC)
│   │   │   └── rate_data.rs          # EU rate table loading + lookup (RateData, RateVersion, CountryRate)
│   │   ├── calculation/              # Calculation Engine — see §12
│   │   │   ├── mod.rs
│   │   │   ├── salary_projection.rs  # CALC-01/02: TRY→EUR, yearly inflation compounding
│   │   │   ├── personnel_cost.rs     # CALC-03/04/20a: cost lines + month-overlap WP allocation
│   │   │   ├── equipment_depreciation.rs
│   │   │   ├── trip_cost.rs
│   │   │   ├── budget_aggregator.rs  # Direct/indirect/eligible/EU-contribution totals
│   │   │   ├── wp_budget.rs          # CALC-20: per-Work-Package budget aggregation
│   │   │   ├── budget_summary.rs     # CALC-19: orchestrates every calculation in dependency order
│   │   │   └── cfs_checker.rs
│   │   ├── validation/
│   │   │   └── mod.rs                # All field/entity validators; ValidationErrors/FieldError
│   │   ├── persistence/
│   │   │   └── mod.rs                # .ercbudget file I/O (JSON, format_version'd) + auto-save
│   │   └── error.rs                  # AppError (Validation / NoProject / Persistence / Internal)
│   ├── resources/
│   │   └── eu_travel_rates/          # Bundled EU Annex 2a/2b rate tables (embedded via include_str!)
│   │       ├── v_before_2024_07_31.json
│   │       ├── v_2024_07_31_to_2025_05_12.json
│   │       └── v_from_2025_05_13.json     # current tier as of this writing
│   ├── capabilities/
│   │   └── default.json              # Tauri v2 ACL — dialog/shell/updater/process/fs permissions
│   ├── tests/
│   │   └── integration_test.rs       # End-to-end CALC-19 tests against a full Project fixture
│   ├── Cargo.toml
│   └── tauri.conf.json               # Bundle config + plugins.updater (pubkey, endpoints)
│
├── scripts/
│   └── generate-latest-json.mjs      # Builds the updater's latest.json manifest for a release — see deployment-guide.md
│
├── src/__tests__/
│   ├── setup.ts                      # Vitest global setup (Tauri IPC mock)
│   ├── validators.test.ts
│   └── store.test.ts
│
├── .github/workflows/
│   └── windows-build.yml             # Builds + signs the Windows installer on every push to main
│
├── package.json
├── vite.config.ts
├── vitest.config.ts
└── tsconfig.json
```

Two things worth calling out explicitly because they differ from what you might expect from a typical Tauri starter:

- There is **no `commands/export.rs`**. All export formats (Excel, CSV, PDF) run entirely client-side in TypeScript against the `BudgetSummaryDto` already in the Zustand store — the Rust backend has no knowledge of export formats at all.
- There is **no separate `persistence/project_file.rs` / `persistence/rate_data.rs` split** — both concerns live in the single `persistence/mod.rs` (file I/O) and `domain/rate_data.rs` (rate table loading) respectively.

---

## 3. Running the Application in Development

**Start the development server (hot-reload on both frontend and backend changes):**
```bash
cd erc-budget
pnpm tauri dev
```

This starts Vite in watch mode, compiles the Rust backend, and launches the Tauri window pointing at the Vite dev server (`http://localhost:5173`, per `tauri.conf.json`'s `build.devUrl`). Frontend changes hot-reload; Rust changes trigger a recompile and restart.

**Frontend-only development (no Tauri window, browser-based):**
```bash
pnpm dev
```
IPC calls to Rust will fail (no Tauri bridge), so screens depending on backend data show empty/error states. Useful for pure layout/CSS work only.

**Inspect the Tauri webview:** right-click → **Inspect Element** for WebKit (macOS) / WebView2 (Windows) DevTools.

---

## 4. Running the Test Suite

**Rust tests (unit + integration, 186 total as of v1.6.0):**
```bash
cd erc-budget/src-tauri
cargo test
```

Individual modules:
```bash
cargo test calculation::personnel_cost   # one module's tests
cargo test --test integration_test       # only the end-to-end integration tests
```

**TypeScript tests (Vitest, 116 total as of v1.6.0):**
```bash
cd erc-budget
pnpm test                    # single run
pnpm test:watch              # watch mode
pnpm test:coverage           # coverage report (opens HTML in ./coverage/)
```

Coverage thresholds (enforced by `vitest.config.ts`, only over `validators/`, `store/`, `hooks/`, `ipc/`, `export/` — screens/components are excluded from the coverage gate):

| Metric | Threshold |
|---|---|
| Lines | 80% |
| Branches | 75% |
| Functions | 80% |
| Statements | 80% |

**Running everything before a release:**
```bash
cd erc-budget/src-tauri && cargo test && cd .. && pnpm tsc --noEmit && pnpm vitest run && pnpm build
```

There is no ESLint or Prettier configuration in this repository at the time of writing — formatting is by convention (match surrounding code), not tool-enforced on the TypeScript side. Rust formatting/linting (`cargo fmt`, `cargo clippy`) is available via the standard toolchain but is not wired into CI.

---

## 5. Frontend Architecture (TypeScript / React)

### Store (`src/store/projectStore.ts`)

The Zustand store is the single source of truth for UI state:

```typescript
interface ProjectStore {
  screen: Screen;
  summary: BudgetSummaryDto | null;
  projectPath: string | null;
  projectConfig: ProjectConfigInput | null;
  rateVersions: RateVersionSummary[];
  countries: CountrySummary[];
  isLoading: boolean;
  isDirty: boolean;
  globalError: AppError | null;
  // ...setters for each field
}
```

The store does **not** hold lists of personnel roles, equipment items, trips, or cost items as separately-managed arrays. Those come back inside `BudgetSummaryDto.role_detail`, `equipment_detail`, `trip_detail`, `other_cost_detail`, and `wp_budgets` after every mutation — the Rust backend is the single source of truth for all domain data, and the frontend only ever mirrors the latest `BudgetSummaryDto`.

A separate, smaller `src/store/updaterStore.ts` holds the auto-updater's check state (`update`, `result`, `currentVersion`) so the silent background check (`UpdateChecker`, mounted globally in `App.tsx`) and the manual "Check for Updates" button (`Welcome.tsx`) share one code path and one modal instead of duplicating the download/install flow.

### IPC Layer (`src/ipc/commands.ts`)

All communication with the Rust backend goes through typed wrapper functions in this one module — no other module calls `invoke()` directly. See §7 for the full command list.

### Validators (`src/validators/schemas.ts`)

All Zod schemas live here, performing field-level validation before the IPC call. Field-level backend errors (`AppError.kind === "Validation"`) are matched back to a specific form field by name via each screen's `fieldError(field)` helper, which checks both the backend's `fieldErrors` and the local Zod `errors` object.

### Types (`src/types/index.ts`)

All TypeScript types mirroring Rust DTOs live here. **There is no code generation** — when a Rust DTO in `domain/dto.rs` changes, the corresponding TypeScript interface must be updated by hand. This is the single most common source of bugs in this codebase (see the "recurring bug pattern" callouts in `future-extensions.md`): forgetting to add a new DTO field on the TypeScript side, or getting a `#[serde(rename = ...)]`/casing mismatch wrong.

### Screens and Edit Forms

Each screen manages its own form state with React Hook Form + a Zod resolver. A screen with an editable list (Personnel, Equipment, Travel, OtherCosts) follows the same `mode: 'list' | 'add' | 'edit'` pattern: an `openAdd()` resets the form to blank defaults, an `openEdit(item)` resets the form to the item's *current* values, and `onSubmit` calls either the `add*` or `update*` IPC command depending on whether an item is being edited.

**Important invariant when adding a new editable field:** the backend's `*DetailDto` for that entity must carry every raw input value the add-form collects, not just computed/derived output values — otherwise `openEdit()` has nothing to populate that field with and it silently comes back empty. This exact bug (Equipment/Travel/Personnel edit forms losing previously-entered values) was fixed in v1.6.0 by adding the missing raw fields to `EquipmentItemDetailDto` and `TripDetailDto`.

---

## 6. Backend Architecture (Rust)

### Crate Configuration (`src-tauri/Cargo.toml`)

```toml
[lib]
name = "erc_budget_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

The `rlib` type is essential — it lets `tests/integration_test.rs` import `erc_budget_lib` as an ordinary dependency.

### Application State

```rust
pub struct AppState {
    pub project: Mutex<Option<Project>>,
    pub project_path: Mutex<Option<std::path::PathBuf>>,
    pub rate_data: RateData,   // loaded once at startup, immutable for the app's lifetime
}
```

### Command Handler Pattern

```rust
#[tauri::command]
pub fn add_personnel_role(
    state: State<'_, AppState>,
    input: PersonnelRoleInputDto,
) -> Result<BudgetSummaryDto, AppError> {
    let mut lock = state.project.lock().unwrap();
    let project = lock.as_mut().ok_or(AppError::NoProject)?;

    validate_personnel_role(&input, project)?;
    let entity = /* construct PersonnelRole from input */;
    project.personnel_roles.push(entity);

    // auto-save, then recompute the full summary
    calculate_budget_summary(project, &state.rate_data)
}
```

Pattern: validate → construct entity → mutate the in-memory `Project` → auto-save → recalculate the entire `BudgetSummaryDto` → return it. **Every** mutation command recalculates and returns the full summary; there is no incremental/partial recalculation anywhere in the app.

### Error Handling

All errors are `AppError`, serialising to one of:
```json
{ "kind": "Validation", "detail": [{ "field": "role_label", "code": "DUPLICATE_LABEL", "message": "..." }] }
{ "kind": "NoProject" }
{ "kind": "Persistence", "detail": "..." }
{ "kind": "Internal", "detail": "..." }
```
The frontend maps `Validation` to per-field form errors and everything else to global error banners.

### Decimal Arithmetic

All money uses `rust_decimal::Decimal`, serialised across IPC as strings (`#[serde(with = "rust_decimal::serde::str")]`) — never `f64`. Rounding to 2 decimal places happens only at DTO-mapping/display time; internal calculations retain full precision.

---

## 7. The IPC Contract

Every mutation command follows: **Frontend → Backend**: a JSON payload wrapped in `{ input: ... }` (or `{ id, input }` for updates); **Backend → Frontend (success)**: the full recalculated `BudgetSummaryDto`; **Backend → Frontend (error)**: an `AppError` JSON object.

**Full command list (from `src-tauri/src/lib.rs`'s `invoke_handler!` and `src/ipc/commands.ts`):**

| Command | Frontend wrapper | Returns |
|---|---|---|
| `create_project` | `createProject(config)` | `BudgetSummaryDto` |
| `update_project_config` | `updateProjectConfig(config)` | `BudgetSummaryDto` |
| `load_project` | `loadProject(path)` | `BudgetSummaryDto` |
| `save_project` | `saveProject()` | `void` |
| `save_project_as` | `saveProjectAs(path)` | `void` |
| `get_project` | `getProject()` | `BudgetSummaryDto` |
| `get_rate_versions` | `getRateVersions()` | `RateVersionSummary[]` |
| `get_countries` | `getCountries(versionId)` | `CountrySummary[]` |
| `add_personnel_role` | `addPersonnelRole(input)` | `BudgetSummaryDto` |
| `update_personnel_role` | `updatePersonnelRole(id, input)` | `BudgetSummaryDto` |
| `delete_personnel_role` | `deletePersonnelRole(id)` | `BudgetSummaryDto` |
| `preview_role_cost` | `previewRoleCost(input)` | `RoleCostPreviewDto` |
| `add_equipment_item` | `addEquipmentItem(input)` | `BudgetSummaryDto` |
| `update_equipment_item` | `updateEquipmentItem(id, input)` | `BudgetSummaryDto` |
| `delete_equipment_item` | `deleteEquipmentItem(id)` | `BudgetSummaryDto` |
| `preview_equipment_depreciation` | `previewEquipmentDepreciation(input)` | `EquipmentPreviewDto` |
| `add_trip` | `addTrip(input)` | `BudgetSummaryDto` |
| `update_trip` | `updateTrip(id, input)` | `BudgetSummaryDto` |
| `delete_trip` | `deleteTrip(id)` | `BudgetSummaryDto` |
| `preview_trip_cost` | `previewTripCost(input)` | `TripCostPreviewDto` |
| `add_other_cost` | `addOtherCost(input)` | `BudgetSummaryDto` |
| `update_other_cost` | `updateOtherCost(id, input)` | `BudgetSummaryDto` |
| `delete_other_cost` | `deleteOtherCost(id)` | `BudgetSummaryDto` |
| `add_cfs_item` | `addCfsItem(amountEur, workPackageIds)` | `BudgetSummaryDto` |
| `remove_cfs_item` | `removeCfsItem()` | `BudgetSummaryDto` |
| `dismiss_cfs_warning` | `dismissCfsWarning()` | `BudgetSummaryDto` |
| `set_subcontracting` | `setSubcontracting(amountEur, workPackageId)` | `BudgetSummaryDto` |

**Rule: never add a frontend-only recalculation.** If you need a new total or derived value, add it to `BudgetSummaryDto` and compute it in the Rust calculation engine (`budget_summary.rs` or one of the `calculation/*.rs` modules it orchestrates), not in the frontend.

---

## 8. Work-Package-Based Budgeting (the core architectural decision)

The single most important fact about this codebase, and the thing every one of the pre-implementation design docs in this folder gets wrong: **cost categories are organised by Work Package, not by project year.**

- `ProjectConfig` has `work_package_count`, `work_package_names`, `work_package_start_months`, `work_package_end_months` (1-indexed project months, inclusive) — not years.
- Personnel roles have a `start_month`/`end_month` charging period (not a set of "active years" or a manually-picked WP list). Their cost is allocated to Work Packages **automatically**, month-by-month: `calculation/personnel_cost.rs::allocate_personnel_cost_by_wp` walks every month of the role's charging period, finds every WP whose own `[start_month, end_month]` contains that month, and splits that month's (inflation-adjusted) salary evenly across all of them.
- `validate_project_config` enforces that Work Packages **collectively cover the entire project duration** with no gaps (overlaps are fine — that's exactly the even-split case above). This exists specifically so a personnel-active month can never fall outside every WP and silently vanish from the per-WP budget view while still counting toward the Category A total.
- Equipment, Travel, and Other Direct Cost items are tagged to WP(s) directly by the user (`work_package_id` for Equipment — single, required; `work_package_ids` for Travel/OtherCosts — one or more, cost split evenly if more than one).
- `calculation/wp_budget.rs` aggregates all five categories (Personnel, Equipment, Travel, Other Costs, Subcontracting) into one `WpBudgetDto` per Work Package, which drives the Review & Export screen's per-WP table and the Excel export's per-WP columns.

Because of this, the **Work Packages screen runs before Personnel** in the wizard (`App.tsx`'s `STEP_ORDER`) — WP timelines must exist before a Personnel role's automatic WP allocation has anything to allocate against.

If you're implementing something described in `business-rules.md`, `domain-model.md`, `calculation-engine.md`, or `input-catalog.md` that mentions "project year" or "active years" for any category other than personnel salary inflation compounding, treat it as superseded by this section.

---

## 9. Adding a New Budget Category

Suppose you need to add **Category D: Exceptional Costs**.

1. **Domain entity** (`domain/entities.rs`): add an `ExceptionalCostItem` struct with `id`, `name`, `amount_eur: Decimal`, `work_package_ids: Vec<u8>` (follow the OtherDirectCostItem pattern — WP-tagged, not year-tagged), and a `Vec<ExceptionalCostItem>` field on `Project`.
2. **DTOs** (`domain/dto.rs`): an `ExceptionalCostInputDto` and `ExceptionalCostItemDetailDto` (make sure the detail DTO includes every field the input form collects — see §5's edit-form invariant). Add `category_d_total: Decimal` and `exceptional_cost_detail: Vec<...>` to `BudgetSummaryDto`.
3. **Validation** (`validation/mod.rs`): `validate_exceptional_cost_item(...)`.
4. **Calculation**: sum items into `category_d_total` (a new function in `calculation/budget_aggregator.rs` or a new module, following the C3 pattern in `budget_aggregator.rs`). Decide whether Category D belongs in the indirect-cost base (`calculate_indirect_costs`) and in `calculate_total_eligible_costs` — don't assume; confirm the EU rule.
5. **WP aggregation** (`calculation/wp_budget.rs`): thread Category D items through `aggregate_wp_budgets` the same way Other Costs are (WP-tagged, split evenly across multiple WPs).
6. **Commands** (`commands/exceptional_costs.rs`, new file): `add_exceptional_cost`, `update_exceptional_cost`, `delete_exceptional_cost`, following `commands/other_costs.rs`.
7. **Register** the new commands in `lib.rs`'s `invoke_handler!`.
8. **Frontend types** (`types/index.ts`): mirror every new DTO field exactly.
9. **Zod schema** (`validators/schemas.ts`): `exceptionalCostSchema`.
10. **IPC wrappers** (`ipc/commands.ts`): `addExceptionalCost`, etc.
11. **UI**: a new `ExceptionalCosts.tsx` screen (or extend `OtherCosts.tsx` if it's conceptually close to C3), added to `STEP_ORDER` in `App.tsx` and the stepper.
12. **Excel/CSV/PDF exporters** (`src/export/*.ts`): add a sheet/section, and a column in the Budget Summary / WP breakdown table.
13. **Tests**: inline `#[cfg(test)]` tests for the new calculation function(s), a Rust integration test exercising the full pipeline, and Zod schema tests in `validators.test.ts`.

---

## 10. Adding a New Rate Table Version

When the European Commission publishes a new Annex 2a/2b rate update:

1. Create `src-tauri/resources/eu_travel_rates/v_from_YYYY_MM_DD.json`, matching the schema of the existing files (`version_id`, `valid_from`, `valid_until`, `description`, `flight_bands[]`, `countries[]` — see `domain/rate_data.rs`'s `RateVersion`/`FlightBand`/`CountryRate` structs for exact field names and the `#[serde(rename = ...)]` mappings). Every version must include an `"OTHER"` fallback country entry.
2. Add it to the `include_str!` list in `RateData::load_embedded()` (`domain/rate_data.rs`).
3. Add a structural smoke test and at least one spot-check test in `domain/rate_data.rs`'s `#[cfg(test)] mod tests` pinning a couple of known values (see `test_current_version_matches_official_source_values` for the pattern) — this is what would have caught the fact that the originally-shipped rate tables were fabricated placeholder data rather than the real EU figures (fixed in v1.4.0).
4. **Transcribe carefully.** If you're extracting the table from a PDF, install `poppler` (`brew install poppler`) for `pdftotext -layout`, and prefer scripting the JSON construction (see the pattern used for the v1.4.0 fix) over hand-typing ~200 country rows — it's far too easy to introduce a transcription error, and GitHub's asset-name space→dot sanitization and similar surprises are easier to catch with an automated diff than by eye.

No other code changes are required — `get_rate_versions` dynamically returns every loaded version and the frontend's Rate Table Version dropdown picks it up automatically.

---

## 11. Adding a New Validation Rule

All Rust validators live in `validation/mod.rs`, built on a shared `ValidationErrors`/`FieldError` pattern (see `validate_project_config`, `validate_personnel_role`, etc. for examples). To add a rule:

1. Add the check and `errors.push(FieldError::new(field, code, message))` call to the relevant validator function.
2. Add a `#[cfg(test)]` test in the same file using the existing `has_field_error(...)` helper.
3. If the rule is reachable from an "add" or "edit" form, confirm the frontend's `fieldError(field)` lookup in that screen will surface the new code against the right input (it matches on the `field` string, so as long as the Zod schema's field name matches, no frontend change is usually needed).

---

## 12. Updating the Calculation Engine

Each `calculation/*.rs` module has inline `#[cfg(test)]` tests. When changing a formula:

1. Write/update the test first to describe the new expected behaviour — include a hand-computed worked example in a comment (see `personnel_cost.rs`'s tests for the style: exact numbers, not just "should be greater than").
2. Implement the change.
3. Run `cargo test`.
4. Check whether `tests/integration_test.rs` needs a new or updated scenario — it exercises the full `calculate_budget_summary` pipeline against a realistic fixture project, which is what catches cross-module regressions unit tests miss.
5. If the change affects Personnel cost, also check whether the Excel exporter's `SUMPRODUCT`-based formulas in `excelExporter.ts` (which independently replicate the WP-allocation algorithm as spreadsheet formulas — see §13) need the same change, and re-run the HyperFormula-based tests in `excelExporter.test.ts`.

**Key rule:** never use `f64` for money — always `rust_decimal::Decimal`.

---

## 13. The Excel Export Engine

`src/export/excelExporter.ts` is the most structurally complex file in the frontend. It builds a multi-sheet workbook (Budget Summary, Gantt Chart, Personnel, Equipment, Travel, Other Direct Costs) using ExcelJS, and — unlike the CSV/PDF exporters — several of its cells are **live formulas**, not static values:

- The Personnel sheet's per-Work-Package cost cells are `SUMPRODUCT` formulas over a hidden `_WPMonthHelper` sheet that replicate the backend's `allocate_personnel_cost_by_wp` month-by-month overlap-and-split algorithm entirely in spreadsheet formula form, including yearly inflation compounding (`(1+rate/100)^ROUNDUP(month/12,0)`) and even-splitting across simultaneously-active WPs.
- The Personnel sheet's "Work Package Timelines" table also computes each role's **Person-Months (PM)** per WP as an FTE-weighted formula (`months × FTE`), plus a live Total PM / Employment Months / Reconciled? check block, so the allocation is auditable directly in the spreadsheet.
- The Budget Summary sheet's category totals are `SUM(...)` formulas referencing the detail sheets, not copied numbers — editing a detail sheet's inputs and letting Excel recalculate should reproduce the same totals the app computed.

Because of the formulas, **`excelExporter.test.ts` doesn't just assert formula text** — it uses `hyperformula` (a spreadsheet-formula evaluation library, dev dependency) to actually evaluate the generated workbook and assert on the resulting numbers, cross-checked against hand-derived expected values. This is the pattern to follow for any future change to the formula-generating code: assert on evaluated output, not just that a formula string was produced.

---

## 14. The Auto-Updater

M2-EU Budgeter checks GitHub Releases for a newer signed version on launch (silently, via `UpdateChecker.tsx`) and on demand (via the Welcome screen's "Check for Updates" button, both backed by `src/store/updaterStore.ts`). This is powered by `tauri-plugin-updater` / `@tauri-apps/plugin-updater`, configured in `tauri.conf.json`'s `plugins.updater` (a public key + the `https://github.com/<owner>/<repo>/releases/latest/download/latest.json` endpoint).

This is purely a release-process concern from a day-to-day development standpoint — you don't need to think about it unless you're cutting a release (see `deployment-guide.md`) or touching `UpdateChecker.tsx`/`updaterStore.ts` directly. One thing worth knowing if a local build starts failing: `tauri.conf.json` has `bundle.createUpdaterArtifacts: true`, which means **every** `pnpm tauri build` — not just official releases — requires `TAURI_SIGNING_PRIVATE_KEY` (and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""`) to be set in the environment, or the build fails outright with "no private key". See `deployment-guide.md` for where that key lives.

---

## 15. Code Style and Conventions

**Rust:**
- No `unwrap()` on `Result`/`Option` in production code — use `?`, `ok_or()`, or an explicit `match`.
- No `f64` for monetary values — always `Decimal`.
- Test function names: `test_<what>_<condition>_<expected>`, e.g. `test_calc_20a_split_across_two_wps_by_month_count`.
- `cargo fmt`/`cargo clippy` are available but not CI-enforced in this repo — use your judgement, match surrounding style.

**TypeScript:**
- No raw `invoke()` calls outside `src/ipc/commands.ts`.
- IPC wrapper functions: `<verb><Entity>`, e.g. `addPersonnelRole`.
- Zod schemas: `<entity>Schema`, e.g. `personnelRoleSchema`.
- No ESLint/Prettier config exists in this repo — match surrounding formatting by hand.

---

## 16. Dependency Management

**Rust (`src-tauri/Cargo.toml`):**

| Crate | Purpose |
|---|---|
| `tauri` | Desktop framework (IPC, window management) |
| `tauri-plugin-dialog` | Native file open/save dialogs |
| `tauri-plugin-fs` | File system read/write |
| `tauri-plugin-shell` | Opening external URLs in the OS default browser |
| `tauri-plugin-updater` | Auto-update check/download/install |
| `tauri-plugin-process` | App relaunch after an update installs |
| `serde` + `serde_json` | JSON serialisation of DTOs |
| `rust_decimal` + `rust_decimal_macros` | Exact decimal arithmetic (macros are dev-only, for test fixtures) |
| `uuid` | Entity ID generation |
| `chrono` | Timestamps (auto-save, rate version dates) |
| `thiserror` | `AppError` |
| `tokio` | Async runtime (Tauri's requirement) |

Add a Rust dependency with `cargo add <crate>` in `src-tauri/`, then verify `cargo build` succeeds.

**TypeScript (`package.json`):**

| Package | Purpose |
|---|---|
| `react` + `react-dom` | UI framework |
| `zustand` | State management |
| `react-hook-form` + `@hookform/resolvers` | Form handling |
| `zod` | Schema validation |
| `recharts` | Ring chart + per-WP bar chart |
| `exceljs` | Excel export (with live formulas — see §13) |
| `@radix-ui/*` | Accessible UI primitives |
| `@tauri-apps/api`, `@tauri-apps/plugin-{dialog,fs,shell,updater,process}` | Tauri JS bindings |
| `uuid` | Client-side optimistic ID generation |
| `hyperformula` (dev) | Formula-evaluation tests for the Excel exporter — see §13 |
| `vitest` + `@vitest/coverage-v8` (dev) | Test runner + coverage |
| `jsdom` (dev) | DOM environment for Vitest |

There is no `@react-pdf/renderer` dependency — the PDF exporter (`src/export/pdfExporter.ts`) deliberately avoids a PDF library and uses the browser's `window.print()` with a dedicated print stylesheet instead.

Add an npm dependency with `pnpm add <package>` (or `pnpm add -D <package>` for dev-only) in the repo root.
