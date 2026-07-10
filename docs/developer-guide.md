# ERC Budget Tool — Developer Guide

**Version:** 1.0  
**Applies to:** ERC Budget Tool v1.0  
**Audience:** Engineers maintaining or extending the application  
**Date:** 2026-07-10

---

## Contents

1. Prerequisites and Environment Setup
2. Repository Layout
3. Running the Application in Development
4. Running the Test Suite
5. Frontend Architecture (TypeScript / React)
6. Backend Architecture (Rust)
7. The IPC Contract
8. Adding a New Budget Category
9. Adding a New Rate Table Version
10. Adding a New Validation Rule
11. Updating the Calculation Engine
12. Adding a New Export Format
13. Code Style and Conventions
14. Dependency Management

---

## 1. Prerequisites and Environment Setup

| Tool | Version | Purpose |
|---|---|---|
| Rust (stable) | ≥ 1.78 | Backend compilation |
| Node.js | ≥ 20 LTS | Frontend toolchain |
| npm | ≥ 10 | Package management |
| Tauri CLI v2 | ≥ 2.0 | Development server + bundler |
| VS Code (recommended) | any | Editor with `rust-analyzer` + `ESLint` extensions |

**Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-apple-darwin aarch64-apple-darwin   # macOS
rustup target add x86_64-pc-windows-msvc                     # Windows (cross)
```

**Install Node dependencies:**
```bash
cd erc-budget
npm install
```

**Verify Tauri CLI:**
```bash
npx tauri --version   # should print: tauri-cli 2.x.x
```

**macOS additional requirement:** Xcode Command Line Tools must be installed (`xcode-select --install`).

**Windows additional requirement:** Microsoft C++ Build Tools (Visual Studio Build Tools 2022 with "Desktop development with C++").

---

## 2. Repository Layout

```
erc-budget/
├── src/                            # TypeScript / React frontend
│   ├── screens/                    # One file per wizard step
│   │   ├── Welcome.tsx
│   │   ├── ProjectSetup.tsx
│   │   ├── BudgetSettings.tsx
│   │   ├── WorkPackages.tsx
│   │   ├── Personnel.tsx
│   │   ├── Equipment.tsx
│   │   ├── Travel.tsx
│   │   ├── OtherCosts.tsx
│   │   └── ReviewExport.tsx
│   ├── components/                 # Shared UI components
│   │   ├── FormField.tsx
│   │   ├── RoleCard.tsx
│   │   ├── EquipmentCard.tsx
│   │   ├── TripCard.tsx
│   │   ├── CostItemCard.tsx
│   │   ├── LivePreviewBox.tsx
│   │   ├── EmptyStateCard.tsx
│   │   ├── WarningBanner.tsx
│   │   ├── BudgetRingChart.tsx
│   │   ├── BudgetYearBarChart.tsx
│   │   ├── CategoryTotalsPanel.tsx
│   │   └── CFSModal.tsx
│   ├── store/
│   │   └── projectStore.ts         # Zustand store
│   ├── hooks/
│   │   ├── useBudgetSummary.ts     # Subscribes to live backend summary
│   │   └── useAutoSave.ts          # Triggers save on valid state change
│   ├── validators/
│   │   └── schemas.ts              # All Zod schemas
│   ├── ipc/
│   │   └── commands.ts             # Typed wrappers around invoke()
│   ├── export/
│   │   ├── excelExporter.ts
│   │   ├── pdfExporter.ts
│   │   └── csvExporter.ts
│   ├── types/
│   │   └── index.ts                # TypeScript types mirroring Rust DTOs
│   ├── App.tsx                     # Root component / screen router
│   ├── App.css                     # All application styles
│   └── main.tsx                    # React entry point
│
├── src-tauri/                      # Rust backend
│   ├── src/
│   │   ├── lib.rs                  # Crate root; registers commands + state
│   │   ├── main.rs                 # Tauri application entry point
│   │   ├── commands/               # Application Layer (IPC handlers)
│   │   │   ├── mod.rs
│   │   │   ├── project.rs
│   │   │   ├── personnel.rs
│   │   │   ├── equipment.rs
│   │   │   ├── travel.rs
│   │   │   ├── other_costs.rs
│   │   │   └── export.rs
│   │   ├── domain/                 # Domain entities and value types
│   │   │   ├── mod.rs
│   │   │   └── entities.rs
│   │   ├── calculation/            # Calculation Engine
│   │   │   ├── mod.rs
│   │   │   ├── salary_projection.rs
│   │   │   ├── personnel_cost.rs
│   │   │   ├── equipment_depreciation.rs
│   │   │   ├── trip_cost.rs
│   │   │   ├── budget_aggregator.rs
│   │   │   └── cfs_checker.rs
│   │   ├── validation/             # Validation Engine (Rust side)
│   │   │   └── mod.rs
│   │   ├── persistence/            # File I/O + rate data loading
│   │   │   ├── mod.rs
│   │   │   ├── project_file.rs
│   │   │   └── rate_data.rs
│   │   ├── domain/
│   │   │   ├── dto.rs              # Data Transfer Objects (JSON-serialisable)
│   │   └── error.rs                # AppError type
│   ├── resources/
│   │   └── eu_travel_rates/        # Bundled EU Annex 2a/2b rate tables
│   │       ├── v_before_2024_07_31.json
│   │       ├── v_2024_07_31_to_2025_05_12.json
│   │       └── v_from_2025_05_13.json
│   ├── tests/
│   │   └── integration_test.rs     # CALC-19 end-to-end integration tests
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/__tests__/                  # TypeScript test suite
│   ├── setup.ts                    # Vitest global setup (Tauri mock)
│   ├── validators.test.ts          # ~80 Zod schema tests
│   └── store.test.ts               # 27 Zustand store tests
│
├── package.json
├── vite.config.ts
├── vitest.config.ts
└── tsconfig.json
```

---

## 3. Running the Application in Development

**Start the development server (hot-reload on both frontend and backend changes):**
```bash
cd erc-budget
npm run tauri dev
```

This command:
1. Starts Vite in watch mode (TypeScript/React frontend).
2. Compiles the Rust backend with `cargo build`.
3. Launches the Tauri window pointing at `http://localhost:1420`.

Frontend changes (`.tsx`, `.css`) hot-reload without restarting the Rust process. Rust changes trigger a Cargo recompile and restart.

**Frontend-only development (no Tauri window, browser-based):**
```bash
npm run dev
```
Opens `http://localhost:1420` in your browser. IPC calls to Rust will fail (the Tauri bridge is absent), so all screens that depend on backend data will show empty states. Useful for pure UI layout work.

**Inspect the Tauri webview:**
Right-click anywhere in the app window and choose **Inspect Element** to open WebKit / WebView2 DevTools. Console, Network, and Elements panels are all available.

---

## 4. Running the Test Suite

**Rust tests (unit + integration, 142 total):**
```bash
cd erc-budget/src-tauri
cargo test
```

Individual modules:
```bash
cargo test calculation::salary_projection   # run one module's tests
cargo test --test integration_test          # run only integration tests
```

**TypeScript tests (Vitest, ~107 total):**
```bash
cd erc-budget
npm test                    # single run
npm run test:watch          # watch mode
npm run test:coverage       # coverage report (opens HTML in ./coverage/)
```

**Coverage thresholds** (enforced by Vitest):

| Metric | Threshold |
|---|---|
| Lines | 80% |
| Branches | 75% |
| Functions | 80% |
| Statements | 80% |

The build fails if any threshold is not met on `npm run test:coverage`.

**Running all tests before a release:**
```bash
cd erc-budget/src-tauri && cargo test && cd .. && npm run test:coverage
```

---

## 5. Frontend Architecture (TypeScript / React)

### Store (`src/store/projectStore.ts`)

The Zustand store is the single source of truth for all UI state. It holds:

```typescript
interface ProjectState {
  screen: Screen;                    // current wizard step
  summary: BudgetSummaryDto | null;  // latest backend calculation result
  projectPath: string | null;        // path to the saved .ercbudget file
  projectConfig: ProjectConfigInput | null;
  rateVersions: RateVersionSummary[];
  isLoading: boolean;
  isDirty: boolean;
  globalError: AppError | null;
}
```

All mutations go through store actions (`setScreen`, `setSummary`, `setProjectPath`, etc.). Components call store actions; they never mutate state directly.

The store does **not** hold lists of personnel roles, equipment items, trips, or cost items. Those are owned by the Rust backend and returned inside `BudgetSummaryDto.role_detail`, `equipment_detail`, `trip_detail` after every mutation. This is a deliberate architectural choice: the Rust backend is the single source of truth for all domain data.

### IPC Layer (`src/ipc/commands.ts`)

All communication with the Rust backend goes through typed wrapper functions in this module. Example:

```typescript
export async function addPersonnelRole(
  role: PersonnelRoleInput
): Promise<BudgetSummaryDto> {
  return invoke<BudgetSummaryDto>('add_personnel_role', { role });
}
```

No other module calls `invoke()` directly. This ensures:
- IPC calls are type-checked at compile time.
- Mocking in tests requires patching only one module.
- Renaming a Tauri command requires a change in exactly one place.

### Validators (`src/validators/schemas.ts`)

All Zod schemas live here. They perform field-level validation on form inputs before the IPC call. The schemas mirror the Rust `ValidationError` codes so that backend errors can be mapped back to specific fields.

### Types (`src/types/index.ts`)

All TypeScript types that mirror Rust DTOs are defined here. When the Rust `dto/mod.rs` is updated, the corresponding TypeScript type in `types/index.ts` must be updated to match.

### Screens and Components

Each screen file manages its own form state using React Hook Form + Zod resolvers. Screens call IPC functions, update the Zustand store with the returned `BudgetSummaryDto`, and render the appropriate components.

The right-panel dashboard components (`BudgetRingChart`, `BudgetYearBarChart`, `CategoryTotalsPanel`) subscribe to `summary` from the Zustand store and re-render automatically on every change.

---

## 6. Backend Architecture (Rust)

### Crate Configuration (`src-tauri/Cargo.toml`)

```toml
[lib]
name = "erc_budget_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

The `rlib` type is essential — it allows the integration tests in `tests/integration_test.rs` to import `erc_budget_lib` as a dependency.

### Application State

A single `AppState` struct is registered in `lib.rs` and injected into every command handler:

```rust
pub struct AppState {
    pub project: Mutex<Option<Project>>,               // current in-memory project
    pub project_path: Mutex<Option<PathBuf>>,          // open file path (None for unsaved)
    pub rate_data: RateData,                           // immutable EU rate tables
}
```

Handlers extract what they need: `state.project.lock()` for mutations, `&state.rate_data` for lookups.

### Command Handlers

Each IPC command is a `#[tauri::command]` function. The pattern is:

```rust
#[tauri::command]
pub fn add_personnel_role(
    role: PersonnelRoleInput,
    project_state: State<'_, ProjectState>,
    rate_data: State<'_, RateDataState>,
) -> Result<BudgetSummaryDto, AppError> {
    let mut project = project_state.0.lock().unwrap();
    let project = project.as_mut().ok_or(AppError::NoProject)?;

    validate_personnel_role(&role, project)?;
    let entity = PersonnelRole::from_input(role)?;
    project.personnel_roles.push(entity);

    auto_save(project)?;
    let summary = aggregate_budget(project, &rate_data.0)?;
    Ok(summary.into_dto())
}
```

The pattern is: validate → construct entity → mutate project → auto-save → recalculate → return DTO.

### Error Handling

All errors are returned as `AppError`, which serialises to a JSON object:

```json
{
  "kind": "Validation",
  "detail": [
    { "field": "role_label", "code": "DUPLICATE_LABEL", "message": "This label is already used." }
  ]
}
```

```json
{ "kind": "NoProject" }
```

```json
{ "kind": "Persistence", "detail": "Failed to write auto-save file: ..." }
```

The frontend maps `AppError` back to form errors (`kind: "Validation"`) or global error banners (`kind: "Persistence"`, `"Internal"`).

### Decimal Arithmetic

All monetary values use `rust_decimal::Decimal`. The `dec!()` macro (from `rust_decimal_macros`) is used in test fixtures. Conversion from user string inputs uses `Decimal::from_str()` with explicit error handling — no `unwrap()` on money values.

Rounding occurs exactly once: in the DTO mapping, where amounts are rounded to 2 decimal places for display. Internal calculations carry full precision.

---

## 7. The IPC Contract

Every mutation command follows this contract:

| Direction | Format |
|---|---|
| Frontend → Backend | JSON payload matching the `*Input` DTO struct |
| Backend → Frontend (success) | `BudgetSummaryDto` (full recalculated summary) |
| Backend → Frontend (error) | `AppError` JSON object |

**Full command list:**

| Command | Input | Returns |
|---|---|---|
| `create_project` | `ProjectConfigInput` | `ProjectDto` |
| `load_project` | `{ path: string }` | `ProjectDto` |
| `save_project` | `{ path: string }` | `void` |
| `update_project_config` | `ProjectConfigInput` | `BudgetSummaryDto` |
| `add_personnel_role` | `PersonnelRoleInput` | `BudgetSummaryDto` |
| `update_personnel_role` | `{ id: string, role: PersonnelRoleInput }` | `BudgetSummaryDto` |
| `delete_personnel_role` | `{ id: string }` | `BudgetSummaryDto` |
| `add_equipment_item` | `EquipmentItemInput` | `BudgetSummaryDto` |
| `update_equipment_item` | `{ id: string, item: EquipmentItemInput }` | `BudgetSummaryDto` |
| `delete_equipment_item` | `{ id: string }` | `BudgetSummaryDto` |
| `add_trip` | `TripInput` | `BudgetSummaryDto` |
| `update_trip` | `{ id: string, trip: TripInput }` | `BudgetSummaryDto` |
| `delete_trip` | `{ id: string }` | `BudgetSummaryDto` |
| `add_other_cost` | `OtherCostInput` | `BudgetSummaryDto` |
| `update_other_cost` | `{ id: string, item: OtherCostInput }` | `BudgetSummaryDto` |
| `delete_other_cost` | `{ id: string }` | `BudgetSummaryDto` |
| `set_subcontracting` | `{ amount_eur: string }` | `BudgetSummaryDto` |
| `acknowledge_cfs` | `void` | `BudgetSummaryDto` |
| `preview_role_cost` | `PersonnelRoleInput` | `RoleCostPreviewDto` |
| `preview_equipment_depreciation` | `EquipmentItemInput` | `DepreciationPreviewDto` |
| `preview_trip_cost` | `TripInput` | `TripCostPreviewDto` |
| `get_rate_versions` | `void` | `RateVersionSummary[]` |
| `get_countries` | `{ version_id: string }` | `CountryDto[]` |

**Rule: never add a frontend-only recalculation.** If you need a new total or derived value, add it to `BudgetSummaryDto` and compute it in `budget_aggregator.rs`, not in the frontend.

---

## 8. Adding a New Budget Category

Suppose you need to add **Category D: Exceptional Costs** (a new EU cost category).

**Step 1 — Domain entity** (`src-tauri/src/domain/entities.rs`):
```rust
pub struct ExceptionalCostItem {
    pub id: Uuid,
    pub name: String,
    pub amount_eur: Decimal,
    pub project_year: u8,
    pub justification: Option<String>,
}
```
Add a `Vec<ExceptionalCostItem>` field to `Project`.

**Step 2 — DTO** (`src-tauri/src/domain/dto.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionalCostInputDto {
    pub name: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_eur: Decimal,  // Decimal serialised as string across IPC
    pub project_year: u8,
    pub justification: Option<String>,
}
```
Add `category_d_total: Decimal` (with `#[serde(with = "rust_decimal::serde::str")]`) and `exceptional_cost_detail: Vec<ExceptionalCostDetailDto>` to `BudgetSummaryDto`.

**Step 3 — Validation** (`src-tauri/src/validation/mod.rs`):
Add `validate_exceptional_cost_item(dto: &ExceptionalCostInput) -> Result<(), AppError>`.

**Step 4 — Calculation** (`src-tauri/src/calculation/budget_aggregator.rs`):
Sum `exceptional_cost_items` into `category_d_total`. Add D to the indirect cost base if EU rules require it.

**Step 5 — Commands** (`src-tauri/src/commands/exceptional_costs.rs`):
Add `add_exceptional_cost`, `update_exceptional_cost`, `delete_exceptional_cost` command handlers, following the same pattern as `other_costs.rs`.

**Step 6 — Register commands** (`src-tauri/src/lib.rs`):
Add the new handlers to `.invoke_handler(tauri::generate_handler![...])`.

**Step 7 — Frontend types** (`src/types/index.ts`):
Add `ExceptionalCostInput`, `ExceptionalCostDetailDto`. Update `BudgetSummaryDto` with `category_d_total`.

**Step 8 — Zod schema** (`src/validators/schemas.ts`):
Add `exceptionalCostSchema`.

**Step 9 — IPC wrapper** (`src/ipc/commands.ts`):
Add `addExceptionalCost`, `updateExceptionalCost`, `deleteExceptionalCost`.

**Step 10 — UI**:
Add a `ExceptionalCosts.tsx` screen. Add it to the wizard step list in `App.tsx` and the left-panel stepper.

**Step 11 — Tests**:
Add inline `#[cfg(test)]` tests to `budget_aggregator.rs` covering Category D totals. Add Zod schema tests to `validators.test.ts`.

---

## 9. Adding a New Rate Table Version

When the European Commission publishes a new Annex 2a/2b rate update:

**Step 1 — Create the JSON file** in `src-tauri/resources/eu_travel_rates/`:

```
v_from_YYYY_MM_DD.json
```

The file must follow the same schema as the existing versions:
```json
{
  "version_id": "v_from_2026_01_01",
  "version_label": "From 1 January 2026",
  "applicable_from": "2026-01-01",
  "flight_bands": [
    { "band_id": "F-01", "min_km": 400, "max_km": 999, "rate_eur": 180 },
    ...
  ],
  "countries": [
    {
      "country_code": "AT",
      "country_name": "Austria",
      "accommodation_per_night_eur": 160,
      "subsistence_per_day_eur": 133
    },
    ...
  ]
}
```

**Step 2 — Register in `rate_data.rs`**:
Add the new file to the `include_str!` array:
```rust
const RATE_FILES: &[&str] = &[
    include_str!("../../resources/eu_travel_rates/v_before_2024_07_31.json"),
    include_str!("../../resources/eu_travel_rates/v_2024_07_31_to_2025_05_12.json"),
    include_str!("../../resources/eu_travel_rates/v_from_2025_05_13.json"),
    include_str!("../../resources/eu_travel_rates/v_from_2026_01_01.json"),  // NEW
];
```

**Step 3 — Test**:
Add at least one test in `integration_test.rs` that verifies a specific rate from the new version (accommodation cost for a known country, a flight band boundary). This guards against JSON parsing errors and rate transcription mistakes.

**No other code changes are required.** The `get_rate_versions` command dynamically returns all loaded versions. The frontend automatically adds the new version to the dropdown.

---

## 10. Adding a New Validation Rule

All Rust validation rules live in `src-tauri/src/validation/mod.rs`.

**Structure of a validator:**
```rust
pub fn validate_personnel_role(
    dto: &PersonnelRoleInput,
    project: &Project,
) -> Result<(), AppError> {
    let mut errors = ValidationErrorBuilder::new();

    if dto.role_label.trim().is_empty() {
        errors.push(FieldError::new("role_label", "REQUIRED", "Role label is required."));
    }

    if project.personnel_roles.iter().any(|r| r.role_label == dto.role_label) {
        errors.push(FieldError::new("role_label", "DUPLICATE_LABEL", "This label is already in use."));
    }

    errors.into_result()
}
```

**Adding a new rule:**
1. Add the `errors.push(...)` call in the appropriate validator function.
2. Add the error code as a constant string if it is reused across multiple validators.
3. Add a test case in the `#[cfg(test)]` block in `validation/mod.rs` using `has_field_error()` or `has_entity_error()` helpers.
4. Map the new error code in the frontend: add handling in the form component that calls the relevant IPC command, so the error appears next to the correct field.

---

## 11. Updating the Calculation Engine

Each calculation module contains `#[cfg(test)]` inline tests. When changing a formula:

1. Update or add the `#[cfg(test)]` tests first to describe the expected new behaviour.
2. Implement the change in the calculation function.
3. Run `cargo test` to confirm all tests pass.
4. Check if `integration_test.rs` scenarios need updating (they test the full pipeline with known workbook values).

**Key arithmetic rule:** never use `f64` for money. All monetary arithmetic must use `rust_decimal::Decimal`. Division must use `Decimal::checked_div()` to handle the zero-denominator case explicitly.

---

## 12. Adding a New Export Format

Export logic lives in `src/export/`. Each exporter is a standalone TypeScript module.

**Template for a new exporter:**
```typescript
// src/export/xmlExporter.ts

import { BudgetSummaryDto } from '../types';
import { save } from '@tauri-apps/plugin-fs';

export async function exportToXml(
  summary: BudgetSummaryDto,
  path: string
): Promise<void> {
  const xml = buildXmlString(summary);
  await save(path, new TextEncoder().encode(xml));
}

function buildXmlString(summary: BudgetSummaryDto): string {
  // build the XML string from the summary DTO fields
}
```

Add a button on the Review & Export screen (`src/screens/ReviewExport.tsx`) that calls `exportToXml(summary, chosenPath)`.

---

## 13. Code Style and Conventions

**Rust:**

- Format with `cargo fmt` before committing.
- Lint with `cargo clippy -- -D warnings` (CI enforces zero warnings).
- All `pub` functions must have documentation comments (`///`).
- No `unwrap()` on `Result` or `Option` in production code — use `?`, `ok_or()`, or explicit `match`.
- No `f64` for monetary values — always `Decimal`.
- Test function names: `test_<what>_<condition>_<expected>`, e.g. `test_equipment_depreciation_cap_applied`.

**TypeScript:**

- Format with Prettier (project `.prettierrc` enforced by CI).
- Lint with ESLint (project `.eslintrc.cjs`).
- All exported functions and types must have JSDoc comments.
- Zod schemas are named `<entity>Schema`, e.g. `personnelRoleSchema`.
- IPC wrapper functions are named `<verb><Entity>`, e.g. `addPersonnelRole`.
- No raw `invoke()` calls outside `src/ipc/commands.ts`.

**Branch and commit conventions:**

- Branches: `feat/<description>`, `fix/<description>`, `docs/<description>`, `test/<description>`.
- Commits: imperative mood, `<type>: <subject>`, e.g. `feat: add Category D exceptional costs`.

---

## 14. Dependency Management

**Rust (`Cargo.toml`):**

| Crate | Purpose |
|---|---|
| `tauri` | Desktop framework (IPC, window management) |
| `tauri-plugin-dialog` | Native file open/save dialogs |
| `tauri-plugin-fs` | File system read/write |
| `tauri-plugin-shell` | Shell commands (used for open-in-folder) |
| `serde` + `serde_json` | JSON serialisation of DTOs |
| `rust_decimal` + `rust_decimal_macros` | Exact decimal arithmetic |
| `uuid` | UUID generation for entity IDs |
| `chrono` | Date handling (auto-save timestamps) |
| `thiserror` | `AppError` derive macro |

Adding a Rust dependency: `cargo add <crate>` in `src-tauri/`, then verify `cargo build` succeeds.

**TypeScript (`package.json`):**

| Package | Purpose |
|---|---|
| `react` + `react-dom` | UI framework |
| `zustand` | State management |
| `react-hook-form` | Form handling |
| `zod` | Schema validation |
| `recharts` | Charts (ring chart, bar chart) |
| `exceljs` | Excel export |
| `@react-pdf/renderer` | PDF export |
| `@radix-ui/*` | Accessible UI primitives |
| `uuid` | UUID generation (client-side, for optimistic IDs) |
| `vitest` | Test runner |
| `@vitejs/plugin-react` | Vite plugin |

Adding an npm dependency: `npm install <package>` in `erc-budget/`. Always pin to a minor version range (`^x.y.z`) in `package.json`.
