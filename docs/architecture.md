# Architecture

**Document:** TASK-07 Deliverable  
**Date:** 2026-07-10  
**Status:** Draft — Awaiting Approval Before Proceeding to TASK-08  
**Source documents:** business-rules.md, domain-model.md, input-catalog.md, ux-design.md

---

## 1. Requirements Summary (Architecture Constraints)

Before evaluating frameworks, the constraints that must be satisfied are:

| Constraint | Detail |
|---|---|
| Platform | Windows 10+ and macOS 12+ — both must be first-class targets |
| Distribution | Installable desktop app; no browser, no server required |
| UI complexity | Split-panel layout; live-updating charts; rich forms with real-time validation |
| Calculation engine | Year-by-year salary chains, depreciation, aggregations — all exact arithmetic |
| Export | Excel (.xlsx), PDF, CSV |
| Persistence | File-based project save/load (`.ercbudget` files); auto-save |
| Bundled data | EU travel rate tables embedded in the application |
| User type | Non-technical (researchers, grant writers) — install must be trivial |
| Team | Assumed small (2–4 developers); maintainability critical |
| Online dependency | None — must work fully offline |

---

## 2. Framework Evaluation

Four frameworks are evaluated against six criteria, each scored 1–5 (5 = best).

---

### 2.1 Tauri (v2)

Tauri is a framework for building desktop applications using web technologies (HTML/CSS/TypeScript) for the frontend and Rust for the backend. It embeds the operating system's native webview (WebKit on macOS, WebView2 on Windows) rather than bundling a full browser engine.

**Performance — 5/5**  
The Rust backend handles all business logic: calculations, persistence, validation, and data processing. Rust is compiled to native code, making the calculation engine extremely fast — far faster than this application's workload demands. The frontend webview handles UI rendering only. There is no JavaScript-heavy computation for the critical path (all salary chains, depreciation, and aggregation run in Rust).

**Maintainability — 4/5**  
The frontend is standard TypeScript + React — a widely known and well-documented stack with a large hiring pool. The Rust backend introduces a learning curve for developers unfamiliar with Rust, but the backend surface in this application is small and well-bounded (a set of IPC command handlers + calculation functions). The strict Rust type system and ownership model actually reduce runtime bugs in the calculation engine. Clear Rust documentation and a growing community support long-term maintenance. Score reduced by one point for the Rust learning curve.

**Distribution — 5/5**  
Tauri produces native platform installers: `.msi` / `.exe` on Windows, `.dmg` / `.app` on macOS. Bundle size is approximately 5–15 MB (no bundled browser engine). Installation is a standard double-click installer experience. Tauri v2 includes built-in auto-update support. Code signing is supported for both platforms. This is significantly better than Electron's 150–200 MB bundles.

**UX — 5/5**  
The frontend is pure HTML/CSS/TypeScript. The split-panel layout, live charts (Recharts or Chart.js), rich form components, and real-time dashboard updates are all straightforwardly implemented with standard web UI libraries. The application can adopt OS-native look-and-feel via CSS (system fonts, colours) while maintaining full design control. There are no UI component limitations from the framework.

**Export support — 5/5**  
Excel: ExcelJS or SheetJS — both mature, well-supported npm packages for generating `.xlsx` files with formatting, formulas, and multiple sheets. PDF: `@react-pdf/renderer` or `jsPDF` — both capable of producing formatted PDF reports. CSV: trivial plain-text generation. All run in the TypeScript layer with no server dependency.

**Long-term support — 4/5**  
Tauri v2.0 was released as stable in 2024 and is actively maintained by the Crabnebula team and the open-source community. It is used in production by companies including 1Password (partially). The project has strong GitHub activity and community adoption. One point reduced relative to Electron's longer track record, but the trajectory is positive and the project has significant momentum.

**Total: 28/30**

---

### 2.2 Electron

Electron bundles Node.js and the Chromium browser engine together with the application, enabling desktop apps built entirely in JavaScript/TypeScript.

**Performance — 3/5**  
The calculation engine would run in Node.js (JavaScript/TypeScript). For this application's scale — salary chains for ~15 roles, a handful of equipment items, a few dozen trips — performance is adequate. However, launching the app and memory usage are significantly heavier than Tauri (Chromium alone uses ~80–150 MB RAM at startup). Live dashboard recalculation in JS is fast enough but less precise for financial arithmetic without care (floating-point issues must be managed explicitly with decimal libraries).

**Maintainability — 5/5**  
Electron is the most mature, best-documented cross-platform desktop framework. The entire stack is TypeScript — frontend and backend use the same language. The developer hiring pool is largest. VS Code, Slack, Discord, Notion, and hundreds of other applications are built on Electron, generating extensive community knowledge, tutorials, and tooling.

**Distribution — 2/5**  
Electron bundles ship at 150–200 MB minimum. On macOS, auto-update and code signing require notarization with Apple. On Windows, a valid certificate and Squirrel or NSIS installer are needed. The large bundle size is the most significant drawback — a budget tool distributed to grant writers at research institutions may be rejected by IT departments that scrutinise large installers, or frustrate users on slow connections.

**UX — 5/5**  
Identical to Tauri — full control over HTML/CSS/TypeScript UI. Chromium guarantees consistent rendering on both platforms (no WebKit quirks). All UI libraries are available.

**Export support — 5/5**  
Same npm ecosystem as Tauri. ExcelJS, SheetJS, jsPDF, Puppeteer — all available. Puppeteer is particularly effective for high-fidelity PDF generation in Electron (the built-in Chromium can render to PDF).

**Long-term support — 5/5**  
Electron has been in active production use since 2013. It is maintained by GitHub (now Microsoft) under the OpenJS Foundation. VS Code's continued existence on Electron is a strong signal of its long-term viability. Security updates follow Chromium's release cycle.

**Total: 25/30**

---

### 2.3 .NET MAUI

.NET MAUI (Multi-platform App UI) is Microsoft's cross-platform native UI framework for C#. On macOS it targets the Mac Catalyst layer; on Windows it uses WinUI 3.

**Performance — 4/5**  
C# and .NET are fast for the calculation workload. Native UI controls render smoothly. However, Mac Catalyst introduces some overhead compared to a fully native macOS app, and MAUI's macOS support has historically lagged behind Windows.

**Maintainability — 3/5**  
C# is a well-known language with good tooling. However, the MAUI framework is relatively young (released 2022) and its macOS support via Mac Catalyst was still maturing as of 2024. UI customisation is more limited than web-based frameworks — achieving the split-panel dashboard with live charts requires third-party commercial controls (e.g., Telerik, Syncfusion) or significant custom implementation. The debugging experience on macOS is notably weaker than on Windows.

**Distribution — 4/5**  
Produces native `.msix`/`.exe` on Windows and `.app`/`.pkg` on macOS. Bundle sizes are reasonable (~20–50 MB). Code signing and notarization follow the standard platform paths. Auto-update requires additional work (no built-in solution).

**UX — 3/5**  
MAUI uses native controls per platform — a good baseline but limited flexibility for the custom split-panel dashboard required by this application. Achieving live-updating charts with the quality shown in the UX design requires commercial chart components. The inconsistency between Windows WinUI 3 controls and macOS Catalyst controls means some UI differences between platforms are unavoidable.

**Export support — 4/5**  
Excel: ClosedXML, EPPlus, or NPOI — all mature .NET libraries for `.xlsx` generation. PDF: QuestPDF (excellent) or iText. CSV: trivial. Strong export support in the .NET ecosystem.

**Long-term support — 4/5**  
Microsoft is committed to MAUI as the successor to Xamarin.Forms. The backing is strong, but the macOS experience has been the weakest link. Future improvements to Mac Catalyst may address some current limitations.

**Total: 22/30**

---

### 2.4 Flutter Desktop

Flutter is Google's UI framework using the Dart language. It draws every pixel itself using the Skia/Impeller rendering engine rather than native OS controls.

**Performance — 4/5**  
Flutter's rendering is smooth and consistent. Dart performance is adequate for the calculation workload. The custom rendering engine avoids OS-specific quirks in UI rendering.

**Maintainability — 3/5**  
Dart is a less commonly known language with a smaller hiring pool than TypeScript, C#, or Python. The Flutter desktop ecosystem (particularly for Windows and macOS) is less mature than the mobile ecosystem, with fewer packages and less community knowledge for desktop-specific scenarios. The calculation logic in Dart lacks the rich financial library ecosystem of Rust or .NET.

**Distribution — 4/5**  
Produces `.msix`/`.exe` on Windows and `.app`/`.dmg` on macOS. Bundle sizes are moderate (~30–60 MB). No built-in auto-update mechanism — requires third-party packages.

**UX — 4/5**  
Flutter's custom renderer gives full control over visual design with pixel-perfect consistency across platforms. Charts are available via `fl_chart` or `syncfusion_flutter_charts`. The split-panel layout is straightforward. One drawback: Flutter's desktop UI does not use OS-native controls, which can feel slightly "off" to users accustomed to system fonts and widgets.

**Export support — 3/5**  
Excel: the Dart ecosystem has `excel` and `spreadsheet_decoder` packages, but they are less mature than ExcelJS or ClosedXML. PDF: the `pdf` package is high-quality. CSV: trivial. The Excel export quality is the weakest link — producing a well-formatted `.xlsx` with multiple sheets and proper cell formatting is more difficult in Dart than in JS or .NET.

**Long-term support — 3/5**  
Google backs Flutter actively for mobile. Desktop support was declared stable in 2022 but remains less battle-tested than mobile. Google's history of deprecating developer platforms (e.g., Fuschia, web apps) introduces some uncertainty.

**Total: 21/30**

---

### 2.5 Evaluation Summary

| Criterion | Tauri | Electron | .NET MAUI | Flutter |
|---|---|---|---|---|
| Performance | 5 | 3 | 4 | 4 |
| Maintainability | 4 | 5 | 3 | 3 |
| Distribution | 5 | 2 | 4 | 4 |
| UX | 5 | 5 | 3 | 4 |
| Export support | 5 | 5 | 4 | 3 |
| Long-term support | 4 | 5 | 4 | 3 |
| **Total** | **28** | **25** | **22** | **21** |

---

## 3. Recommendation: Tauri v2

**Tauri v2 is the recommended platform.**

The decisive advantages are:

**Distribution wins the practical argument.** This application will be distributed to grant writers at academic institutions — people who have limited IT support and may download the tool on restricted-bandwidth networks. A 5–10 MB installer is frictionless. A 150–200 MB Electron installer is a genuine barrier. In institutional IT environments, large installers often require administrator approval; small installers often do not.

**The Rust calculation engine is the right choice for financial software.** The salary projection chain, depreciation formula with cap logic, and multi-category aggregations involve compounding floating-point arithmetic. Rust's strong type system, overflow safety, and the availability of decimal arithmetic crates (`rust_decimal`) make the calculation engine provably correct in a way that JavaScript cannot guarantee without additional discipline.

**The frontend is still just TypeScript.** The overwhelming majority of development effort is in the UI layer — forms, charts, validation messages, state management — all of which are pure TypeScript/React. The Rust backend is a small, bounded surface (IPC command handlers + calculation functions) with a clean API contract. A developer proficient in web technologies can build 80% of this application without knowing Rust deeply.

**Fallback option:** If the development team has no Rust experience and cannot acquire it, **Electron** is the recommended alternative. It scores second-highest and provides a completely TypeScript stack at the cost of bundle size. The calculation engine in TypeScript must use a decimal arithmetic library (`decimal.js`) to avoid floating-point rounding errors.

---

## 4. Technology Stack (Tauri v2)

| Layer | Technology |
|---|---|
| Desktop framework | Tauri v2 |
| Backend language | Rust (stable, edition 2021) |
| Frontend language | TypeScript 5.x |
| Frontend framework | React 18 (with hooks) |
| State management | Zustand (lightweight, TypeScript-first) |
| UI component library | Radix UI (unstyled, accessible primitives) + custom CSS |
| Charts | Recharts (React-based, declarative) |
| Form handling | React Hook Form + Zod (validation schemas) |
| Excel export | ExcelJS |
| PDF export | @react-pdf/renderer |
| CSV export | plain TypeScript string builder |
| Decimal arithmetic | `rust_decimal` (Rust side) |
| Serialization | `serde` + `serde_json` (Rust); JSON (IPC protocol) |
| Persistence format | JSON with `.ercbudget` extension |
| Build tool | Vite (frontend) + Cargo (backend) |
| Testing (Rust) | Rust built-in test harness + `rstest` |
| Testing (TS) | Vitest + React Testing Library |

---

## 5. Clean Architecture

The application is structured in seven layers. Dependencies flow strictly inward: outer layers depend on inner layers, never the reverse.

```
┌──────────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                         │
│            (TypeScript / React / Tauri Webview)              │
├──────────────────────────────────────────────────────────────┤
│                    APPLICATION LAYER                          │
│              (Rust — Tauri IPC Command Handlers)             │
├──────────────────────────────────────────────────────────────┤
│                      DOMAIN LAYER                             │
│               (Rust — Entities + Domain Rules)               │
├─────────────────────┬────────────────────────────────────────┤
│   CALCULATION       │   VALIDATION       │   EXPORT          │
│   ENGINE            │   ENGINE           │   ENGINE          │
│   (Rust)            │   (Rust + TS)      │   (TypeScript)    │
├─────────────────────┴────────────────────┴────────────────────┤
│                     PERSISTENCE LAYER                          │
│               (Rust — File I/O + Rate Data Store)             │
└──────────────────────────────────────────────────────────────┘
```

---

### 5.1 Presentation Layer

**Technology:** TypeScript 5, React 18, Recharts, Radix UI, Zustand, React Hook Form, Zod  
**Location:** `src/` (frontend, compiled to HTML/JS/CSS served in the Tauri webview)

**Responsibilities:**
- Render all 8 screens defined in ux-design.md
- Manage UI state (current step, open forms, loading indicators)
- Collect and validate user inputs at the field level (Zod schemas)
- Call Application Layer via Tauri `invoke()` commands
- Display computed results returned from the backend (never re-compute in the frontend)
- Render the live budget dashboard (right panel) from the BudgetSummary DTO returned by the backend

**Key modules:**

```
src/
├── screens/
│   ├── Welcome.tsx
│   ├── ProjectSetup.tsx
│   ├── BudgetSettings.tsx
│   ├── WorkPackages.tsx
│   ├── Personnel.tsx
│   ├── Equipment.tsx
│   ├── Travel.tsx
│   ├── OtherCosts.tsx
│   └── ReviewExport.tsx
├── components/
│   ├── FormField.tsx
│   ├── RoleCard.tsx
│   ├── EquipmentCard.tsx
│   ├── TripCard.tsx
│   ├── CostItemCard.tsx
│   ├── LivePreviewBox.tsx
│   ├── EmptyStateCard.tsx
│   ├── WarningBanner.tsx
│   ├── BudgetRingChart.tsx
│   ├── BudgetYearBarChart.tsx
│   ├── CategoryTotalsPanel.tsx
│   └── CFSModal.tsx
├── store/
│   └── projectStore.ts          ← Zustand store; holds local UI state
├── hooks/
│   ├── useBudgetSummary.ts      ← subscribes to live backend recalculation
│   └── useAutoSave.ts           ← triggers save on every valid state change
├── validators/
│   └── schemas.ts               ← Zod schemas matching all input-catalog.md fields
├── ipc/
│   └── commands.ts              ← typed wrappers around Tauri invoke() calls
└── export/
    ├── excelExporter.ts
    ├── pdfExporter.ts
    └── csvExporter.ts
```

**Rules enforced in this layer:**
- No business logic. The frontend never calculates a salary, a depreciation, or a total. It only renders values returned by the backend.
- No raw Tauri `invoke()` calls outside `ipc/commands.ts`. All backend calls go through typed wrapper functions.
- No direct JSON manipulation of the project state. All mutations go through the Application Layer.

---

### 5.2 Application Layer

**Technology:** Rust (Tauri command handlers)  
**Location:** `src-tauri/src/commands/`

**Responsibilities:**
- Expose a typed IPC API to the Presentation Layer
- Orchestrate calls to the Domain Layer, Calculation Engine, Validation Engine, and Persistence Layer
- Convert domain entities to Data Transfer Objects (DTOs) safe for JSON serialisation
- Trigger BudgetSummary recalculation after every state mutation
- Return errors to the frontend as structured `AppError` types (never panics)

**IPC Commands (Tauri `#[tauri::command]` functions):**

```rust
// Project lifecycle
create_project(config: ProjectConfigDto) -> Result<ProjectDto, AppError>
load_project(path: String) -> Result<ProjectDto, AppError>
save_project(path: String) -> Result<(), AppError>

// Personnel
add_personnel_role(role: PersonnelRoleDto) -> Result<BudgetSummaryDto, AppError>
update_personnel_role(id: String, role: PersonnelRoleDto) -> Result<BudgetSummaryDto, AppError>
delete_personnel_role(id: String) -> Result<BudgetSummaryDto, AppError>

// Equipment
add_equipment_item(item: EquipmentItemDto) -> Result<BudgetSummaryDto, AppError>
update_equipment_item(id: String, item: EquipmentItemDto) -> Result<BudgetSummaryDto, AppError>
delete_equipment_item(id: String) -> Result<BudgetSummaryDto, AppError>

// Travel
add_trip(trip: TripDto) -> Result<BudgetSummaryDto, AppError>
update_trip(id: String, trip: TripDto) -> Result<BudgetSummaryDto, AppError>
delete_trip(id: String) -> Result<BudgetSummaryDto, AppError>

// Other costs
add_other_cost(item: OtherCostItemDto) -> Result<BudgetSummaryDto, AppError>
update_other_cost(id: String, item: OtherCostItemDto) -> Result<BudgetSummaryDto, AppError>
delete_other_cost(id: String) -> Result<BudgetSummaryDto, AppError>

// Subcontracting
set_subcontracting(amount: Decimal) -> Result<BudgetSummaryDto, AppError>

// Preview (called live while typing, before saving)
preview_role_cost(role: PersonnelRoleDto) -> Result<RoleCostPreviewDto, AppError>
preview_equipment_depreciation(item: EquipmentItemDto) -> Result<DepreciationPreviewDto, AppError>
preview_trip_cost(trip: TripDto) -> Result<TripCostPreviewDto, AppError>

// Reference data
get_countries(version_id: String) -> Result<Vec<CountryDto>, AppError>
get_flight_bands(version_id: String, distance_km: u32) -> Result<FlightBandDto, AppError>
get_rate_versions() -> Result<Vec<RateVersionDto>, AppError>
```

Every mutation command returns an updated `BudgetSummaryDto` — the frontend never needs to manually recalculate or re-fetch the summary separately.

---

### 5.3 Domain Layer

**Technology:** Rust structs and enums  
**Location:** `src-tauri/src/domain/`

**Responsibilities:**
- Define all entities as strongly typed Rust structs (Project, PersonnelRole, EquipmentItem, Trip, OtherDirectCostItem, Subcontracting)
- Enforce entity-level constraints as constructor validation (a PersonnelRole cannot be created with FTE ≤ 0)
- Define domain value types: `Decimal` for all monetary values (via `rust_decimal`), `Uuid` for IDs
- No I/O, no external dependencies — pure data and rules

**Key domain structs:**

```rust
// src-tauri/src/domain/entities.rs

pub struct Project {
    pub id: Uuid,
    pub config: ProjectConfig,
    pub personnel_roles: Vec<PersonnelRole>,
    pub equipment_items: Vec<EquipmentItem>,
    pub trips: Vec<Trip>,
    pub other_cost_items: Vec<OtherDirectCostItem>,
    pub subcontracting: Subcontracting,
}

pub struct ProjectConfig {
    pub duration_years: u8,            // 1–7
    pub work_package_count: u8,        // 1–10
    pub default_inflation_rate: Decimal, // 0–100 %
    pub try_eur_exchange_rate: Decimal, // > 0
    pub indirect_cost_rate: Decimal,   // 0–50 %, default 25
    pub applicable_rate_version_id: String,
    pub work_package_names: Vec<Option<String>>,
}

pub struct PersonnelRole {
    pub id: Uuid,
    pub role_type: RoleType,           // PI | Expert | PostDoc | Admin
    pub role_label: String,            // unique within project
    pub current_monthly_salary_try: Decimal, // > 0
    pub fte_fraction: Decimal,         // 0 < x ≤ 1
    pub inflation_rate: Decimal,       // 0–100 %
    pub active_years: Vec<u8>,         // subset of 1..=duration_years
    pub work_package_ids: Vec<u8>,
}

pub struct EquipmentItem {
    pub id: Uuid,
    pub name: String,
    pub purchase_cost_eur: Decimal,    // > 0
    pub useful_lifetime_months: u32,   // ≥ 1
    pub grant_usage_pct: Decimal,      // 0 < x ≤ 100
    pub grant_usage_months: u32,       // ≥ 1
    pub year_of_purchase: Option<u8>,
    pub work_package_ids: Vec<u8>,
}

pub enum TripType {
    Itemized {
        destination_country_code: String,
        one_way_distance_km: u32,
        number_of_nights: u32,
        number_of_days: u32,
        domestic_transport_per_instance_eur: Decimal,
    },
    FlatAmount {
        flat_amount_per_instance_eur: Decimal,
    },
}

pub struct Trip {
    pub id: Uuid,
    pub name: String,
    pub trip_type: TripType,
    pub project_year: u8,
    pub number_of_instances: u32,
    pub work_package_id: Option<u8>,
}

pub struct OtherDirectCostItem {
    pub id: Uuid,
    pub name: String,
    pub amount_eur: Decimal,
    pub project_year: u8,
    pub is_cfs_item: bool,
    pub notes: Option<String>,
    pub work_package_id: Option<u8>,
}

pub struct Subcontracting {
    pub amount_eur: Decimal,           // ≥ 0, default 0
}
```

---

### 5.4 Calculation Engine

**Technology:** Rust  
**Location:** `src-tauri/src/calculation/`

**Responsibilities:**
- Execute all budget calculations as defined in business-rules.md
- Use `rust_decimal::Decimal` for all monetary arithmetic (exact decimal, no floating-point rounding)
- Pure functions only — no side effects, no I/O, no state mutation
- Return detailed result structs that the Application Layer maps to DTOs

**Modules:**

```
src-tauri/src/calculation/
├── mod.rs
├── salary_projection.rs      ← PE-02: TRY→EUR, inflation chain per year
├── personnel_cost.rs         ← PE-03: annual cost = salary × 12 × FTE per active year
├── equipment_depreciation.rs ← EQ-02: (cost / lifetime) × pct × months, capped
├── trip_cost.rs              ← TR-02 to TR-05: rate lookup + component sum
├── budget_aggregator.rs      ← IC-01, PT-01 to PT-03: totals + indirect
└── cfs_checker.rs            ← OC-02: threshold monitoring
```

**Key calculation function signatures:**

```rust
// salary_projection.rs
pub fn project_salaries(
    current_salary_try: Decimal,
    try_eur_rate: Decimal,
    inflation_rate_pct: Decimal,
    duration_years: u8,
) -> Vec<SalaryProjection>  // one entry per year

// personnel_cost.rs
pub fn calculate_personnel_cost_lines(
    salary_projections: &[SalaryProjection],
    fte_fraction: Decimal,
    active_years: &[u8],
) -> Vec<PersonnelCostLine>

// equipment_depreciation.rs
pub fn calculate_depreciation(
    purchase_cost_eur: Decimal,
    useful_lifetime_months: u32,
    grant_usage_pct: Decimal,
    grant_usage_months: u32,
) -> EquipmentDepreciationResult

// trip_cost.rs
pub fn calculate_trip_cost(
    trip: &Trip,
    country_rates: Option<&CountryRates>,    // None for FlatAmount
    flight_band: Option<&FlightDistanceBand>, // None if distance < 400
) -> TripCostResult

// budget_aggregator.rs
pub fn aggregate_budget(
    project: &Project,
    rate_data: &RateData,
) -> BudgetSummary

// cfs_checker.rs
pub fn is_cfs_required(
    requested_contribution: Decimal,
    has_cfs_item: bool,
) -> CfsStatus   // enum: NotRequired | RequiredAndPresent | RequiredButMissing
```

**Arithmetic precision contract:**  
All monetary values are `rust_decimal::Decimal` throughout the calculation engine. Rounding to the nearest euro occurs only at the point of serialisation to the frontend DTO. Internal calculations carry full precision.

---

### 5.5 Validation Engine

**Technology:** Rust (business-rule validators) + TypeScript/Zod (field-level validators)  
**Location:** `src-tauri/src/validation/` (Rust) and `src/validators/` (TypeScript)

**Responsibilities:**
- Field-level validation (TypeScript/Zod): runs in the browser on every input change; provides immediate per-field feedback without a round-trip to Rust
- Business-rule validation (Rust): runs before any entity is persisted; enforces cross-field and cross-entity rules
- Validation results are structured errors with: field name, error code, human-readable message

**Validation layers:**

```
TypeScript (Zod schemas) — field-level, instant, in the browser
    ↓ on form submit
Rust validation (command handler) — business rules, before persistence
    ↓ on entity creation
Domain layer constructors — invariant enforcement, always
```

**Example: PersonnelRole validation flow**

```
User fills role form → Zod validates field by field (type, range, required)
→ User clicks Save
→ Frontend calls add_personnel_role() IPC command
→ Rust: validate_personnel_role():
    - role_label unique within project
    - only one PI allowed
    - active_years all within project duration
    - inflation_rate between 0 and 100
→ If valid: create domain entity + calculate + persist + return BudgetSummaryDto
→ If invalid: return AppError { field: "role_label", code: "DUPLICATE_LABEL",
                                message: "This label is already in use." }
→ Frontend maps AppError back to field-level error display
```

**Validation error structure (Rust):**

```rust
pub struct ValidationError {
    pub field: Option<String>,  // None for entity-level errors
    pub code: String,           // machine-readable (DUPLICATE_LABEL, etc.)
    pub message: String,        // human-readable, shown in UI
}

pub enum AppError {
    Validation(Vec<ValidationError>),
    Persistence(String),
    NotFound(String),
    Internal(String),
}
```

---

### 5.6 Export Engine

**Technology:** TypeScript  
**Location:** `src/export/`

**Responsibilities:**
- Receive a fully computed `BudgetSummaryDto` + full project data from the Application Layer
- Generate Excel, PDF, or CSV output
- Write the file to the user-selected path via Tauri's file system API

**Excel Export (`excelExporter.ts`):**

Uses ExcelJS to produce a formatted `.xlsx` workbook with three sheets:

- **Sheet 1: Overview** — one-page summary matching the ERC submission format (Categories A, B, C1, C2, C3, E, total direct, total eligible, requested EU contribution)
- **Sheet 2: Budget by Year** — per-year breakdown table (years as columns, categories as rows)
- **Sheet 3: Detail** — itemised breakdown per personnel role, equipment item, trip, and C3 item

Formatting: column widths, bold headers, number format `#,##0 €`, total rows with double underline border.

**PDF Export (`pdfExporter.ts`):**

Uses `@react-pdf/renderer` to produce a formatted one-page PDF summary. Includes: project name, PI name, call reference, preparation date, budget table by category, indirect costs, total eligible, and requested contribution. Matches the Overview sheet layout.

**CSV Export (`csvExporter.ts`):**

Produces a flat comma-separated file with all budget lines. Useful for import into institutional finance systems. Structure: Category, Item, Year, Amount (EUR).

---

### 5.7 Persistence Layer

**Technology:** Rust (file I/O via Tauri's `tauri::api::path` and `std::fs`)  
**Location:** `src-tauri/src/persistence/`

**Responsibilities:**
- Serialise the complete `Project` entity tree to JSON
- Deserialise from JSON on project load
- Auto-save to a temporary file after every valid mutation (durable auto-save)
- Save/load named `.ercbudget` files via the user's file system
- Load bundled EU rate data at application startup

**File structure (`.ercbudget`):**

```json
{
  "format_version": "1.0",
  "created_at": "2026-07-10T10:00:00Z",
  "updated_at": "2026-07-10T14:30:00Z",
  "project": {
    "id": "uuid",
    "config": { ... },
    "personnel_roles": [ ... ],
    "equipment_items": [ ... ],
    "trips": [ ... ],
    "other_cost_items": [ ... ],
    "subcontracting": { "amount_eur": "0" }
  }
}
```

All `Decimal` values are serialised as strings (e.g., `"4500.00"`) to avoid JSON floating-point representation issues.

**Bundled rate data:**

The EU Annex 2a/2b rate tables are stored as embedded JSON files compiled into the application binary at build time (using Rust's `include_str!` macro):

```
src-tauri/
└── resources/
    └── eu_travel_rates/
        ├── v_before_2024_07_31.json
        ├── v_2024_07_31_to_2025_05_12.json
        └── v_from_2025_05_13.json    ← current version
```

Each file contains a complete country list with accommodation + subsistence rates, and the full flight distance band table. These files are loaded once at startup into an in-memory `RateData` struct. To update the rates in a future version, only these JSON files need to be updated — no code changes required.

---

## 6. Component Interaction Diagram

```
User types in Role Form (salary field)
          │
          ▼
Zod schema validates field (TypeScript)
  → shows/hides field error immediately
          │
          ▼
User clicks "Save Role"
          │
          ▼
Frontend calls invoke("add_personnel_role", { role: dto })
          │
          ▼  [IPC boundary — JSON serialised]
          │
          ▼
Application Layer (Rust) — add_personnel_role command handler
  → Validation Engine: validate_personnel_role()
  → if error: return AppError to frontend
          │ if valid:
          ▼
  → Domain Layer: construct PersonnelRole entity
          │
          ▼
  → Calculation Engine: project_salaries() + calculate_personnel_cost_lines()
          │
          ▼
  → Project state updated (in-memory)
          │
          ▼
  → Persistence Layer: auto_save()
          │
          ▼
  → Calculation Engine: aggregate_budget()
          │
          ▼
  → CFS Checker: is_cfs_required()
          │
          ▼
  → Application Layer: map to BudgetSummaryDto
          │
          ▼  [IPC boundary — JSON serialised]
          │
          ▼
Frontend receives BudgetSummaryDto
  → Zustand store updated
  → Right panel re-renders (ring chart, category totals, year bar chart)
  → Role card added to personnel list
  → LivePreviewBox shows updated totals
```

Total round-trip time from "Save Role" to updated right panel: < 50 ms (including Rust calculation + auto-save + JSON serialisation/deserialisation + React re-render).

---

## 7. Project Structure

```
erc-budget/
├── src/                        ← TypeScript / React frontend
│   ├── screens/
│   ├── components/
│   ├── store/
│   ├── hooks/
│   ├── validators/
│   ├── ipc/
│   └── export/
├── src-tauri/                  ← Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/           ← Application Layer (IPC handlers)
│   │   ├── domain/             ← Domain Layer (entities + types)
│   │   ├── calculation/        ← Calculation Engine
│   │   ├── validation/         ← Validation Engine (Rust side)
│   │   ├── persistence/        ← Persistence Layer
│   │   └── export/             ← (future: server-side export if needed)
│   ├── resources/
│   │   └── eu_travel_rates/    ← Bundled rate JSON files
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/
│   ├── calculation/            ← Rust unit tests for all calculation functions
│   └── ui/                     ← Vitest + React Testing Library tests
├── package.json
├── vite.config.ts
└── tsconfig.json
```

---

## 8. Key Architecture Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Desktop framework | Tauri v2 | Small bundle, Rust calculation engine, TypeScript UI |
| UI framework | React 18 | Large ecosystem, concurrent rendering for live updates |
| State management | Zustand | Minimal boilerplate; fits small team |
| Decimal arithmetic | `rust_decimal` (Rust), always-string serialisation | Exact decimal for all financial calculations; no floating-point errors |
| Validation approach | Dual-layer (Zod frontend + Rust backend) | Instant field feedback + guaranteed server-side correctness |
| Persistence format | JSON with `.ercbudget` extension | Human-readable, debuggable, easy to migrate to future schema versions |
| Rate data storage | Embedded JSON compiled into binary | Fully offline; no network calls; updatable without code changes |
| Export | TypeScript (ExcelJS, @react-pdf/renderer) | Best library support for xlsx/PDF in the JS ecosystem |
| Budget recalculation trigger | Every mutation → returns full BudgetSummaryDto | Guarantees right panel is always in sync; simplifies frontend state |

---

## 9. Open Questions

No open questions — all architectural decisions are resolved within this document and consistent with the approved business rules, domain model, input catalog, and UX design.

---

**Confidence Level: 94%**

High confidence on platform selection, layer responsibilities, and module organisation. Residual 6%: the choice of React state management library (Zustand vs. Jotai vs. React Context) and chart library (Recharts vs. Victory vs. Chart.js) may be adjusted during TASK-10 implementation based on real-time performance profiling of the live dashboard update path.

**Recommended Next Step:**  
Await PI approval. Once approved, proceed to TASK-08 (Calculation Engine Specification).
