# Project Execution Application — Technical Architecture

**Phase 06 — Technical Architecture**
**Project:** Horizon Europe Project Management Platform
**Date:** 2026-08-03

---

## 1. Architecture Goals

| Goal | Requirement |
|---|---|
| Independence | The Execution Application is a fully standalone desktop binary |
| Shared Core | All shared business logic lives in `erc-core`; no duplication |
| Extensibility | Adding new modules must not require changes to existing modules |
| Testability | Every engine is independently testable with no Tauri dependency |
| Offline-First | No network access required at runtime |
| Cross-Platform | macOS 12+, Windows 10+ |
| Future-Ready | Designed to become a module in a future unified platform |

---

## 2. Technology Stack

The Execution Application uses the **identical technology stack** as the Budget Application. This minimises the learning curve, maximises code reuse, and ensures the two apps can share UI primitives.

| Layer | Technology | Version |
|---|---|---|
| Desktop Runtime | **Tauri** | 2.x |
| Frontend Framework | **React** | 18.x |
| Frontend Language | **TypeScript** | 5.x |
| State Management | **Zustand** | 4.x |
| Form Validation | **Zod** + React Hook Form | 3.x / 7.x |
| Charting | **Recharts** | 2.x |
| Excel Export | **ExcelJS** | 4.x |
| PDF Export | Browser `window.print()` (upgrade to `@react-pdf/renderer` if needed) | — |
| Backend Language | **Rust** | stable |
| Decimal Arithmetic | **rust_decimal** | 1.x |
| Serialization | **serde** / **serde_json** | 1.x |
| UUID Generation | **uuid** (v4) | 1.x |
| Date Handling | **chrono** | 0.4.x |
| Error Handling | **thiserror** | 1.x |
| Shared Core | **erc-core** (Cargo workspace member) | — |
| Testing (frontend) | **Vitest** + Testing Library | — |
| Testing (backend) | Rust `#[cfg(test)]` + integration tests | — |
| Build Tool | **Vite** | 5.x |
| Package Manager | **pnpm** | 11.x |

---

## 3. System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│  ERC Execution — Tauri Desktop Application                         │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Frontend Process (WebView)                                 │   │
│  │  React + TypeScript + Zustand                               │   │
│  │                                                             │   │
│  │  ┌──────────┐  ┌───────────┐  ┌────────┐  ┌───────────┐  │   │
│  │  │ Screens  │  │Components │  │ Store  │  │  IPC      │  │   │
│  │  │ (pages)  │  │(shared UI)│  │(Zustand│  │ commands.ts│  │   │
│  │  └──────────┘  └───────────┘  └────────┘  └───────────┘  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                            │ Tauri IPC (invoke / serde_json)        │
│  ┌─────────────────────────▼───────────────────────────────────┐   │
│  │  Backend Process (Rust)                                     │   │
│  │                                                             │   │
│  │  ┌─────────────────────────────────────────────────────┐   │   │
│  │  │  IPC Command Layer (commands/)                      │   │   │
│  │  └─────────────────────────────────────────────────────┘   │   │
│  │     │              │              │              │           │   │
│  │  ┌──▼──┐       ┌───▼───┐     ┌───▼───┐      ┌──▼──┐      │   │
│  │  │Val. │       │Exec.  │     │Report.│      │Notif│      │   │
│  │  │Eng. │       │Engine │     │Engine │      │Eng. │      │   │
│  │  └──┬──┘       └───┬───┘     └───┬───┘      └──┬──┘      │   │
│  │     └──────────────┴─────────────┴──────────────┘         │   │
│  │                          │                                  │   │
│  │  ┌───────────────────────▼──────────────────────────────┐  │   │
│  │  │  erc-core (Shared Core Library)                      │  │   │
│  │  │  Domain Entities · Calculation Engine                │  │   │
│  │  │  Validation · Persistence · Rate Data · Error Types  │  │   │
│  │  └──────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                          │
               .ercbudget file (disk)
```

---

## 4. Rust Project Structure

### 4.1 Cargo Workspace

```toml
# /Cargo.toml (workspace root)
[workspace]
members = [
    "erc-core",           # Shared Core Library
    "erc-budget/src-tauri",   # Budget Application backend
    "erc-execution/src-tauri" # Execution Application backend
]
resolver = "2"
```

If the two applications are in separate repositories, `erc-core` is published as a private crate (or referenced via Git path dependency).

### 4.2 erc-execution/src-tauri Folder Structure

```
erc-execution/src-tauri/src/
├── main.rs                   # Binary entry point
├── lib.rs                    # AppState, plugin registration, command handler list
├── error.rs                  # Re-exports erc_core::error + execution-specific errors
├── domain/
│   ├── mod.rs
│   ├── execution_entities.rs # ExecutionData and all execution-specific entities
│   ├── dto.rs                # Execution-specific DTOs (input/output)
│   └── enums.rs              # Status enums, category enums
├── commands/
│   ├── mod.rs
│   ├── project.rs            # open_project, save_project, get_project_summary
│   ├── persons.rs            # add/update/delete person, link to role
│   ├── person_months.rs      # add/update/delete PM record
│   ├── deliverables.rs       # CRUD deliverables
│   ├── milestones.rs         # CRUD milestones
│   ├── actual_costs.rs       # CRUD actual cost entries
│   ├── travel.rs             # CRUD trip executions
│   ├── equipment.rs          # CRUD equipment procurements
│   ├── subcontracting.rs     # CRUD subcontracting lines
│   ├── risks.rs              # CRUD risk entries
│   ├── issues.rs             # CRUD issue entries
│   ├── periods.rs            # CRUD reporting periods
│   ├── meetings.rs           # CRUD meetings (V2)
│   ├── actions.rs            # CRUD action items (V2)
│   └── documents.rs          # CRUD documents (V2)
├── engines/
│   ├── mod.rs
│   ├── execution_summary.rs  # Master orchestrator → ExecutionSummaryDto
│   ├── financial_engine.rs   # Actual vs. planned calculation
│   ├── progress_engine.rs    # WP/deliverable/milestone status derivation
│   ├── notification_engine.rs # Warning generation
│   └── reporting_engine.rs   # Period report assembly
├── validation/
│   └── mod.rs                # Execution-specific validators
└── persistence/
    └── mod.rs                # load/save using erc-core persistence + execution_data block
```

---

## 5. Application State

### 5.1 Rust AppState

```rust
pub struct AppState {
    /// The open project's budget data (from .ercbudget).
    /// Source of truth for all planned values.
    pub project: Mutex<Option<erc_core::domain::entities::Project>>,

    /// The open project's execution data.
    /// None until the file is first opened by the Execution App.
    pub execution_data: Mutex<Option<ExecutionData>>,

    /// File path of the currently open .ercbudget file.
    pub project_path: Mutex<Option<std::path::PathBuf>>,

    /// EU travel rate tables — embedded, read-only.
    pub rate_data: erc_core::domain::rate_data::RateData,
}
```

### 5.2 Zustand Frontend Stores

Unlike the Budget Application's single monolithic store, the Execution Application uses **domain-scoped stores** to manage the larger state surface:

```typescript
// projectStore.ts — Project identity + planned budget (read-only)
useProjectStore: {
    project: ProjectDto | null;            // From .ercbudget
    budgetSummary: BudgetSummaryDto | null; // Planned budget totals
    projectPath: string | null;
    isLoading: boolean;
    globalError: AppError | null;
}

// executionStore.ts — All live execution state
useExecutionStore: {
    summary: ExecutionSummaryDto | null;   // Full computed state
    activeScreen: ExecutionScreen;
    activePeriodId: string | null;         // Global period filter
    warnings: WarningDto[];
    isDirty: boolean;
}

// uiStore.ts — UI state only
useUiStore: {
    sidebarCollapsed: boolean;
    activeDetailPanel: PanelType | null;
    activePanelEntityId: string | null;
}
```

---

## 6. IPC Command Design

Every command follows the same pattern established in the Budget Application:

```rust
#[tauri::command]
pub async fn add_deliverable(
    state: tauri::State<'_, AppState>,
    input: DeliverableInputDto,
) -> Result<ExecutionSummaryDto, AppError> {
    let project = state.project.lock().unwrap();
    let project = project.as_ref().ok_or(AppError::NoProject)?;

    let mut exec = state.execution_data.lock().unwrap();
    let exec = exec.as_mut().ok_or(AppError::NoProject)?;

    // 1. Validate
    validate_deliverable(&input, project.config.work_package_count, project.config.duration_years * 12)?;

    // 2. Mutate
    let deliverable = Deliverable::from_input(input);
    exec.deliverables.push(deliverable);

    // 3. Auto-save
    let path = state.project_path.lock().unwrap();
    if let Some(p) = path.as_deref() {
        persistence::auto_save(project, exec, p)?;
    }

    // 4. Recalculate summary
    calculate_execution_summary(project, exec, &state.rate_data)
}
```

**All mutating commands return `ExecutionSummaryDto`** — the same full-recalculation pattern as the Budget Application. This ensures the frontend never holds stale state.

---

## 7. Execution Engine Design

### 7.1 Master Orchestrator — `calculate_execution_summary()`

```rust
pub fn calculate_execution_summary(
    project: &Project,
    exec: &ExecutionData,
    rate_data: &RateData,
) -> Result<ExecutionSummaryDto, AppError> {
    // 1. Planned budget (from erc-core)
    let planned = erc_core::calculation::calculate_budget_summary(project, rate_data)?;

    // 2. Actual financials
    let actuals = financial_engine::calculate_actuals(project, exec, rate_data)?;

    // 3. Progress state (WP, deliverables, milestones)
    let progress = progress_engine::calculate_progress(project, exec)?;

    // 4. Warnings
    let warnings = notification_engine::evaluate_warnings(&planned, &actuals, &progress, exec)?;

    // 5. Assemble DTO
    Ok(ExecutionSummaryDto {
        project_info: build_project_info_dto(project),
        planned,
        actuals,
        progress,
        warnings,
        persons: build_person_dtos(exec),
        person_months: build_pm_dtos(exec, project),
        deliverables: build_deliverable_dtos(exec),
        milestones: build_milestone_dtos(exec),
        risks: build_risk_dtos(exec),
        issues: build_issue_dtos(exec),
        periods: build_period_dtos(exec),
        current_project_month: calculate_current_month(project, exec),
    })
}
```

### 7.2 Financial Engine

```rust
pub fn calculate_actuals(
    project: &Project,
    exec: &ExecutionData,
    rate_data: &RateData,
) -> Result<ActualFinancialsDto, AppError> {
    // Category A: sum of (approved PM records × inflation-adjusted monthly salary)
    let a_actual = calculate_actual_personnel_cost(project, exec)?;

    // Category B: sum of approved subcontracting line amounts
    let b_actual = exec.subcontracting_lines
        .iter()
        .filter(|l| l.approved)
        .map(|l| l.amount_eur)
        .sum();

    // Category C1: sum of approved trip execution actual costs
    let c1_actual = calculate_actual_travel_cost(exec)?;

    // Category C2: sum of actual eligible depreciation (re-run CALC-05 with actual purchase cost)
    let c2_actual = calculate_actual_equipment_cost(project, exec)?;

    // Category C3: sum of approved actual cost entries with category = C3
    let c3_actual = exec.actual_cost_entries
        .iter()
        .filter(|e| e.category == CostCategory::C3 && e.status == EntryStatus::Approved)
        .map(|e| e.actual_amount_eur)
        .filter_map(|v| v)
        .sum();

    // Category E: indirect = (A + C1 + C2 + C3 actuals) × rate
    let e_actual = (a_actual + c1_actual + c2_actual + c3_actual)
        * (project.config.indirect_cost_rate_pct / Decimal::ONE_HUNDRED);

    // Totals
    let total_direct = a_actual + b_actual + c1_actual + c2_actual + c3_actual;
    let total_eligible = total_direct + e_actual;
    let eu_contribution = total_eligible; // 100% funding rate

    // CFS re-check
    let cfs_result = erc_core::calculation::cfs_checker::check_cfs_threshold(
        eu_contribution,
        project.has_cfs_item(),
        project.cfs_warning_dismissed,
    )?;

    // Per-WP actual breakdown
    let wp_actuals = calculate_wp_actuals(project, exec)?;

    Ok(ActualFinancialsDto {
        a_actual, b_actual, c1_actual, c2_actual, c3_actual,
        e_actual, total_direct, total_eligible, eu_contribution,
        cfs_result, wp_actuals,
    })
}
```

### 7.3 Progress Engine

```rust
pub fn calculate_progress(
    project: &Project,
    exec: &ExecutionData,
) -> Result<ProgressDto, AppError> {
    // Current project month (derived from today's date + project start context)
    let current_month = derive_current_project_month(exec)?;

    // WP status derivation
    let wp_statuses = (1..=project.config.work_package_count)
        .map(|wp_id| derive_wp_status(wp_id, project, exec, current_month))
        .collect::<Result<Vec<_>, _>>()?;

    // Deliverable status (auto-flag overdue)
    let deliverable_statuses = exec.deliverables.iter()
        .map(|d| derive_deliverable_status(d, current_month))
        .collect();

    // Milestone status (auto-flag at-risk)
    let milestone_statuses = exec.milestones.iter()
        .map(|m| derive_milestone_status(m, &exec.deliverables, current_month))
        .collect();

    Ok(ProgressDto { current_month, wp_statuses, deliverable_statuses, milestone_statuses })
}
```

### 7.4 Notification Engine

```rust
pub fn evaluate_warnings(
    planned: &BudgetSummaryDto,
    actuals: &ActualFinancialsDto,
    progress: &ProgressDto,
    exec: &ExecutionData,
) -> Result<Vec<WarningDto>, AppError> {
    let mut warnings = Vec::new();

    // W-01: Overdue deliverables
    for d in &exec.deliverables {
        if d.actual_submission_date.is_none() && progress.current_month > d.planned_month {
            warnings.push(WarningDto::new("W-01", WarningSeverity::Error,
                format!("{} is overdue (due M{})", d.deliverable_number, d.planned_month),
                NavigationTarget::Deliverables));
        }
    }

    // W-05: Category overrun > 15%
    for (planned_cat, actual_cat, label) in budget_category_pairs(planned, actuals) {
        if planned_cat > Decimal::ZERO && actual_cat > planned_cat * dec!(1.15) {
            warnings.push(WarningDto::new("W-05", WarningSeverity::Warning,
                format!("{} actual exceeds planned by >15%", label),
                NavigationTarget::FinancialReport));
        }
    }

    // ... all W-01 through W-12 checks

    Ok(warnings)
}
```

---

## 8. Persistence Layer

The Execution Application's persistence extends the `erc-core` persistence module:

```rust
// erc-execution/src-tauri/src/persistence/mod.rs

pub fn save_execution(
    project: &Project,
    exec: &ExecutionData,
    path: &Path,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let created_at = read_created_at(path).unwrap_or_else(|_| now.clone());

    let file = ProjectFile {
        format_version: EXECUTION_FORMAT_VERSION.to_string(), // "1.1"
        created_at,
        updated_at: now,
        project: project.clone(),
        execution_data: Some(exec.clone()),
    };

    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| AppError::Persistence(format!("Serialisation failed: {e}")))?;

    std::fs::write(path, json.as_bytes())
        .map_err(|e| AppError::Persistence(format!("Write failed: {e}")))?;

    Ok(())
}

pub fn load_execution(path: &Path) -> Result<(Project, ExecutionData), AppError> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| AppError::Persistence(format!("Read failed: {e}")))?;

    let file: ProjectFile = serde_json::from_str(&json)
        .map_err(|e| AppError::Persistence(format!("Parse failed: {e}")))?;

    // Validate format version
    match file.format_version.as_str() {
        "1.0" | "1.1" => {},
        v => return Err(AppError::Persistence(format!(
            "Unsupported format version: {v}"
        ))),
    }

    let exec = file.execution_data.unwrap_or_default();
    Ok((file.project, exec))
}
```

---

## 9. Export Engine

### 9.1 Financial Report (Excel)

A multi-sheet ExcelJS workbook:
- **Sheet 1**: Summary — Planned vs. Actual per category + variance
- **Sheet 2**: Personnel — PM records + salary cost breakdown per person per period
- **Sheet 3**: Travel — Trip execution records vs. planned
- **Sheet 4**: Equipment — Procurement records vs. planned depreciation
- **Sheet 5**: Other Costs — Actual C3 entries vs. planned items
- **Sheet 6**: Per-WP breakdown — Actual costs allocated to each WP

### 9.2 Technical Report Annex (Excel)

Two sheets:
- **Deliverables**: Number, title, WP, type, planned month, actual submission date, status
- **Milestones**: Title, WP, planned month, actual completion, linked deliverables, status

### 9.3 Person-Month Declaration (Excel)

One sheet per reporting period:
- Role, planned PM, reported PM, approved PM, cost estimate
- Pre-filled with approved records; empty cells for unsubmitted entries

### 9.4 PDF Export

Project status dashboard rendered via HTML + `window.print()`:
- Project header
- Budget health summary (planned vs. actual)
- WP status overview
- Open risks and issues count
- Upcoming deadlines

---

## 10. Validation Engine

The validation module imports from `erc-core` for all shared validators and adds execution-specific validators:

```rust
// Shared validators (from erc-core):
use erc_core::validation::{
    validate_personnel_role,
    validate_equipment_item,
    validate_trip,
    validate_other_cost,
    validate_project_config,
};

// Execution-specific validators:
pub fn validate_person(dto: &PersonInputDto, roles: &[PersonnelRole]) -> Result<(), AppError>;
pub fn validate_person_month_record(dto: &PMRecordInputDto, exec: &ExecutionData, project: &Project) -> Result<(), AppError>;
pub fn validate_deliverable(dto: &DeliverableInputDto, wp_count: u8, max_month: u32) -> Result<(), AppError>;
pub fn validate_milestone(dto: &MilestoneInputDto, wp_count: u8, max_month: u32) -> Result<(), AppError>;
pub fn validate_reporting_periods(periods: &[ReportingPeriod], project: &ProjectConfig) -> Result<(), AppError>;
pub fn validate_actual_cost_entry(dto: &ActualCostEntryInputDto, exec: &ExecutionData) -> Result<(), AppError>;
pub fn validate_risk_entry(dto: &RiskEntryInputDto) -> Result<(), AppError>;
pub fn validate_subcontracting_line(dto: &SubcontractingLineInputDto, planned: Decimal, existing_total: Decimal) -> Result<(), AppError>;
```

---

## 11. Key DTOs

### 11.1 ExecutionSummaryDto (master output)

```typescript
interface ExecutionSummaryDto {
    project_info: ProjectInfoDto;
    current_project_month: number;
    planned: BudgetSummaryDto;         // From erc-core (unchanged)
    actuals: ActualFinancialsDto;
    progress: ProgressDto;
    warnings: WarningDto[];
    persons: PersonDetailDto[];
    person_months: PersonMonthDetailDto[];
    deliverables: DeliverableDetailDto[];
    milestones: MilestoneDetailDto[];
    risks: RiskEntryDetailDto[];
    issues: IssueEntryDetailDto[];
    periods: ReportingPeriodDetailDto[];
}
```

### 11.2 ActualFinancialsDto

```typescript
interface ActualFinancialsDto {
    a_actual: string;         // Decimal string
    b_actual: string;
    c1_actual: string;
    c2_actual: string;
    c3_actual: string;
    e_actual: string;
    total_direct: string;
    total_eligible: string;
    eu_contribution: string;
    cfs_status: CfsStatus;
    wp_actuals: WpActualDto[];
    period_actuals: PeriodActualDto[];
}
```

### 11.3 WarningDto

```typescript
interface WarningDto {
    code: string;                    // W-01 through W-12
    severity: 'Error' | 'Warning' | 'Info';
    message: string;
    navigation_target: NavigationTarget;
    entity_id: string | null;        // UUID of the specific entity if applicable
}
```

---

## 12. Folder Structure (Full Application)

```
erc-execution/
├── src/                              # React / TypeScript frontend
│   ├── App.tsx                       # Root layout + navigation
│   ├── main.tsx
│   ├── screens/
│   │   ├── Welcome.tsx
│   │   ├── Dashboard.tsx
│   │   ├── WorkPackages.tsx
│   │   ├── Deliverables.tsx
│   │   ├── Milestones.tsx
│   │   ├── FinancialReport.tsx
│   │   ├── Personnel.tsx
│   │   ├── Travel.tsx
│   │   ├── Equipment.tsx
│   │   ├── OtherCosts.tsx
│   │   ├── Subcontracting.tsx
│   │   ├── RiskRegister.tsx
│   │   ├── IssueLog.tsx
│   │   ├── ReportingPeriods.tsx
│   │   ├── Meetings.tsx              [V2]
│   │   ├── ActionItems.tsx           [V2]
│   │   ├── Documents.tsx             [V2]
│   │   └── ReportsExport.tsx
│   ├── components/
│   │   ├── shared/                   # Shared with Budget App (copy or package)
│   │   │   ├── FormField.tsx
│   │   │   ├── EmptyStateCard.tsx
│   │   │   ├── WarningBanner.tsx
│   │   │   └── LivePreviewBox.tsx
│   │   ├── execution/                # Execution-specific
│   │   │   ├── StatusBadge.tsx
│   │   │   ├── PlannedActualRow.tsx
│   │   │   ├── ProgressBar.tsx
│   │   │   ├── WpCard.tsx
│   │   │   ├── DeliverableCard.tsx
│   │   │   ├── MilestoneCard.tsx
│   │   │   ├── RiskCard.tsx
│   │   │   ├── NotificationTray.tsx
│   │   │   ├── PeriodSelector.tsx
│   │   │   ├── BudgetHealthGauge.tsx
│   │   │   ├── CategoryBar.tsx
│   │   │   └── SidePanel.tsx         # Slide-in edit panel
│   │   └── charts/
│   │       ├── BudgetHealthChart.tsx
│   │       ├── WpBudgetChart.tsx
│   │       └── RiskMatrix.tsx
│   ├── store/
│   │   ├── projectStore.ts           # Planned budget + project info
│   │   ├── executionStore.ts         # Live execution state
│   │   └── uiStore.ts                # UI-only state
│   ├── ipc/
│   │   └── commands.ts               # All Tauri invoke() wrappers
│   ├── hooks/
│   │   ├── useAutoSave.ts
│   │   ├── useWarnings.ts            # Derived from executionStore
│   │   └── usePeriodFilter.ts        # Active period filter hook
│   ├── export/
│   │   ├── financialReportExporter.ts
│   │   ├── technicalReportExporter.ts
│   │   ├── pmDeclarationExporter.ts
│   │   ├── statusReportPdf.ts
│   │   └── riskRegisterExporter.ts
│   ├── validators/
│   │   └── schemas.ts
│   └── types/
│       └── index.ts                  # Execution-specific TS types
├── src-tauri/
│   ├── src/                          # Rust backend (see §4.2)
│   ├── resources/
│   │   └── eu_travel_rates/          # Same embedded JSON as Budget App
│   └── tauri.conf.json
├── Cargo.toml                         # Declares erc-core dependency
└── package.json
```

---

## 13. Shared Core Integration (Rust)

In `erc-execution/src-tauri/Cargo.toml`:

```toml
[package]
name = "erc-execution"
version = "1.0.0"

[dependencies]
erc-core = { path = "../../erc-core" }
tauri = { version = "2", features = [] }
# ... same plugin dependencies as erc-budget
```

The Execution Application never re-implements anything that exists in `erc-core`. All shared types are imported via `use erc_core::...`.

---

## 14. TypeScript Type Sharing

Two options (in order of preference):

**Option A — ts-rs code generation (Recommended)**
The `ts-rs` Rust crate generates TypeScript types from Rust structs automatically. This ensures zero divergence between Rust DTOs and TypeScript types.

```toml
# erc-core/Cargo.toml
[dev-dependencies]
ts-rs = "10"
```

```rust
#[derive(TS)]
#[ts(export)]
pub struct DeliverableDetailDto { ... }
```

Generated TypeScript is placed in a shared `@erc/core-types` package that both apps import.

**Option B — Manual sync (Fallback)**
TypeScript types are maintained by hand in `types/index.ts` in each application, using the Rust DTOs as the authoritative spec.

---

## 15. CI/CD Pipeline

```yaml
# .github/workflows/build.yml
jobs:
  test-core:
    runs-on: ubuntu-latest
    steps:
      - cargo test -p erc-core

  test-execution-backend:
    needs: test-core
    runs-on: ubuntu-latest
    steps:
      - cargo test -p erc-execution

  test-execution-frontend:
    runs-on: ubuntu-latest
    steps:
      - pnpm test --project erc-execution

  build-macos:
    needs: [test-core, test-execution-backend, test-execution-frontend]
    runs-on: macos-latest
    steps:
      - pnpm tauri build

  build-windows:
    needs: [test-core, test-execution-backend, test-execution-frontend]
    runs-on: windows-latest
    steps:
      - pnpm tauri build
```

---

## 16. Security Considerations

The Execution Application handles financially sensitive project data. Key security measures:

- **File-level access only**: Tauri capabilities restrict file system access to user-selected paths only
- **No network traffic**: All operations are local; the updater endpoint is the only outbound connection (same as Budget App)
- **No credentials stored**: The application stores no passwords, API keys, or authentication tokens
- **No code injection**: All user-provided strings are serialized to JSON and never executed
- **Audit trail**: `created_at` / `updated_at` timestamps on all execution entities provide basic audit capability

---

## 17. Open Questions

1. Should `erc-core` be published to a private crate registry, or referenced via path/git dependency?
2. Should the Execution Application auto-detect the project's start date (to compute current project month) from the call opening date, or require the user to set it?
3. Is a SQLite database preferable to JSON for the execution data at scale? (JSON is sufficient for single-project MVP; SQLite would be needed for multi-project portfolio management.)
4. Should the Execution App support multi-window (one window per WP)?

---

## 18. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| `erc-core` API changes break both apps simultaneously | High | Semantic versioning; compatibility tests in both app CI pipelines |
| Large execution data sets slow down full-recalculation | Medium | Profile and add lazy evaluation for large lists; target < 100ms recalculation |
| Tauri v3 releases before development completes | Low | Monitor Tauri roadmap; architecture is framework-agnostic below the command layer |

---

## 19. Confidence Level

**88%** — The architecture follows proven patterns from the Budget Application and extends them cleanly. The main uncertainty is the optimal approach to TypeScript type sharing between two apps (ts-rs vs. manual). The financial calculation engine design is high-confidence given the shared core approach.

---

## 20. Recommended Next Step

**Proceed to Phase 07 — Shared Core Refactoring Plan.**

The architecture is defined. Phase 07 specifies exactly what must be extracted from the existing Budget Application into `erc-core` and how to do so without breaking the Budget App.
