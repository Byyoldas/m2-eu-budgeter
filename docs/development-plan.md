# Development Plan

**Document:** TASK-09 Deliverable  
**Date:** 2026-07-10  
**Status:** Draft — Awaiting Approval Before Proceeding to TASK-10  
**Source documents:** architecture.md, calculation-engine.md, ux-design.md, domain-model.md

---

## 1. Project Context

This plan covers the full development lifecycle of the ERC Budget application from environment setup through production release. The application is a cross-platform desktop tool (Tauri v2 + Rust + TypeScript/React) that replaces an Excel workbook for ERC Actual Costs grant budgeting.

**Assumed team size:** 2–3 developers (1 Rust-primary, 1–2 TypeScript/React-primary). The plan scales to a solo developer by treating each sprint as a longer time block.

**Reference velocity assumption:** 2-week sprints. The full plan is 10 sprints (20 weeks / ~5 months) from environment setup to public release. Adjust sprint length to team size.

**No code exists yet.** The plan begins from zero.

---

## 2. Milestones

Five milestones gate progression. No milestone may be entered until its predecessor is accepted.

| ID | Milestone | Sprint | Exit Criteria |
|---|---|---|---|
| M-01 | Foundation Ready | End of Sprint 2 | Project scaffolding complete; Rust IPC round-trip working; React shell renders; CI pipeline green |
| M-02 | Core Engine Complete | End of Sprint 5 | All 19 CALC functions implemented and passing unit tests; personnel, equipment, and travel calculations verified against worked examples in calculation-engine.md |
| M-03 | Full Feature Complete | End of Sprint 7 | All 8 wizard screens implemented; all 23 business rules enforced; BudgetSummary live-updates on every form save |
| M-04 | Quality Gate | End of Sprint 9 | All test categories (unit, integration, calculation, validation, regression) green; export produces correct xlsx/PDF/CSV; no P1/P2 bugs open |
| M-05 | Release Ready | End of Sprint 10 | Signed installers built for Windows and macOS; documentation complete; final acceptance test passed by PI |

---

## 3. Sprint Plan

### Sprint 1 — Environment & Scaffolding (Weeks 1–2)

**Goal:** Every developer can build, run, and test the project locally. CI/CD pipeline is green on an empty app.

**Deliverables:**

- Tauri v2 project initialised (`cargo tauri init`) with React + TypeScript frontend via Vite
- Rust workspace structure created:
  ```
  src-tauri/src/
  ├── commands/mod.rs     (empty stubs)
  ├── domain/mod.rs       (empty stubs)
  ├── calculation/mod.rs  (empty stubs)
  ├── validation/mod.rs   (empty stubs)
  └── persistence/mod.rs  (empty stubs)
  ```
- Frontend directory structure created (screens/, components/, store/, hooks/, ipc/, validators/, export/)
- `rust_decimal` and `serde` added to `Cargo.toml`
- React, Zustand, React Hook Form, Zod, Recharts, Radix UI added to `package.json`
- Tauri IPC round-trip smoke test: a `ping` command returns `"pong"` from Rust to TypeScript
- GitHub Actions (or equivalent) CI: `cargo test`, `cargo clippy`, `pnpm test`, `pnpm typecheck` all green on push to main
- `docs/` folder committed to repository

**Dependencies:** None.

**Definition of Done:** CI pipeline green; `pnpm tauri dev` launches a window showing "ERC Budget App" placeholder text.

---

### Sprint 2 — Domain Layer & Persistence Foundation (Weeks 3–4)

**Goal:** All domain entities exist as Rust types. Project files can be saved and loaded.

**Deliverables:**

- Domain entities implemented in Rust (all structs from domain-model.md):
  - `Project`, `ProjectConfig`, `PersonnelRole`, `EquipmentItem`, `Trip` (with `TripType` enum), `OtherDirectCostItem`, `Subcontracting`
  - All value types: `RoleType` enum, `Decimal` for monetary fields, `Uuid` for IDs
- `serde` serialisation/deserialisation for all entities (JSON, with `Decimal` as string)
- Persistence layer: `save_project(path)` and `load_project(path)` Tauri commands
- `.ercbudget` file format: write and read a full `Project` struct round-trip without data loss
- Auto-save stub: Tauri command `auto_save()` writes to a temp file (content can be empty for now)
- EU rate data JSON files created for all three rate versions and embedded via `include_str!`
- `RateData` struct loaded at application startup from embedded JSON
- `create_project()` IPC command: accepts `ProjectConfig` DTO, returns an empty `ProjectDto`
- TypeScript DTO types generated (can be hand-written initially; codegen tool optional)

**Dependencies:** Sprint 1 complete (M-01 partially satisfied).

**Definition of Done:** A `Project` with all entity types can be saved to a `.ercbudget` file and loaded back with byte-identical JSON. EU rate data loads successfully at startup with correct decimal values.

**M-01 exit criteria met at end of Sprint 2.**

---

### Sprint 3 — Calculation Engine: Personnel (Weeks 5–6)

**Goal:** CALC-01 through CALC-04 implemented, tested, and exposed via IPC.

**Deliverables:**

- `salary_projection.rs`: CALC-01 (`convert_try_to_eur`) + CALC-02 (`project_salary_chain`)
- `personnel_cost.rs`: CALC-03 (`calculate_personnel_cost_lines`) + CALC-04 (`aggregate_personnel_costs`)
- Rust unit tests covering all worked examples from calculation-engine.md CALC-01/02/03/04, plus:
  - Zero inflation rate (flat salary across all years)
  - Single active year only
  - All years active
  - FTE = 1.0 and FTE = 0.1 edge cases
  - Exchange rate producing non-terminating decimal (verified precision is maintained)
- IPC commands: `add_personnel_role`, `update_personnel_role`, `delete_personnel_role` — each returns a partial `BudgetSummaryDto` (only personnel fields populated; other categories return 0)
- `preview_role_cost` IPC command returning `RoleCostPreviewDto` (year-by-year table for the live preview box)
- Rust validation: `validate_personnel_role()` enforcing all PE-01 constraints

**Dependencies:** Sprint 2 complete (domain layer and persistence available).

**Test count target:** ≥ 30 unit tests for CALC-01 through CALC-04.

**Definition of Done:** All worked examples from calculation-engine.md for CALC-01–04 pass as automated tests. IPC round-trip from TypeScript to Rust and back for add/update/delete role is functional.

---

### Sprint 4 — Calculation Engine: Equipment & Travel (Weeks 7–8)

**Goal:** CALC-05 through CALC-12 implemented, tested, and exposed via IPC.

**Deliverables:**

- `equipment_depreciation.rs`: CALC-05 (`calculate_depreciation`) + CALC-06 (`aggregate_equipment_costs`)
- `trip_cost.rs`: CALC-07 (`lookup_flight_cost`), CALC-08 (`calculate_accommodation_cost`), CALC-09 (`calculate_subsistence_cost`), CALC-10 (`calculate_itemized_trip_cost`), CALC-11 (`calculate_flat_trip_cost`), CALC-12 (`aggregate_travel_by_year`)
- Rust unit tests:
  - CALC-05: all four worked examples (capped laptop, audio recorder, 80% usage laptop, partial server); boundary case (usage_months = lifetime); usage_months = 1
  - CALC-07: all band boundaries; exact-boundary cases (600 km = F-01; 601 km = F-02); distance = 0; distance = 399; distance = 10,001
  - CALC-08/09: each sample country; country not in table → `COUNTRY_NOT_IN_RATE_TABLE`
  - CALC-10: India fieldwork and France conference examples; zero domestic transport; zero flight (distance 0)
  - CALC-11: flat amount × multiple instances
  - CALC-12: trips spread across years; all trips in same year; no trips (zero result)
- IPC commands: `add_equipment_item`, `update_equipment_item`, `delete_equipment_item`, `add_trip`, `update_trip`, `delete_trip`
- `preview_equipment_depreciation` and `preview_trip_cost` IPC commands
- Rust validation for equipment (EQ-01 constraints) and trips (TR-01 constraints)

**Dependencies:** Sprint 3 complete.

**Test count target:** ≥ 50 unit tests for CALC-05 through CALC-12.

**Definition of Done:** All worked examples from calculation-engine.md for CALC-05–12 pass as automated tests. Rate table lookup returns correct values for all sample countries and all band boundaries.

---

### Sprint 5 — Calculation Engine: Totals, Indirect, CFS, BudgetSummary (Weeks 9–10)

**Goal:** CALC-13 through CALC-19 implemented. Full CALC-19 orchestration working end-to-end.

**Deliverables:**

- `budget_aggregator.rs`: CALC-13 (`aggregate_c3_costs`), CALC-14 (`calculate_indirect_costs`), CALC-15 (`calculate_total_direct_costs`), CALC-16 (`calculate_total_eligible_costs`), CALC-17 (`calculate_requested_contribution`)
- `cfs_checker.rs`: CALC-18 (`check_cfs_threshold`) with all four `CfsStatus` variants
- `budget_summary.rs`: CALC-19 (`calculate_budget_summary`) — full orchestration in correct dependency order
- `add_other_cost`, `update_other_cost`, `delete_other_cost`, `set_subcontracting` IPC commands — each calls CALC-19 and returns complete `BudgetSummaryDto`
- All prior mutation commands (personnel, equipment, trip) updated to call CALC-19 and return complete `BudgetSummaryDto`
- Integration tests: a complete project with all entity types produces the correct full BudgetSummary (verified against a manually computed reference budget)
- Rust unit tests for CALC-13–18 edge cases: zero C3 items; CFS threshold at exactly €430,000 (not triggered); at €430,001 (triggered); CFS item present; CFS item dismissed

**Dependencies:** Sprint 4 complete.

**Test count target:** ≥ 20 unit tests + 3 integration tests for CALC-13–19.

**Definition of Done:** A project with all cost types populated returns a `BudgetSummaryDto` where every figure matches the manually computed reference. All CfsStatus variants are reachable and correct.

**M-02 exit criteria met at end of Sprint 5.**

---

### Sprint 6 — Frontend: Wizard Screens 0–4 (Weeks 11–12)

**Goal:** First four wizard steps are fully interactive and wired to the Rust backend.

**Deliverables:**

- Application shell: top bar, left 38% / right 62% split layout, ProgressStepper component
- Screen 0: Welcome screen with "New Project" and "Open Project" buttons
- Screen 1: Project Setup (duration stepper, WP stepper, call date, rate version dropdown) — wired to `create_project` IPC command
- Screen 2: Budget Settings (TRY/EUR field, inflation rate, indirect rate with deviation warning + confirmation checkbox) — wired to project update command
- Screen 3: Work Packages (table of optional WP names)
- Screen 4: Personnel (role list + Add/Edit modal with live preview box showing year-by-year EUR projection) — wired to `add_personnel_role`, `update_personnel_role`, `delete_personnel_role`, `preview_role_cost`
- Right panel: BudgetRingChart and CategoryTotalsPanel rendering live from `BudgetSummaryDto` (Recharts)
- Zustand store: `projectStore.ts` holding UI state (current step, open modals, last saved path)
- `useAutoSave` hook wired to trigger `auto_save` IPC on every successful mutation
- Zod schemas for all inputs on Screens 1–4
- EmptyStateCard for Personnel screen (no roles registered)
- All shared components: FormField, RoleCard, LivePreviewBox, WarningBanner

**Dependencies:** Sprint 5 complete (full Rust backend ready). Sprint 6 is the first frontend-heavy sprint.

**Definition of Done:** A user can complete steps 0–4 of the wizard, add multiple personnel roles, and see the right panel update live with correct Category A figures.

---

### Sprint 7 — Frontend: Wizard Screens 5–8 & Full Integration (Weeks 13–14)

**Goal:** All 8 wizard screens complete. The application is feature-complete.

**Deliverables:**

- Screen 5: Equipment (list + Add/Edit form with live depreciation preview + "cap will apply" note)
- Screen 6: Travel (list grouped by year; Itemized form showing EU rates inline + auto-resolved distance band; Flat Amount form)
- Screen 7: Other Costs (list + CFS modal auto-triggered by `CfsStatus.RequiredAndUnaddressed`; persistent CFS warning badge for `RequiredButDismissed`)
- Screen 8: Review & Export (expandable budget table; readiness checklist; format picker: xlsx/PDF/CSV; export buttons)
- BudgetYearBarChart component (Recharts — per-year stacked bar by category)
- Right panel complete: ring chart + category totals + year bar chart + status/warnings strip (CFS badge, indirect rate deviation, empty categories)
- Export engine: `excelExporter.ts` (ExcelJS, 3-sheet workbook), `pdfExporter.ts` (@react-pdf/renderer), `csvExporter.ts`
- Open/Save file dialogs using Tauri's file dialog API
- Step validation: "Next" button on each screen triggers step-level validation before advancing
- Full export validation: "Export" triggers complete project validation before any file is written
- EmptyStateCards for Screens 5, 6, 7

**Dependencies:** Sprint 6 complete.

**Definition of Done:** A user can complete the entire 8-step wizard with realistic ERC-CoG data, export an xlsx file, and the exported figures match the `BudgetSummaryDto` values exactly.

**M-03 exit criteria met at end of Sprint 7.**

---

### Sprint 8 — Testing: Calculation, Validation, and Regression (Weeks 15–16)

**Goal:** Comprehensive test suite covering all calculation paths, all validation rules, and regression against the source Excel workbook.

**Deliverables:**

- **Calculation tests (Rust):** Full suite of unit tests for all 19 CALC functions. Each test is numbered and maps to a specific example or edge case in calculation-engine.md. Minimum coverage: every worked example, every error code, every boundary condition.
- **Validation tests (Rust + TypeScript):** One test per validation rule in business-rules.md (23 rules = minimum 23 tests). Tests confirm that invalid inputs produce the correct error code and do not mutate project state.
- **Integration tests (Rust):** Three complete-project integration tests:
  - Minimal project (PI only, 1 year, no travel, no equipment) → verify BudgetSummary
  - Representative project (all cost categories populated, 5 years) → verify against manually computed reference
  - CFS-triggering project (budget > €430,000) → verify all four CfsStatus transitions
- **Regression tests:** The representative project integration test is additionally verified against the source Excel workbook output (PI confirms the reference numbers from the workbook are used as the expected values).
- **UI tests (Vitest + React Testing Library):** Key user flows:
  - Adding a personnel role updates the right panel
  - CFS modal appears when budget crosses €430,000
  - Indirect rate deviation warning appears and requires checkbox confirmation
  - Export buttons are disabled when readiness checklist has failures
- **Coverage report:** Rust calculation engine must achieve ≥ 95% line coverage. TypeScript must achieve ≥ 80% line coverage.
- All failing tests from earlier sprints (if any) resolved.

**Dependencies:** Sprint 7 complete (M-03).

**Definition of Done:** `cargo test` and `pnpm test` both exit 0. Coverage targets met. Zero P1 bugs (incorrect calculation output) open.

---

### Sprint 9 — Polish, Performance, and Export Quality (Weeks 17–18)

**Goal:** Application feels production-quality. Exports are accurate and professionally formatted. No known P1 or P2 bugs.

**Deliverables:**

- **Performance:** CALC-19 round-trip (from IPC call to UI update) measured and confirmed < 100 ms for the reference project. If > 100 ms, profile and optimise.
- **Excel export quality:**
  - Column widths sized to content
  - Header rows bold, frozen
  - Number format `#,##0 €` for all monetary cells
  - Total rows: double underline border
  - Sheet 1: Overview (ERC submission format — categories A/B/C1/C2/C3/E + totals)
  - Sheet 2: Budget by Year (categories as rows, years as columns)
  - Sheet 3: Detail (itemised per role, per equipment item, per trip, per C3 item)
- **PDF export quality:** One-page formatted summary. Includes project name, PI name, call reference, preparation date, budget table, indirect costs, total eligible, requested contribution. Print-ready at A4.
- **CSV export:** Flat file: Category, Item Name, Year, Amount (EUR). UTF-8 BOM for Excel compatibility.
- **Error messages:** All 24 error codes from calculation-engine.md produce visible, friendly messages in the UI — no raw error codes shown to users.
- **Empty state completeness:** All 5 cost sections show correct EmptyStateCards. Review screen shows correct "no items" messages per category.
- **Accessibility:** Tab order correct on all forms. All form fields have associated labels. All interactive elements are keyboard-accessible.
- **Keyboard shortcuts:** Cmd/Ctrl+S saves; Cmd/Ctrl+O opens; Cmd/Ctrl+N creates new project; Escape closes modals.
- **P1/P2 bug fixes:** All issues found during Sprint 8 testing resolved.
- **Application icon and metadata:** App icon (PNG), bundle identifier (`com.ercbudget.app`), version `1.0.0`, copyright string.

**Dependencies:** Sprint 8 complete.

**Definition of Done:** Full manual walkthrough of the application with realistic ERC-CoG data produces no UX rough edges. Excel export opens correctly in Microsoft Excel and LibreOffice Calc. PDF export prints cleanly on A4. Zero P1 bugs; zero P2 bugs.

**M-04 exit criteria met at end of Sprint 9.**

---

### Sprint 10 — Release Build, Distribution, and Documentation (Weeks 19–20)

**Goal:** Signed, distributable installers for Windows and macOS. Documentation complete. Final PI acceptance test passed.

**Deliverables:**

- **Windows installer:** `.msi` and portable `.exe` built via `cargo tauri build`. Code-signed with a valid Authenticode certificate.
- **macOS installer:** `.dmg` and `.app` bundle built and notarized with Apple Developer ID. Gatekeeper-compliant.
- **Auto-update configuration:** Tauri updater configured to check a GitHub Releases endpoint. Update JSON file published alongside installer.
- **Documentation (4 documents — see TASK-12):**
  - User Manual: step-by-step guide for grant writers; no technical jargon; all 8 screens covered with annotated screenshots
  - Developer Guide: codebase overview, module map, how to add a new cost category, how to update the EU rate tables
  - Architecture Guide: summary of architecture.md decisions in narrative form; suitable for a new team member
  - Deployment Guide: how to build and sign installers on Windows and macOS; how to publish a new release
- **Final acceptance test:** PI runs the application with real ERC-CoG budget data. Output xlsx is verified against the source Excel workbook. Any discrepancies are treated as P1 bugs and resolved before release.
- **GitHub Release v1.0.0:** Tagged release with:
  - Windows installer (`.msi`)
  - macOS installer (`.dmg`)
  - SHA-256 checksums for both files
  - Release notes summarising what the application does

**Dependencies:** Sprint 9 complete (M-04).

**Definition of Done:** PI signs off on the final acceptance test. Installers are downloadable from the GitHub Release page and install successfully on a clean Windows 10 VM and a clean macOS 12 VM.

**M-05 exit criteria met at end of Sprint 10.**

---

## 4. Sprint Dependency Map

```
Sprint 1 (Scaffolding)
    │
Sprint 2 (Domain + Persistence) ──── M-01
    │
Sprint 3 (Personnel Engine)
    │
Sprint 4 (Equipment + Travel Engine)
    │
Sprint 5 (Totals + CFS + BudgetSummary) ──── M-02
    │
Sprint 6 (Frontend: Screens 0–4)
    │
Sprint 7 (Frontend: Screens 5–8 + Exports) ──── M-03
    │
Sprint 8 (Testing Suite)
    │
Sprint 9 (Polish + Performance) ──── M-04
    │
Sprint 10 (Release + Docs) ──── M-05
```

There are no parallel tracks in this plan. The dependency chain is strict: each sprint depends on the previous one being complete. For a team of 3+, Sprints 6 and 7 could be partially parallelised (Screen 0–2 in parallel with Screen 5–6 development) once Sprint 5 is done, but the plan does not assume this.

---

## 5. Risk Register and Mitigation

### R-01 — Rust Learning Curve

**Risk:** Developers unfamiliar with Rust spend excessive time on borrow checker errors and type system issues, delaying the calculation engine.  
**Probability:** High (if team has no prior Rust experience).  
**Impact:** Medium — delays Sprints 3–5 by up to 2 weeks.  
**Mitigation:**
- Assign the Rust backend to the developer with the most systems-programming experience.
- Before Sprint 3 begins, complete the "Rustlings" exercises and read the Rust Book chapters on ownership and error handling.
- Use `thiserror` crate for error types to reduce boilerplate.
- If a blocking Rust issue arises, the Electron fallback (full TypeScript with `decimal.js`) remains viable — switch decision point is end of Sprint 3.

**Fallback (Electron switch trigger):** If the calculation engine is not passing tests by end of Sprint 3, switch to Electron + TypeScript. The domain model, business rules, and calculation specs are framework-agnostic. The TypeScript calculation engine would replace the Rust one; the frontend is identical.

---

### R-02 — EU Rate Table Completeness

**Risk:** The full EU country rate table (Annex 2a/2b, ~200 countries) is not fully transcribed from the official PDF into the embedded JSON before implementation begins.  
**Probability:** Medium.  
**Impact:** Low — missing countries cause a `COUNTRY_NOT_IN_RATE_TABLE` error rather than a wrong value. No financial calculation is silently incorrect.  
**Mitigation:**
- Transcribe the full country table during Sprint 2 (when the rate JSON structure is being built) before it is needed by CALC-08/09 in Sprint 4.
- Use the official Annex 2a/2b PDF directly — copy-paste the tabular data from the PDF into the JSON structure.
- After transcription, write a validation test that checks the entry count matches the official Annex total (confirms no countries were missed).

---

### R-03 — Exchange Rate and Inflation Model Change

**Risk:** The user changes the TRY/EUR rate or a role's inflation rate late in the process, and the application does not recalculate all downstream figures correctly.  
**Probability:** Low — CALC-19 is called after every mutation and returns a fresh BudgetSummary.  
**Impact:** High — if a stale figure is displayed, the submitted budget would be wrong.  
**Mitigation:**
- The architecture enforces that the frontend never stores computed values — it only stores `BudgetSummaryDto` received from the Rust backend.
- Any change to `ProjectConfig` (including rates) triggers a full CALC-19 recalculation of all personnel costs.
- An integration test covers this: change the TRY/EUR rate and verify all role cost lines update correctly.

---

### R-04 — CFS Double-Counting

**Risk:** If a user manually registers a CFS item in OC-01 before the auto-trigger fires (or vice versa), the budget contains two CFS items.  
**Probability:** Low but possible.  
**Impact:** Medium — the budget total would be inflated, affecting indirect costs and the CFS threshold check.  
**Mitigation:**
- CALC-13 validates: `DUPLICATE_CFS_ITEM` error if more than one `is_cfs_item = true` exists.
- CALC-18 checks for an existing CFS item before prompting.
- The "Add Other Cost" form does not expose the `is_cfs_item` flag to users — it is set only by the OC-02 auto-trigger.
- A dedicated validation test covers this scenario.

---

### R-05 — macOS Notarization Failure

**Risk:** The macOS build is rejected by Apple's notarization service, blocking distribution.  
**Probability:** Medium — notarization requirements are strict and depend on correct entitlements.  
**Impact:** High — macOS users cannot install the application from outside the App Store.  
**Mitigation:**
- Tauri v2 has built-in macOS notarization support. Follow the official Tauri notarization guide from the beginning of Sprint 10.
- Test notarization on a test build during Sprint 9 (before Sprint 10 begins) so any issues are caught early.
- If notarization fails, the interim workaround is to instruct users to right-click → Open (bypasses Gatekeeper for Developer ID apps on macOS 13+).

---

### R-06 — Excel Export Formatting in LibreOffice

**Risk:** The xlsx export looks correct in Microsoft Excel but has formatting issues when opened in LibreOffice Calc (column widths, number formats, or cell styles differ).  
**Probability:** Medium — ExcelJS produces standard OpenXML, but LibreOffice compatibility is not guaranteed for all features.  
**Impact:** Low — the data is correct; only formatting may differ.  
**Mitigation:**
- Test the xlsx export in both Excel and LibreOffice Calc during Sprint 9.
- If LibreOffice shows formatting issues, use only the most widely-supported ExcelJS features (avoid worksheet-level styles; prefer cell-level styles).
- Document the "recommended viewer" (Excel) in the User Manual; note LibreOffice is compatible for data but may differ in appearance.

---

### R-07 — Project Budget Size Causes Slow Recalculation

**Risk:** A large project (7 years, 20 roles, 50 trips) causes CALC-19 to take > 100 ms, creating a noticeable delay after each form save.  
**Probability:** Very low — Rust native execution is several orders of magnitude faster than this workload requires.  
**Impact:** Low — the application feels sluggish on large projects.  
**Mitigation:**
- Benchmark CALC-19 for the maximum supported project size during Sprint 9.
- If > 50 ms, profile with `cargo flamegraph` to identify the bottleneck.
- If needed, add a 150 ms debounce on the live preview path (not on the save path — saves always recalculate immediately).

---

## 6. Testing Strategy

### 6.1 Test Categories

| Category | Framework | Location | When Runs |
|---|---|---|---|
| Unit — Rust (calculations) | Rust `#[test]` + `rstest` | `src-tauri/src/calculation/` | Every commit (CI) |
| Unit — Rust (validation) | Rust `#[test]` | `src-tauri/src/validation/` | Every commit (CI) |
| Unit — Rust (persistence) | Rust `#[test]` | `src-tauri/src/persistence/` | Every commit (CI) |
| Integration — Rust | Rust `#[test]` in `tests/` | `src-tauri/tests/` | Every commit (CI) |
| Unit — TypeScript | Vitest | `tests/ui/` | Every commit (CI) |
| UI component tests | Vitest + React Testing Library | `tests/ui/components/` | Every commit (CI) |
| Export tests | Vitest (reads generated files) | `tests/ui/export/` | Every commit (CI) |
| Regression — Rust | Rust `#[test]` | `src-tauri/tests/regression/` | Every commit (CI) |
| Manual acceptance | PI runs application | — | Sprint 10 only |

### 6.2 Calculation Test Requirements

Every calculation function must have tests covering:
1. All worked examples from calculation-engine.md (numbered and traceable)
2. Every error code — one test per code, confirming the correct `CalcError` variant is returned
3. Every boundary condition (e.g., exact band boundaries for CALC-07, exactly at cap for CALC-05)
4. Zero-input cases (empty project, zero amounts where allowed)
5. Maximum-scale input (7 years, maximum supported values for each field)

Test naming convention: `test_calc_NN_description` (e.g., `test_calc_07_boundary_600km_lower_band`).

### 6.3 Regression Test: Reference Budget

A reference project is defined during Sprint 8 with specific inputs chosen to cover all cost categories and all 5 project years. This reference project is computed manually (with PI verification), and the expected `BudgetSummaryDto` is hard-coded as the expected test output. Any future change to the calculation engine that alters this output is a regression and must be investigated before merging.

**Reference project parameters (to be finalised with PI during Sprint 8):**
- 5-year project, 50.62 TRY/EUR, 25% indirect rate
- 1 PI (20% inflation, 0.7 FTE, all 5 years)
- 6 PostDocs (15% inflation, 1.0 FTE, 1 year each: PostDocs 1–6 in years 1–5; PostDoc-6 in Year 2)
- 1 Admin (15% inflation, 0.5 FTE, all 5 years)
- 2 Experts (15% inflation, 0.4 FTE, Year 1 only)
- Equipment: 10 laptops (€2,500 each, 48 months, 100% usage, 55 months used)
- Travel: Fieldwork India Year 1 (4 instances), Conference France Year 2 (3 instances), Other years flat amounts
- C3: MAXQDA Year 1, publications Years 3–5, translation Year 3
- Expected total must match source Excel workbook figures (with corrections for known errors E-01/E-02/E-03)

### 6.4 Coverage Targets

| Scope | Target |
|---|---|
| Rust calculation engine (line coverage) | ≥ 95% |
| Rust validation layer (line coverage) | ≥ 90% |
| TypeScript/React components (line coverage) | ≥ 80% |
| Business rules (each rule has ≥ 1 test) | 100% |
| Error codes (each code has ≥ 1 test) | 100% |

### 6.5 No-Test Policy

The following are explicitly excluded from automated testing (tested manually):
- Visual appearance and layout (font sizes, colours, spacing)
- Recharts rendering (chart library is trusted; we test the data fed to it)
- File dialog interactions (OS-native, cannot be automated in Tauri test environment)
- Auto-updater end-to-end (tested manually during Sprint 10)

---

## 7. Release Strategy

### 7.1 Version Scheme

`MAJOR.MINOR.PATCH` — semantic versioning.

- `1.0.0` — initial public release (end of Sprint 10)
- `1.0.x` — bug fixes (no feature additions)
- `1.x.0` — feature additions (new cost categories, additional rate versions, multi-partner support)
- `2.0.0` — breaking change to `.ercbudget` file format (requires migration tool)

### 7.2 Distribution Channels

| Channel | Format | Audience |
|---|---|---|
| GitHub Releases | `.msi` (Windows), `.dmg` (macOS) | Primary distribution |
| Direct download link in documentation | Same | Institutional IT departments |
| Email to PI | Same | Initial user (Ozyegin University) |

No App Store distribution in v1.0. No auto-update server — users download new versions from GitHub Releases manually in v1.0 (auto-update infrastructure is configured but not activated).

### 7.3 Release Checklist

Before tagging v1.0.0:

- [ ] All Sprint 10 deliverables complete
- [ ] `cargo test` exits 0 (all Rust tests green)
- [ ] `pnpm test` exits 0 (all TypeScript tests green)
- [ ] Coverage targets met
- [ ] Windows installer installs and launches on a clean Windows 10 VM
- [ ] macOS installer installs and launches on a clean macOS 12 VM
- [ ] macOS app passes Gatekeeper (notarization confirmed)
- [ ] Final acceptance test passed by PI
- [ ] SHA-256 checksums generated for both installers
- [ ] Release notes written
- [ ] User Manual reviewed and approved by PI
- [ ] Version number `1.0.0` set in `package.json` and `Cargo.toml`
- [ ] Git tag `v1.0.0` pushed to origin

### 7.4 Post-Release Support

- Bug reports via GitHub Issues.
- P1 bugs (incorrect calculation output): patch release within 5 working days.
- P2 bugs (non-calculation defects): patch release within 2 weeks.
- EU rate table updates (Annex 2a/2b revised): minor release within 4 weeks of official EU publication.

---

## 8. Migration Strategy

### 8.1 Migration from Excel Workbook

There is no automated migration from the existing Excel workbook to the new application. The workbook contains known errors (E-01 through E-08, DUP-01 through DUP-04 as documented in excel-analysis.md) and the data structure is sufficiently different that automated import is not feasible in v1.

**Migration path for existing users:**

1. Open the existing Excel workbook.
2. Open the ERC Budget application and create a new project.
3. Re-enter the project setup parameters (duration, WPs, rates) from the workbook.
4. Re-enter each personnel role using the role's current salary and FTE as the inputs.
5. Re-enter each equipment item.
6. Re-enter each trip — note that the new application requires per-trip year assignment; the workbook's annual average (error E-03) is not carried over.
7. Re-enter each C3 item.
8. Compare the totals in the new application against the workbook. Differences are expected due to the corrected errors (particularly E-03 travel reallocation and E-02 Austria rate).

The User Manual includes a section titled "Migrating from the Excel Workbook" that walks through this process step by step and explains expected numerical differences.

### 8.2 File Format Migration (future versions)

When the `.ercbudget` file format changes in a future version:

- The `format_version` field in every `.ercbudget` file is read before deserialisation.
- If the version is older than the current application supports, a migration function runs automatically before the file is loaded.
- The migration function is tested as part of the persistence layer test suite.
- Old files are never overwritten without confirmation; the application creates a backup copy (`filename.ercbudget.bak`) before migrating.

---

## 9. Success Criteria

The project is considered successfully complete when all of the following are true:

### Functional Success

- [ ] A user can create a new ERC-CoG budget from scratch using the 8-step wizard without any Excel knowledge.
- [ ] The application correctly calculates salaries, depreciation, travel costs, indirect costs, and totals — matching the worked examples in calculation-engine.md.
- [ ] The CFS threshold auto-trigger fires correctly and the user is guided to add the CFS fee.
- [ ] The indirect rate deviation warning prevents accidental submission with a non-standard rate.
- [ ] The exported xlsx file contains three correctly formatted sheets (Overview, By Year, Detail) that open without errors in Microsoft Excel.
- [ ] The exported PDF is a correctly formatted one-page summary suitable for attachment to grant documentation.
- [ ] The application saves and loads `.ercbudget` project files without any data loss.

### Quality Success

- [ ] All 19 calculation specifications from calculation-engine.md are implemented as pure Rust functions with ≥ 95% test coverage.
- [ ] All 24 error codes produce correct, user-friendly messages.
- [ ] The regression test against the reference project passes without modification.
- [ ] No P1 bugs are known at release.

### Distribution Success

- [ ] The macOS installer passes notarization and opens without a Gatekeeper warning.
- [ ] The Windows installer installs without administrator elevation on a standard university-managed Windows 10 PC.
- [ ] The total installer size is under 30 MB on both platforms.

### Acceptance Success

- [ ] PI (Candan Türkkan Ghosh, Ozyegin University) runs the application with real ERC-CoG budget data and confirms the output is correct and the experience is easier than the Excel workbook.
- [ ] The application produces a budget figure for the reference project that matches the corrected expected values (accounting for the three documented workbook errors: E-02 Austria rate, E-03 travel averaging, E-08 overhead base).

---

## Appendix A — Sprint Deliverable Summary

| Sprint | Weeks | Primary Focus | M? |
|---|---|---|---|
| 1 | 1–2 | Environment, scaffolding, CI | — |
| 2 | 3–4 | Domain entities, persistence, rate data | M-01 |
| 3 | 5–6 | CALC-01 to CALC-04 (Personnel) | — |
| 4 | 7–8 | CALC-05 to CALC-12 (Equipment, Travel) | — |
| 5 | 9–10 | CALC-13 to CALC-19 (Totals, CFS, Summary) | M-02 |
| 6 | 11–12 | Frontend Screens 0–4 | — |
| 7 | 13–14 | Frontend Screens 5–8, Export engine | M-03 |
| 8 | 15–16 | Full test suite | — |
| 9 | 17–18 | Polish, performance, export quality | M-04 |
| 10 | 19–20 | Signed installers, documentation, release | M-05 |

---

## Appendix B — Definition of Priority Levels

| Level | Definition | Response Time |
|---|---|---|
| P1 | Incorrect financial calculation output (wrong number shown to user) | Fix before release; patch within 5 days post-release |
| P2 | Application crash, data loss, export failure, blocking UX issue | Fix before release; patch within 2 weeks post-release |
| P3 | Non-blocking UX issue, cosmetic defect, performance concern | Fix in next minor release |
| P4 | Enhancement request, future feature | Backlog |

---

**Confidence Level: 93%**

The sprint plan and milestones are grounded in the completed analysis (TASK-01 through TASK-08). Residual 7%: actual sprint velocity will depend on team composition and Rust experience level. If the team has no prior Rust experience, Sprints 3–5 may each require an additional week. The Electron fallback (R-01 mitigation) is designed to avoid any broader plan disruption.

**Recommended Next Step:**  
Await PI approval. Once approved, proceed to TASK-10 (Implementation).
