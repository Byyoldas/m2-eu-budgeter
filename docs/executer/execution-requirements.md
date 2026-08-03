# Project Execution Application — Functional Requirements

**Phase 04 — Project Execution Requirements**
**Project:** Horizon Europe Project Management Platform
**Date:** 2026-08-03

---

## 1. Purpose and Scope

This document defines the complete functional requirements for the **ERC Execution** desktop application — the second product of the Horizon Europe Project Management Platform. The Execution Application enables research teams to manage, track, and report on a running ERC grant project, using the approved budget (stored in a `.ercbudget` file) as its baseline.

---

## 2. Application Identity

**Product name:** ERC Execution
**Target users:** Project Coordinator, Principal Investigator, Financial Manager, Researcher, Administrative Staff, Work Package Leaders
**Relationship to Budget App:** Reads approved `.ercbudget` file; adds execution data; writes enriched file (format v1.1)

---

## 3. Design Principles

These principles govern every module in this specification:

- The Budget Application is the source of truth for planned values. The Execution App never modifies planned budget figures.
- Every screen shows "Planned vs. Actual" where applicable.
- Users must never need to understand Horizon Europe rules — the application enforces them.
- Warnings and deadlines are surfaced proactively, not discovered through manual checking.
- The application is offline-first. No server is required.
- Data is auto-saved after every interaction.

---

## 4. Module Catalogue

| Module ID | Module Name | Priority |
|---|---|---|
| M-01 | Project Dashboard | MVP |
| M-02 | Budget Open & Project Import | MVP |
| M-03 | Personnel & Person-Month Tracking | MVP |
| M-04 | Work Package Management | MVP |
| M-05 | Deliverable Tracking | MVP |
| M-06 | Milestone Tracking | MVP |
| M-07 | Financial Reporting (Planned vs. Actual) | MVP |
| M-08 | Travel Tracking | MVP |
| M-09 | Equipment Tracking | MVP |
| M-10 | Other Costs Tracking | MVP |
| M-11 | Subcontracting Tracking | MVP |
| M-12 | Risk Register | MVP |
| M-13 | Issue Log | MVP |
| M-14 | Reporting Period Management | MVP |
| M-15 | Meeting Management | V2 |
| M-16 | Action Item Tracker | V2 |
| M-17 | Document Repository | V2 |
| M-18 | Procurement Tracking | V2 |
| M-19 | Excel Import | V2 |
| M-20 | Excel / PDF Export | MVP |
| M-21 | Notifications & Warnings | MVP |
| M-22 | Validation Engine | MVP |
| M-23 | Role-based Access Model (design only) | Architecture |

---

## 5. Module Specifications

---

### M-01 — Project Dashboard

**Purpose:** Single overview screen. The first screen a user sees after opening a project. Surfaces the most critical information without navigation.

**Inputs:** All execution data; approved budget from `.ercbudget`

**Outputs:** Rendered dashboard panels

**Business Rules:**
- `BR-D-01`: Overall budget consumption = Sum of all approved actual costs / Total approved EU contribution × 100%
- `BR-D-02`: Time elapsed % = Current project month / Total project months × 100%
- `BR-D-03`: A warning is displayed if budget consumption % > time elapsed % by more than 10 percentage points
- `BR-D-04`: A warning is displayed if any open deliverable is past its planned month
- `BR-D-05`: A warning is displayed for any reporting period with a submission deadline within 60 days

**Dashboard Panels:**
- Project header (title, PI, call reference, duration, current month)
- Budget consumption gauge (% used vs. % of time elapsed)
- Category spending bars (A, B, C1, C2, C3, E) — planned vs. actual
- WP progress overview (each WP: % budget consumed, milestone/deliverable status)
- Upcoming deadlines (next 90 days: deliverable due dates, reporting period deadlines, milestone planned months)
- Open risks summary (count by severity)
- Open issues summary (count by priority)
- CFS status badge

**Validation:** None (read-only display)

**Future Extensions:** Configurable panel layout; exportable dashboard PDF

---

### M-02 — Budget Open & Project Import

**Purpose:** Load a `.ercbudget` file produced by the Budget Application. This is the mandatory first step.

**Inputs:** `.ercbudget` file path (via file picker dialog)

**Outputs:** Loaded project displayed in the application

**Business Rules:**
- `BR-IO-01`: Only files with `.ercbudget` extension can be opened
- `BR-IO-02`: Format version must be `1.0` or `1.1`; newer MAJOR versions are rejected
- `BR-IO-03`: When opening a `1.0` file for the first time, the application creates an `execution_data` block and upgrades to `1.1`
- `BR-IO-04`: If `execution_data` already exists, it is loaded and merged with the current project state
- `BR-IO-05`: If an `.autosave` sibling exists that is newer than the canonical file, the user is offered the choice to recover from autosave
- `BR-IO-06`: Recent files list is maintained (last 10 files)

**Validation:**
- File must be valid JSON
- `format_version` must be a recognized value
- `project.config.rate_version_id` must exist in embedded rate data
- WP count and array lengths must be consistent

**Future Extensions:** Drag-and-drop file open; file association (double-click `.ercbudget` opens app)

---

### M-03 — Personnel & Person-Month Tracking

**Purpose:** Link actual named individuals to budget roles and track planned vs. actual person-month consumption by reporting period.

**Inputs:**
- Planned roles from `project.personnel_roles` (read-only)
- `Person` records (full name, email, institution, ORCID, linked role ID, actual start/end dates)
- `PersonMonthRecord` entries (per person, per reporting period: planned months, reported months, approved months)

**Outputs:**
- Person-Month tracking table
- Planned vs. actual comparison per role
- Salary cost comparison (planned EUR vs. derived actual EUR)

**Business Rules:**
- `BR-PM-01`: Each `PersonnelRole` may be linked to at most one `Person` at a time
- `BR-PM-02`: Planned months for a role in a period = months of the role's `[start_month, end_month]` range that fall within the period, multiplied by `fte_fraction`
- `BR-PM-03`: Reported months must be ≤ 1.0 per calendar month (full time equivalent cap)
- `BR-PM-04`: Approved months ≤ reported months
- `BR-PM-05`: Sum of approved months across all roles in a period must not exceed the budget's planned person-months for that period by more than 10%
- `BR-PM-06`: A person's actual start date must be ≤ first month of their linked role's `start_month`
- `BR-PM-07`: Salary cost estimate = approved months × (role's current monthly salary in EUR, inflation-adjusted to the period's mid-point year)

**Validation:**
- `full_name`: required
- `linked_role_id`: must reference an existing `PersonnelRole`
- `reported_months`: ≥ 0, ≤ planned months for the period + 10% tolerance
- `actual_start_date`: valid ISO date

**Future Extensions:** Integration with HR system; automatic month calculation from timesheets

---

### M-04 — Work Package Management

**Purpose:** Provide an actionable view of each Work Package with its planned budget, actual spending, and progress status.

**Inputs:**
- WP definitions from `ProjectConfig`
- Actual cost entries filtered by WP
- Milestones and deliverables linked to WP

**Outputs:**
- WP detail view (planned budget, actual spend, % consumed, status)
- WP Gantt timeline (rendered from existing planned months)

**Business Rules:**
- `BR-WP-01`: WP budget (planned) = sum of all planned cost items assigned to that WP (from `BudgetSummaryDto.wp_budgets`)
- `BR-WP-02`: WP actual = sum of all approved `ActualCostEntry` items linked to that WP
- `BR-WP-03`: WP overspend warning triggered when actual > planned × 1.05 (5% tolerance)
- `BR-WP-04`: WP status is derived: NotStarted / OnTrack / AtRisk / Delayed / Completed
  - NotStarted: current month < WP start month
  - Completed: current month > WP end month AND all deliverables Accepted
  - AtRisk: any milestone or deliverable is Delayed or AtRisk
  - Otherwise: OnTrack

**Validation:**
- WP definitions are read-only (sourced from Budget App)
- WP leader assignment: `leader_role_id` must reference an existing `PersonnelRole`

**Future Extensions:** WP-level notes and commentary; WP-to-WP dependencies

---

### M-05 — Deliverable Tracking

**Purpose:** Track all project deliverables from planning through submission, review, and acceptance.

**Inputs:**
- `Deliverable` records (number, title, type, planned month, responsible role, WP, dissemination level)
- Linked `Document` records (submitted files)
- Linked `ReportingPeriod`

**Outputs:**
- Deliverable list with status indicators
- Overdue deliverable warnings
- Deliverable detail view

**Business Rules:**
- `BR-DEL-01`: A deliverable is overdue if `actual_submission_date IS NULL` AND current project month > `planned_month`
- `BR-DEL-02`: Deliverable number format must be `D{wp_id}.{sequence}` (e.g., `D1.1`, `D2.3`)
- `BR-DEL-03`: `Rejected` deliverables must have an associated revision note and a revised planned month
- `BR-DEL-04`: `Public` deliverables must be registered in CORDIS (advisory warning, not blocking)
- `BR-DEL-05`: Deliverables submitted in a reporting period appear in the period's technical report

**Deliverable Types:** Report, Dataset, Software, Prototype, DEM (Demonstrator), Ethics, Other

**Dissemination Levels:** Public (PU), Restricted to Programme (RE), Confidential (CO)

**Status Flow:** `NotStarted` → `InProgress` → `Submitted` → `Accepted` | `Rejected` → `Revised` → `Submitted`

**Validation:**
- `deliverable_number`: required; unique within project; format D{n}.{m}
- `title`: required
- `planned_month`: 1 ≤ x ≤ project duration in months
- `responsible_role_id`: must reference existing PersonnelRole

**Future Extensions:** CORDIS API integration for automatic registration; reviewer assignment and review workflow

---

### M-06 — Milestone Tracking

**Purpose:** Track key project milestones with planned and actual completion dates.

**Inputs:** `Milestone` records (title, WP, planned month, status, linked deliverables)

**Outputs:** Milestone list; Gantt-style milestone markers on WP timeline

**Business Rules:**
- `BR-MS-01`: A milestone with status `NotStarted` and planned month < current month is automatically flagged as `AtRisk`
- `BR-MS-02`: A milestone can only be marked `Completed` if all linked deliverables have status `Accepted`
- `BR-MS-03`: Milestone completion must be recorded by the end of the reporting period in which the planned month falls

**Status Flow:** `NotStarted` → `OnTrack` | `AtRisk` | `Delayed` → `Completed` | `Cancelled`

**Validation:**
- `title`: required
- `planned_month`: 1 ≤ x ≤ project duration
- `work_package_id`: must reference existing WP

**Future Extensions:** Milestone dependencies; critical path highlighting

---

### M-07 — Financial Reporting (Planned vs. Actual)

**Purpose:** The central financial view. Shows planned budget and actual costs side by side for each ERC budget category and each reporting period.

**Inputs:**
- Planned budget from `BudgetSummaryDto` (from `.ercbudget`)
- `ActualCostEntry` records grouped by category and period
- Approved `PersonMonthRecord` entries

**Outputs:**
- Financial summary table: Category A through E, Planned | Actual | Variance | % Used
- Per-period breakdown
- Per-WP breakdown
- CFS compliance status

**Business Rules:**
- `BR-FIN-01`: Category A actual = sum of (approved person-months × inflation-adjusted monthly salary) per role per period
- `BR-FIN-02`: Category B, C1, C2, C3 actuals = sum of approved `ActualCostEntry` items per category
- `BR-FIN-03`: Category E (Indirect) actual = (A + C1 + C2 + C3 actuals) × `indirect_cost_rate_pct`
- `BR-FIN-04`: A budget deviation warning is raised when any category's actual exceeds planned by more than 15%
- `BR-FIN-05`: A significant budget transfer (>10% between categories) must be flagged for PI approval
- `BR-FIN-06`: CFS status is re-evaluated against the running actual EU contribution total
- `BR-FIN-07`: Total requested actual EU contribution = actual total eligible costs (same 100% funding rule as budget)

**Validation:**
- `actual_amount_eur`: > 0
- `category`: must be a valid ERC category code (A, B, C1, C2, C3)
- `reporting_period_id`: must reference existing period

**Future Extensions:** Forecast to completion calculation; automatic audit trail generation

---

### M-08 — Travel Tracking

**Purpose:** Track execution of planned trips and record actual travel costs.

**Inputs:**
- Planned trips from `project.trips` (read-only)
- `TripExecution` records per planned trip instance

**Outputs:**
- Travel tracking table: planned vs. actual cost per trip
- Itemized cost breakdown for EU reimbursement
- Overbudget travel warnings

**Business Rules:**
- `BR-TR-01`: Each planned trip has N instances. Each instance generates one `TripExecution` record.
- `BR-TR-02`: For Itemized trips: actual cost is computed using EU rate tables (same as planned) + submitted receipts for accommodation and flight
- `BR-TR-03`: For FlatAmount trips: actual cost is recorded as submitted by traveller; no rate table validation
- `BR-TR-04`: Actual cost > planned cost × 1.20 triggers a warning requiring justification
- `BR-TR-05`: Unapproved trip instances are excluded from actual cost totals
- `BR-TR-06`: The traveller must be linked to a `Person` record

**Validation:**
- `actual_travel_date`: valid ISO date; must fall within project duration
- `actual_cost_eur`: > 0
- `traveller_role_id`: must reference existing PersonnelRole

**Future Extensions:** Receipt image upload and OCR; per-diem calculator tool; bulk travel import from expense system

---

### M-09 — Equipment Tracking

**Purpose:** Track actual equipment purchases against the approved budget and compute eligible depreciation based on actual purchase data.

**Inputs:**
- Planned equipment items from `project.equipment_items` (read-only)
- `EquipmentProcurement` records (purchase date, actual cost, supplier, invoice)

**Outputs:**
- Equipment tracking table: planned vs. actual eligible depreciation
- Procurement status indicators

**Business Rules:**
- `BR-EQ-01`: Actual eligible depreciation = recalculated using CALC-05 with actual purchase cost (if different from planned)
- `BR-EQ-02`: If actual purchase cost > planned cost × 1.10, a warning is displayed
- `BR-EQ-03`: Equipment purchases must occur within the project duration (purchase date within project start + duration)
- `BR-EQ-04`: An equipment item with `delivery_confirmed = false` is excluded from actuals until confirmed
- `BR-EQ-05`: Items not yet purchased show status "Pending Procurement"

**Validation:**
- `actual_purchase_cost_eur`: > 0
- `purchase_date`: valid ISO date

**Future Extensions:** Asset tagging and audit trail; integration with institutional procurement system

---

### M-10 — Other Costs Tracking

**Purpose:** Track actual expenditure on Other Direct Costs items against approved budget.

**Inputs:**
- Planned other cost items from `project.other_cost_items` (read-only)
- `ActualCostEntry` records with `category = "C3"`

**Outputs:**
- C3 tracking table: planned vs. actual per item
- CFS cost tracking (automatic from CFS-linked items)

**Business Rules:**
- `BR-OC-01`: Each planned OtherDirectCostItem may have one or more actual cost entries linked to it
- `BR-OC-02`: If a planned item's actual total > planned amount × 1.10, a warning is shown
- `BR-OC-03`: Unbudgeted C3 items (actual costs with no corresponding planned item) require justification text
- `BR-OC-04`: CFS item actual cost must be entered before the final financial report is submitted

**Validation:** Same as budget validation, plus: `linked_entity_id` references a valid `OtherDirectCostItem` if provided

**Future Extensions:** Recurring cost auto-population (e.g., annual software renewals)

---

### M-11 — Subcontracting Tracking

**Purpose:** Track subcontracting contracts and payment milestones against the approved budget.

**Inputs:**
- Planned subcontracting from `project.subcontracting` (read-only)
- `SubcontractingLine` records (vendor, amount, contract reference, WP, payment milestones, status)

**Outputs:**
- Subcontracting summary: planned vs. actual total
- Per-contract detail view

**Business Rules:**
- `BR-SC-01`: Total of all `SubcontractingLine` amounts must not exceed `project.subcontracting.amount_eur`
- `BR-SC-02`: Each subcontract must reference an institutional procurement process (advisory)
- `BR-SC-03`: Subcontracting > €200,000 must use competitive tendering (advisory warning)
- `BR-SC-04`: Subcontracting to the host institution's own departments is prohibited (advisory warning)

**Validation:**
- `vendor`: required
- `amount_eur`: > 0
- `contract_reference`: required

**Future Extensions:** Contract PDF attachment; payment schedule tracking with actual payment dates

---

### M-12 — Risk Register

**Purpose:** Maintain a live project risk register with probability/impact scoring and mitigation tracking.

**Inputs:** `RiskEntry` records (title, description, WP, probability, impact, mitigation, status, owner, dates)

**Outputs:**
- Risk matrix (3×3 probability × impact grid with risk count in each cell)
- Risk list sorted by risk score
- Overdue review date warnings

**Business Rules:**
- `BR-RK-01`: Risk score = probability_value × impact_value where Low=1, Medium=2, High=3 (range 1–9)
- `BR-RK-02`: Score ≥ 6 = High priority (red); 3–5 = Medium (amber); 1–2 = Low (green)
- `BR-RK-03`: High-priority risks require a review date ≤ 30 days from current date
- `BR-RK-04`: Risks marked `Closed` cannot be re-opened (a new entry must be created)
- `BR-RK-05`: ERC review meeting preparation automatically exports open High risks

**Validation:**
- `title`: required
- `probability`: Low | Medium | High
- `impact`: Low | Medium | High
- `owner_role_id`: optional; if set, must reference existing PersonnelRole

**Future Extensions:** Risk trend chart over time; automatic risk escalation notification

---

### M-13 — Issue Log

**Purpose:** Track and resolve project issues as they arise during execution.

**Inputs:** `IssueEntry` records (description, raised date, priority, owner, status, resolution)

**Outputs:**
- Issue list sorted by priority and date
- Open issue count on dashboard

**Business Rules:**
- `BR-IS-01`: An issue without a resolution cannot be marked `Closed`
- `BR-IS-02`: High-priority issues unresolved for > 14 days trigger a dashboard warning
- `BR-IS-03`: Issues may optionally be linked to a `RiskEntry` (issue manifested from a risk)

**Validation:**
- `description`: required
- `raised_date`: valid ISO date; ≤ today
- `priority`: Low | Medium | High

**Future Extensions:** Issue-to-action-item linking; ERC reporting integration

---

### M-14 — Reporting Period Management

**Purpose:** Define and manage the project's reporting periods (interim and final reports).

**Inputs:** `ReportingPeriod` records (number, start/end months, deadline, status, report submission flags)

**Outputs:**
- Reporting period list with status and countdown to deadline
- Period financial summary for Financial Report preparation
- Deliverables due in the period for Technical Report preparation

**Business Rules:**
- `BR-RP-01`: Reporting periods must collectively cover the full project duration without gaps
- `BR-RP-02`: The final period must end at `duration_years × 12`
- `BR-RP-03`: A reporting period cannot be marked `Submitted` unless both technical and financial report flags are set
- `BR-RP-04`: Once submitted, a period is locked — its actual cost entries cannot be modified (advisory; no hard lock in MVP)
- `BR-RP-05`: Default reporting periods for ERC CoG: P1 (M1–M18), P2 (M19–M36), P3 (M37–M60) — pre-populated on project open

**Validation:**
- `start_month`, `end_month`: valid within project duration
- `submission_deadline`: valid ISO date
- Periods must be non-overlapping and contiguous

**Future Extensions:** Automated reminder emails; EC reporting portal integration

---

### M-15 — Meeting Management (V2)

**Purpose:** Record all project meetings with agenda, attendees, and minutes references.

**Inputs:** `Meeting` records (title, type, date, attendees, agenda, minutes document link)

**Business Rules:**
- `BR-MT-01`: ERC Progress Review Meeting must be recorded with meeting type `ReviewMeeting`
- `BR-MT-02`: Review meetings auto-populate the risk register and open deliverable list as agenda items

**Future Extensions:** Meeting template library; external participant (non-project) attendance tracking

---

### M-16 — Action Item Tracker (V2)

**Purpose:** Track action items generated from meetings.

**Inputs:** `ActionItem` records (description, owner, due date, status, linked meeting)

**Business Rules:**
- `BR-AI-01`: Overdue open action items appear in the dashboard warnings panel
- `BR-AI-02`: Action items can only be assigned to roles with a linked `Person`

---

### M-17 — Document Repository (V2)

**Purpose:** Maintain a structured repository of project documents.

**Inputs:** `Document` records (type, title, upload date, file path / external URL, linked entity)

**Document Types:** TechnicalReport, FinancialReport, Deliverable, Invoice, Contract, EthicsDocument, MeetingMinutes, Other

**Business Rules:**
- `BR-DOC-01`: Documents are stored as file system paths relative to the `.ercbudget` file location, not embedded in the file
- `BR-DOC-02`: External URL documents (e.g., published papers) are tracked by URL, not path
- `BR-DOC-03`: Invoice documents should be linked to `ActualCostEntry` records

---

### M-18 — Procurement Tracking (V2)

**Purpose:** Track procurement processes for equipment and subcontracting items.

**Inputs:** Equipment items and subcontracting lines requiring procurement documentation

**Business Rules:**
- `BR-PR-01`: Items > €5,000 require a procurement reference
- `BR-PR-02`: Items > €200,000 require evidence of competitive tendering

---

### M-19 — Excel Import (V2)

**Purpose:** Allow import of execution data (person-months, actual costs) from standard Excel templates.

**Templates:**
- Person-Month Declaration template (per period)
- Actual Cost Declaration template (per category)

**Business Rules:**
- `BR-XI-01`: Import validates all imported values against business rules before committing
- `BR-XI-02`: Import is non-destructive — duplicate entries are flagged, not silently overwritten

---

### M-20 — Excel / PDF Export

**Purpose:** Generate reports and deliverable documents from execution data.

**Export Types:**
- **Financial Report (Excel)**: Planned vs. actual per category, per WP, per period — formatted for EC submission review
- **Technical Report Annex (Excel)**: Deliverable and milestone status table
- **Project Status Report (PDF)**: One-page dashboard export for PI/coordinator
- **Risk Register (Excel)**: Full risk register export for review meetings
- **Person-Month Declaration (Excel)**: Pre-filled template per reporting period per role

**Business Rules:**
- `BR-EX-01`: All export values must match the on-screen values at the time of export
- `BR-EX-02`: Exports include a timestamp, project title, and "Generated by ERC Execution" footer

---

### M-21 — Notifications & Warnings

**Purpose:** Proactively alert users to deadlines, overruns, and compliance issues.

**Warning Types:**

| Code | Trigger | Severity |
|---|---|---|
| W-01 | Deliverable overdue | Error |
| W-02 | Milestone planned month passed with no completion | Warning |
| W-03 | Reporting period deadline within 60 days | Warning |
| W-04 | Reporting period deadline within 14 days | Error |
| W-05 | Budget category overrun > 15% | Warning |
| W-06 | Total EU contribution exceeds €430,000 and CFS not addressed | Error |
| W-07 | WP budget overrun > 5% | Warning |
| W-08 | High-priority risk review overdue | Warning |
| W-09 | High-priority issue unresolved > 14 days | Warning |
| W-10 | PersonnelRole has no linked Person | Info |
| W-11 | Travel instance actual cost > 120% of planned | Warning |
| W-12 | Equipment purchase date > project end date | Error |

All warnings are displayed in a persistent notification tray and on the main dashboard.

---

### M-22 — Validation Engine

**Purpose:** Enforce cross-entity business rules at input time.

**Shared validators (from `erc-core`):** project config, personnel role, equipment item, trip, other cost, project config

**New execution validators:**
- `validate_person`: full name required; linked role must exist; start date ≤ role start month
- `validate_person_month_record`: reported months ≥ 0, ≤ 1.0 per calendar month; period must exist
- `validate_actual_cost_entry`: amount > 0; category valid; period must exist
- `validate_deliverable`: number format D{n}.{m}; planned month in range; responsible role must exist
- `validate_milestone`: planned month in range; WP must exist
- `validate_reporting_period`: non-overlapping; no gaps; final period covers last month
- `validate_risk_entry`: probability and impact are valid enum values; title required
- `validate_subcontracting_line`: amount ≤ remaining subcontracting budget; vendor required

---

### M-23 — Role-Based Access Model (Design Only — Not Implemented in MVP)

**Purpose:** Design the access control model for future multi-user scenarios.

**Roles:**
- **Project Coordinator**: Full access to all modules
- **Principal Investigator**: Read all; write personnel, deliverables, milestones, risks
- **Financial Manager**: Read all; write all financial data (actual costs, person-months)
- **WP Leader**: Read all; write own WP deliverables, milestones, tasks
- **Researcher**: Read own data; submit person-month declarations
- **Administrative Staff**: Read financial data; manage documents and meetings
- **Auditor (read-only)**: Read all; write none

This model is designed but not enforced in MVP. The application will be single-user in the first release. Architecture must allow role enforcement to be added without structural changes.

---

## 6. Non-Functional Requirements

| ID | Requirement | Target |
|---|---|---|
| NFR-01 | Application startup time | < 3 seconds on standard hardware |
| NFR-02 | File load time | < 1 second for files up to 1 MB |
| NFR-03 | Auto-save response time | < 500ms after user action |
| NFR-04 | UI response time | < 100ms for any user interaction (no IPC blocking UI thread) |
| NFR-05 | Platform support | macOS 12+, Windows 10+ |
| NFR-06 | Offline operation | 100% — no network required |
| NFR-07 | Accessibility | WCAG 2.1 AA for keyboard navigation and screen reader support |
| NFR-08 | Data integrity | No data loss if application crashes during auto-save |

---

## 7. Out of Scope for MVP

The following are explicitly excluded from the initial release:
- Multi-user collaboration (shared file access, conflict resolution)
- Cloud storage or synchronization
- Email notifications
- CORDIS API integration
- EC reporting portal submission
- Mobile or tablet version
- Multi-language support (English only)
- Partner management (single institution)

---

## 8. Open Questions

1. Should the application auto-populate default reporting periods (P1/P2/P3) on first open, or require the user to define them?
2. Should actual salary costs be computed by the application (from person-months × salary) or entered directly by the Financial Manager?
3. Is the 15% category overspend threshold defined by the EC or should it be configurable?
4. Should risk scores (1–9) be used for internal tracking only, or should they appear in ERC review meeting exports?

---

## 9. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| MVP scope is too large for a single development phase | High | Implement M-01 through M-14 and M-20–22 in MVP; V2 adds M-15–19 |
| Financial calculation discrepancies between planned and actual engines | High | Both engines use `erc-core`; share exact same calculation functions |
| User confusion about planned vs. actual values | Medium | Clear visual language throughout; always label source (Planned / Actual / Variance) |

---

## 10. Confidence Level

**85%** — Core module requirements are solid. Financial reporting calculation rules (M-07) and reporting period rules (M-14) may need refinement against actual EC guidelines during implementation. Subcontracting thresholds are approximate and should be verified against current HE rules.

---

## 11. Recommended Next Step

**Proceed to Phase 05 — User Experience Design.**

With requirements defined for all 23 modules, the UX phase will design the navigation, screen layouts, and interaction patterns that make these modules usable without domain expertise.
