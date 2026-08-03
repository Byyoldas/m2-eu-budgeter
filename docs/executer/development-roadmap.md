# Development Roadmap

**Phase 08 — Development Roadmap**
**Project:** Horizon Europe Project Management Platform
**Date:** 2026-08-03

---

## 1. Overview

This roadmap sequences all development work for the HE Project Management Platform from the current state (Budget Application v1.7.0) through the first public release of the Execution Application. It covers `erc-core` extraction, `erc-execution` implementation, testing, and release.

---

## 2. Versioning Strategy

| Product | Current | After Extraction | First Execution Release |
|---|---|---|---|
| `erc-core` | (does not exist) | 0.1.0 | 1.0.0 |
| `erc-budget` | 1.7.0 | 1.8.0 (internal refactor) | 1.8.x (maintenance only) |
| `erc-execution` | (does not exist) | — | 1.0.0 |

**Release naming convention:**
- `erc-budget` v1.8.0 = internal refactor using `erc-core`; no user-visible feature changes
- `erc-execution` v1.0.0 = MVP release with modules M-01 through M-10 (core execution tracking)
- `erc-execution` v1.1.0 = follow-on with remaining modules and reporting

---

## 3. Milestone Map

```
NOW ──────────────────────────────────────────────────────────────► RELEASE

  │  M0: Workspace Setup    │  M1: erc-core         │  M2: Execution MVP  │  M3: Release
  │  1 week                 │  3 weeks              │  10 weeks           │  2 weeks
  │                         │                       │                     │
  ├── Cargo workspace       ├── Error module        ├── M-01 Dashboard    ├── RC1 candidates
  ├── erc-core skeleton     ├── Domain entities     ├── M-02 WP Tracking  ├── Beta testing
  ├── CI pipeline setup     ├── Calculations        ├── M-03 Personnel    ├── Docs
  ├── Test fixture library  ├── Validation          ├── M-04 Expenditure  └── GA release
  └── Budget App regression ├── Persistence         ├── M-05 Budget View  
                            ├── Shared TS types     ├── M-06 Documents    
                            └── Budget App v1.8.0   ├── M-07 Reporting    
                                                    ├── M-08 Partners     
                                                    ├── M-09 Timeline     
                                                    └── M-10 Audit        
```

---

## 4. Sprint Plan

### Milestone 0: Workspace Setup (Week 1)

**Goal:** Working Cargo workspace with `erc-core` skeleton and CI passing for the Budget App.

| Sprint | Tasks | Deliverable |
|---|---|---|
| M0-S1 | Create `erc-platform` workspace; move Budget App into `erc-budget/`; create `erc-core/` skeleton | Workspace builds; Budget App tests green |
| M0-S1 | Write test fixture library (`test-fixtures/*.ercbudget`); add round-trip regression tests | Regression baseline established |
| M0-S1 | Set up GitHub Actions CI: workspace build, unit tests, `cargo clippy`, `cargo fmt --check` | CI green on every PR |

**Exit criteria:**
- `cargo test --workspace` passes with 0 failures
- `pnpm test` (Budget App) passes with 0 failures
- CI runs on pull requests and `main` branch

---

### Milestone 1: erc-core Extraction (Weeks 2–4)

**Goal:** All shared business logic lives in `erc-core`. Budget App becomes a thin shell. No user-visible changes.

#### Week 2 — Rust Core (Steps 1–4)

| Day | Step | Description |
|---|---|---|
| Mon | Step 1 | Extract `error.rs` → green |
| Mon | Step 2 | Extract `rate_data.rs` + resources → green |
| Tue | Step 3 | Extract `domain/entities.rs` + promote WorkPackage → green |
| Wed | Step 4 | Extract shared DTOs → green |
| Thu–Fri | Buffer | Fix any unexpected issues; code review |

**Exit criteria for Week 2:**
- All extraction steps complete
- Calculation tests all pass in `erc-core`
- Budget App tests still pass (re-export shims working)

#### Week 3 — Rust Calculation + Validation + Persistence (Steps 5–7)

| Day | Step | Description |
|---|---|---|
| Mon | Step 5 | Extract `validation/mod.rs` with all 35+ tests |
| Tue–Wed | Step 6 | Extract all `calculation/` modules (~100 tests move to erc-core) |
| Thu | Step 7 | Extract `persistence/mod.rs`; add optional `execution_data` field |
| Fri | — | Full regression: Budget App smoke test with real `.ercbudget` files |

**Exit criteria for Week 3:**
- `cargo test --workspace` with 0 failures including all moved tests
- v1.0 `.ercbudget` files load correctly in refactored Budget App
- Budget App can create a project, add roles/equipment/travel, save, and reload

#### Week 4 — TypeScript Shared Types (Steps 8–10)

| Day | Step | Description |
|---|---|---|
| Mon–Tue | Step 8 | Set up `ts-rs`; generate shared TypeScript types from Rust structs |
| Wed | Step 9 | Extract shared Zod schemas |
| Thu | Step 10 | Identify + extract generic UI components |
| Fri | — | Budget App v1.8.0 build + release preparation |

**Exit criteria for Milestone 1:**
- `erc-core` v0.1.0 published (internal)
- Budget App v1.8.0 builds and passes all tests
- No user-visible changes to Budget App behavior
- TypeScript types in `@erc/core-types` and Budget App imports from there

---

### Milestone 2: Execution Application MVP (Weeks 5–14)

**Goal:** erc-execution v1.0.0 — working Execution Application covering core tracking workflows.

The 23 modules from Phase 04 are grouped into 5 sprints by dependency:

#### Sprint E1 (Weeks 5–6): Application Shell + Core Infrastructure

**Goal:** The Execution Application opens, reads a `.ercbudget` file, and shows a dashboard.

| Task | Description | Module |
|---|---|---|
| Create `erc-execution/` with Tauri + React scaffold | Identical setup to Budget App | — |
| IPC command: `open_execution_project` | Load `.ercbudget` file; read execution_data block | M-01 |
| IPC command: `save_execution_project` | Write execution_data back to file | M-01 |
| AppState for Execution App | Project, execution_data, path, loading state | — |
| Zustand store: `executionStore` | Core state management | — |
| Dashboard screen scaffold | Static layout; no data yet | M-01 |
| Sidebar + navigation | Left panel with module list | — |
| Tauri window config | 1440×900 minimum; app identifier `com.erc.execution` | — |
| Error handling | Shared `AppError` from `erc-core::error` | — |
| Auto-save hook | 2s debounce; Rust `auto_save` | — |

**Exit criteria:**
- App opens a `.ercbudget` file, reads project config, displays project name
- Dashboard shows correct budget summary (recomputed from `erc-core`)
- File saves and reloads without data loss

#### Sprint E2 (Weeks 7–8): Work Package Tracking (M-02, M-03, M-04)

| Module | Description | Key entities |
|---|---|---|
| M-02: Work Package Tracker | Start/end date tracking; status per WP; milestone management | `WorkPackageExecution` |
| M-03: Personnel Tracking | Team roster; FTE vs planned; monthly allocation actuals | `PersonnelRecord` |
| M-04: Amendment Management | Log scope/budget amendments; version history | `Amendment` |

**Key IPC commands:**
- `update_work_package_status`
- `add_milestone`, `complete_milestone`
- `add_personnel_record`, `update_personnel_record`
- `record_amendment`

**Exit criteria:**
- User can update WP status, record milestones, log team members
- All data persists across app restarts
- Validation errors surface inline

#### Sprint E3 (Weeks 9–10): Financial Tracking (M-05, M-06, M-07, M-08)

| Module | Description | Key entities |
|---|---|---|
| M-05: Expenditure Tracker | Record actual expenditures; categorise to ERC budget categories | `ExpenditureRecord` |
| M-06: Budget vs. Actuals | Real-time budget vs. actuals by category and WP | (computed) |
| M-07: Financial Forecasting | Project year-end position based on actuals + remaining burn rate | (computed) |
| M-08: Cost Category Compliance | Flag overspend vs. lump sum; alert when approaching limits | (computed) |

**Key IPC commands:**
- `add_expenditure`, `update_expenditure`, `delete_expenditure`
- `get_financial_summary`
- `get_forecast`
- `check_compliance`

**Calculation engine additions (EXEC-CALC-01 through EXEC-CALC-05):**
See `/docs/execution-requirements.md` Module M-06 for full specs.

**Exit criteria:**
- User can enter actual expenditures with date, amount, category, WP
- Budget vs. actuals display is live (updates on entry)
- Forecast updates automatically
- Warning shown when category approaches 90% of budgeted amount

#### Sprint E4 (Weeks 11–12): Documents + Reporting (M-09, M-10, M-11, M-12)

| Module | Description |
|---|---|
| M-09: Document Repository | Link external documents (reports, deliverables) to WPs |
| M-10: Progress Reporting | Generate periodic progress summaries |
| M-11: Financial Reports | Generate ERC-compliant financial status reports |
| M-12: Excel Export | Export financial summary to multi-sheet Excel workbook |

**Excel export spec (M-12):**
Reuses ExcelJS (already a Budget App dependency). Sheets:
1. Executive Summary
2. Budget vs. Actuals by Category
3. Budget vs. Actuals by Work Package
4. Expenditure Ledger
5. Forecast

**Exit criteria:**
- User can generate and export a financial report
- Excel export produces valid `.xlsx` file matching the report data
- Document links persist across app restarts

#### Sprint E5 (Weeks 13–14): Polish, Edge Cases, Integration (M-13 through M-23)

Focus areas:
- Remaining modules from Phase 04 requirements (M-13 through M-23) at MVP scope
- Edge case handling across all modules
- Performance testing with large expenditure ledgers (1000+ records)
- Accessibility review (keyboard navigation, screen reader labels)
- Error state handling (file not found, corrupted file, missing budget data)
- Loading states and optimistic UI
- Empty state designs (new project with no execution data)

---

### Milestone 3: Release Preparation (Weeks 15–16)

#### Week 15: Beta + Stabilisation

| Task | Description |
|---|---|
| Internal beta | Test with 3–5 real ERC/HE project budgets |
| Regression testing | Full test suite; manual walkthrough of all modules |
| Performance profiling | Target <100ms for all IPC commands; <16ms for UI updates |
| Security review | No credentials stored; no network calls from app |
| Installer builds | Windows (NSIS), macOS (DMG) via GitHub Actions |

#### Week 16: Documentation + Release

| Task | Description |
|---|---|
| User Manual | Complete end-to-end guide for all modules |
| Release notes | What's new, known issues, upgrade instructions |
| GitHub Release | Tag `erc-execution/v1.0.0`; attach installers |
| Budget App compatibility notice | erc-budget v1.8.0 files are compatible with erc-execution v1.0.0 |

---

## 5. Dependency Map

```
erc-core v0.1.0
    └── required by erc-budget v1.8.0  (extraction complete)
    └── required by erc-execution v1.0.0  (new app)
    
erc-budget v1.8.0
    └── no longer developed; maintenance only
    
erc-execution v1.0.0
    └── requires: erc-core v0.1.0
    └── requires: Sprint E1 (shell) before E2, E3, E4, E5
    └── Sprint E2 and E3 can be developed in parallel after E1
    └── Sprint E4 requires E3 (financial data must exist before reporting)
    └── Sprint E5 requires E2, E3, E4 complete
```

---

## 6. Testing Strategy

### 6.1 Test Pyramid

```
                   ┌───────────────────┐
                   │   E2E / Manual    │  (5%)  — Full install + real file walkthroughs
                   ├───────────────────┤
                   │  Integration      │  (25%) — IPC command layer; file round-trips
                   ├───────────────────┤
                   │  Unit Tests       │  (70%) — All calculation + validation functions
                   └───────────────────┘
```

### 6.2 Unit Tests

All calculation functions in `erc-core::calculation` are covered by unit tests. Minimum requirement: one happy-path test + one edge-case test per function.

Calculation reference values are documented in `test-fixtures/` expected output JSON files. These are regression tests — they must not change unless a deliberate specification change is approved.

### 6.3 Integration Tests

Each IPC command has at least one integration test:
- Load a project → verify state matches expected
- Mutate via command → verify returned summary is correct
- Save → reload → verify round-trip fidelity

### 6.4 Validation Tests

All 35+ validators from `erc-core::validation` are tested. Each test verifies:
- Valid input produces no errors
- Each invalid condition produces the correct `FieldError` code

### 6.5 Execution Calculation Tests

New calculation functions added for the Execution Application (EXEC-CALC-01 through EXEC-CALC-N) follow the same pattern: pure functions, no IO, tested with reference values.

### 6.6 Frontend Tests (Vitest + React Testing Library)

Frontend tests cover:
- Form validation (Zod schemas)
- Store state transitions (Zustand)
- Component rendering (happy path + empty state + error state)
- IPC mock: `vi.mock('../ipc/commands')` for all command wrappers

### 6.7 CI Gates

Every pull request must pass:
1. `cargo fmt --check` (Rust formatting)
2. `cargo clippy -- -D warnings` (linting, no warnings allowed)
3. `cargo test --workspace` (all unit + integration tests)
4. `pnpm lint` (ESLint)
5. `pnpm test` (Vitest)
6. `pnpm build` (production build, no type errors)

Merging to `main` without passing all CI gates is blocked.

---

## 7. Risk Register

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| erc-core extraction takes > 3 weeks | Medium | High (delays Execution App) | Begin E1 shell in parallel with extraction Week 3–4; E1 doesn't need erc-core fully extracted |
| Budget App regression during extraction | Medium | High | Regression test suite established in M0; revert + fix policy |
| File format compatibility broken | Low | Critical | Round-trip test with real `.ercbudget` files before and after every persistence change |
| ERC reporting rules change during development | Medium | Medium | Reporting rules isolated to `erc-core::rules` module; single update point |
| Tauri 2.x API breaking change | Low | Medium | Pin Tauri version in workspace; monitor release notes; upgrade only on minor releases |
| TypeScript / Rust type drift | Medium | Medium | `ts-rs` automation eliminates manual sync; CI gate fails on generated type mismatch |
| Excel export compatibility issue | Low | Low | Test with LibreOffice + Excel 365 + Excel 2019 |
| Performance degradation with large ledgers | Low | Medium | Performance test with 1000+ expenditure records in Sprint E5 |

---

## 8. Migration Strategy for End Users

### 8.1 Budget App Users → Execution App

1. User continues using Budget App v1.8.0 for budget planning (no change)
2. User installs Execution App (separate installer)
3. User opens their existing `.ercbudget` file in the Execution App
4. The Execution App reads the project and budget data; creates empty execution_data block
5. Execution data is written back to the same `.ercbudget` file on save
6. Budget App v1.8.0 can still open the file (execution_data block is ignored by Budget App)

### 8.2 File Compatibility Matrix

| File version | Budget App v1.7.0 | Budget App v1.8.0 | Execution App v1.0.0 |
|---|---|---|---|
| v1.0 (budget only) | ✅ Full | ✅ Full | ✅ Reads budget; creates execution_data |
| v1.1 (with execution_data) | ❌ Cannot open | ✅ Opens; ignores execution_data | ✅ Full |

> **Note:** Budget App v1.8.0 must be released before or alongside Execution App v1.0.0 to prevent Budget App v1.7.0 users from encountering files they cannot open.

---

## 9. Release Strategy

### 9.1 Release Channels

| Channel | Audience | Cadence |
|---|---|---|
| `stable` | All users | Major + minor releases |
| `beta` | Internal testers | Release candidate builds |
| `nightly` | Developers | Automated nightly builds on `main` |

### 9.2 Auto-update

Both applications use Tauri's built-in updater plugin (already wired in Budget App). Update manifests are hosted on GitHub Releases. Users are notified of updates on launch.

### 9.3 Release Sequence

1. `erc-core` v0.1.0 → internal (not user-facing)
2. `erc-budget` v1.8.0 → beta → stable
3. `erc-execution` v1.0.0 → beta → stable (announced together with Budget App v1.8.0 or after)

---

## 10. Success Criteria

### MVP Success (erc-execution v1.0.0)

The MVP release is successful if:

1. A user with an existing `.ercbudget` file can open it in the Execution Application without error
2. The user can track actual expenditures and see budget vs. actuals in real time
3. The user can record work package progress and milestone completions
4. The user can export a financial summary to Excel
5. All data persists reliably (no data loss across app restarts)
6. The Budget Application continues to work correctly for all existing users (zero regressions)
7. All CI checks pass with 0 failures
8. p95 IPC latency is < 100ms for all commands
9. App cold start time is < 2 seconds on reference hardware

### Phase Completion Criteria

| Phase | Done when... |
|---|---|
| Phase 07 (this phase) | `/docs/shared-core-roadmap.md` written and approved |
| Phase 08 (this phase) | `/docs/development-roadmap.md` written and approved |
| Phase 09 (Implementation) | `erc-core` v0.1.0 extracted; `erc-budget` v1.8.0 tests green; `erc-execution` Shell (Sprint E1) functional |
| Phase 10 (Testing) | Full test suite written; CI gates passing; performance targets met |
| Phase 11 (Documentation) | User Manual, Developer Guide, Architecture Guide complete |

---

## 11. Open Questions

1. Should `erc-execution` v1.0.0 include all 23 modules (Phase 04), or ship with M-01 through M-12 first?
2. Is a web version of the Execution Application in scope for v1.x? (Architecture permits it via WASM but it is not planned.)
3. Should the Execution App support multiple projects open simultaneously? (Current design: one at a time, same as Budget App.)
4. What is the expected number of internal beta testers, and do they have existing `.ercbudget` files available?

---

## 12. Assumptions

- One developer working full-time on this project
- No new features added to Budget App during extraction period
- ERC/HE lump sum rules do not change during development (v1.0.0 targets current rules as of 2026)
- Target platforms: macOS (Apple Silicon + Intel) and Windows 10/11 x64
- Linux not targeted for v1.0.0

---

## 13. Confidence Level

**88%** — The timeline is achievable for a single focused developer. The main uncertainty is Sprint E2–E3 scope: the execution tracking modules have less prior art to learn from (the Budget App was the reference), so estimation confidence is lower there. A 2-week buffer is built in.

---

## 14. Recommended Next Step

**Proceed to Phase 09 — Implementation.**

All analysis, specification, UX design, architecture, shared-core planning, and roadmap phases are complete. The next action is to begin coding, starting with **Milestone 0** (workspace setup) followed by **Milestone 1** (erc-core extraction).
