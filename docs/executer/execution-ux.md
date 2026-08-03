# Project Execution Application — User Experience Design

**Phase 05 — User Experience Design**
**Project:** Horizon Europe Project Management Platform
**Date:** 2026-08-03

---

## 1. Design Philosophy

The ERC Execution application is built for **proposal writers turned project managers** — researchers and administrators who are experts in their field but not in grant management software. The design must feel like modern commercial project management software, not a spreadsheet replacement.

Core principles:

- **Clarity over density**: Show what matters now; reveal detail on demand
- **Status at a glance**: Every list item communicates health without opening a detail view
- **Proactive guidance**: The application tells the user what needs attention, not the other way around
- **Planned vs. Actual always visible**: Financial context is persistent, never buried
- **No dead ends**: Every screen has a clear next action
- **Progressive disclosure**: Simple defaults with expert options available but not intrusive

---

## 2. Target User Profiles

| Persona | Primary Goal | Most-used Modules | Pain Points |
|---|---|---|---|
| **Project Coordinator (primary)** | Keep the project on track across all dimensions | Dashboard, All modules | Coordinating across people, chasing deadlines |
| **Principal Investigator** | Maintain scientific direction; approve key decisions | Dashboard, WP, Deliverables, Risks | Too many administrative details |
| **Financial Manager** | Ensure financial compliance; prepare EC reports | Financial Reporting, Travel, Personnel months | Complex ERC category rules |
| **WP Leader** | Track deliverables and milestones for their WP | WP view, Deliverables, Milestones | No visibility into other WPs' impact on budget |
| **Researcher** | Submit their person-month records | Personnel (own record only) | Unfamiliarity with PM terminology |
| **Administrative Staff** | Process documents, schedule meetings, track procurement | Documents, Meetings, Actions, Procurement | Switching between too many tools |

---

## 3. Application Layout

### 3.1 Main Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Top Bar                                                         │
│  [ERC Execution logo]  [Project Title]  [▲ Warnings: 3]  [⚙]  │
├──────────────────┬───────────────────────────────────────────────┤
│  Left Navigation │  Content Area                                 │
│  (collapsible,   │                                               │
│  80px / 220px)   │  (scrollable main content)                   │
│                  │                                               │
│  ─ Overview      │                                               │
│  ─ Planning      │                                               │
│    · WP          │                                               │
│    · Deliverables│                                               │
│    · Milestones  │                                               │
│  ─ Financial     │                                               │
│    · Budget      │                                               │
│    · Personnel   │                                               │
│    · Travel      │                                               │
│    · Equipment   │                                               │
│    · Other Costs │                                               │
│    · Subcontract │                                               │
│  ─ Management    │                                               │
│    · Risks       │                                               │
│    · Issues      │                                               │
│    · Periods     │                                               │
│    · Meetings    │                                               │
│    · Actions     │                                               │
│    · Documents   │                                               │
│  ─ Reports       │                                               │
│                  │                                               │
│  ─ ─ ─ ─ ─ ─ ─  │                                               │
│  [Open File]     │                                               │
│  [Save]          │                                               │
└──────────────────┴───────────────────────────────────────────────┘
```

The left navigation is always visible and reflects the current location. Each section header is collapsible. The navigation shows live badge counts for items needing attention (e.g., "Deliverables ⚠ 2").

### 3.2 Window Dimensions

- Default: 1440 × 900 px
- Minimum: 1100 × 700 px
- Navigation collapsed: 80px wide (icons only)
- Navigation expanded: 220px wide (icons + labels)

---

## 4. Welcome Screen

Displayed when no project is open.

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│          [ERC Execution Logo]                           │
│          ERC Execution                                  │
│          Project Tracking for Horizon Europe Grants     │
│                                                         │
│     ┌─────────────────────┐  ┌─────────────────────┐  │
│     │   Open Budget File   │  │   Recent Projects    │  │
│     │   (.ercbudget)       │  │                      │  │
│     │                      │  │  Neural Plasticity…  │  │
│     │   [Browse…]          │  │  ERC-2025-CoG        │  │
│     │   or drag & drop     │  │  Last opened 2h ago  │  │
│     └─────────────────────┘  │                      │  │
│                               │  Quantum Sensing…    │  │
│                               │  ERC-2024-AdG        │  │
│                               │  Last opened 3d ago  │  │
│                               └─────────────────────┘  │
│                                                         │
│  What is this app?                                      │
│  ERC Execution tracks your running ERC grant project    │
│  against the approved budget. Open the .ercbudget file  │
│  created with M2-EU Budgeter to get started.           │
└─────────────────────────────────────────────────────────┘
```

---

## 5. Project Dashboard (M-01)

The primary landing screen after a project is opened.

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Neural Plasticity in Aging Brains                                       │
│  Prof. Ayşe Kaya  ·  ERC-2025-CoG  ·  Month 14 / 60  ·  Year 2 of 5   │
├───────────────────────┬──────────────────────────────────────────────────┤
│  BUDGET HEALTH        │  UPCOMING (next 90 days)                         │
│                       │                                                   │
│  Planned spend: 23%   │  ⚠  D2.1 — Interim Report     DUE Month 15      │
│  Actual spend:  19%   │  ●  ERC Review Meeting         Mar 31, 2027      │
│  ● On track           │  ○  MS-1 — Dataset Complete    Month 18          │
│                       │  ●  Period 1 Financial Report  Apr 30, 2027      │
│  [Budget Health bar]  │                                                   │
│  [see full view →]    │                                                   │
├───────────────────────┴──────────────────────────────────────────────────┤
│  CATEGORY OVERVIEW                                                        │
│                                                                           │
│  A — Personnel     ████████░░░░░░░  €187,000 / €402,500    47% / 58%    │
│  B — Subcontracting ░░░░░░░░░░░░░░  €0 / €0                —            │
│  C1 — Travel       ████░░░░░░░░░░░  €8,200 / €21,661       38% / 58%    │
│  C2 — Equipment    ████████████░░░  €2,500 / €2,536         99% / 58%   │
│  C3 — Other Direct ██████░░░░░░░░░  €5,900 / €36,870        16% / 58%  │
│  E — Indirect      ████████░░░░░░░  €51,400 / €115,267      45% / 58%   │
│                       [Legend: ■ Actual  ░ Planned]                     │
├────────────────────┬─────────────────────┬───────────────────────────────┤
│  WP STATUS         │  RISKS & ISSUES      │  CFS STATUS                  │
│                    │                      │                               │
│  WP1 ● On Track    │  🔴 High risks: 1    │  ✅ REQUIRED AND PRESENT     │
│  WP2 ● On Track    │  🟡 Med risks: 3     │  CFS item: €12,000           │
│  WP3 ○ Not Started │  🟢 Low risks: 2     │  EU contribution: €462,000   │
│                    │  Open issues: 2      │  Threshold: €430,000         │
│  [View WPs →]      │  [View all →]        │                              │
└────────────────────┴─────────────────────┴───────────────────────────────┘
```

Color coding:
- 🟢 Green: on track / compliant
- 🟡 Amber: warning / approaching limit
- 🔴 Red: overrun / overdue / non-compliant
- ⚪ Gray: not started / not applicable

---

## 6. Work Package Screen (M-04)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Work Packages                                                           │
├──────────────────────────────────────────────────────────────────────────┤
│  [WP Timeline Gantt — visual bar chart M1 to M60]                       │
│  WP1 [====================]                                              │
│  WP2        [==============================]                             │
│  WP3                               [===========]                        │
│                                                                           │
│  Current month: ▲ M14                                                   │
├──────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │ WP1 — Data Collection                    ● On Track                │ │
│  │ Leader: Prof. Kaya  ·  Months 1–60                                 │ │
│  │                                                                     │ │
│  │ Budget:  €189,214 planned  /  €71,200 actual  (38%)               │ │
│  │          [████████░░░░░░░░░░░░]                                    │ │
│  │                                                                     │ │
│  │ Deliverables: 2 / 5 complete   Milestones: 0 / 2 complete         │ │
│  │ [View details →]                                                   │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │ WP2 — Analysis                           ● On Track                │ │
│  │ ...                                                                 │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

Clicking a WP card expands it into a full detail view showing:
- Budget breakdown (A/B/C1/C2/C3 planned vs. actual bars)
- Deliverable list for this WP
- Milestone list for this WP
- Person-months for roles assigned to this WP

---

## 7. Deliverables Screen (M-05)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Deliverables                            [+ Add Deliverable]             │
│  Filter: [All WPs ▼] [All Status ▼]                                     │
├────────┬────────────────────────┬──────┬────────┬───────────┬───────────┤
│  #     │  Title                 │  WP  │  Type  │  Due Month│  Status   │
├────────┼────────────────────────┼──────┼────────┼───────────┼───────────┤
│  D1.1  │  Data Collection       │  WP1 │ Dataset│  M12      │  ✅ Accepted│
│        │  Protocol              │      │        │           │           │
├────────┼────────────────────────┼──────┼────────┼───────────┼───────────┤
│  D2.1  │  Interim Analysis Rpt  │  WP2 │ Report │  M15 ⚠    │  🔴 Overdue│
│        │                        │      │        │  (Now M14)│           │
├────────┼────────────────────────┼──────┼────────┼───────────┼───────────┤
│  D2.2  │  Analysis Software v1  │  WP2 │Software│  M24      │  ○ Not Started│
├────────┼────────────────────────┼──────┼────────┼───────────┼───────────┤
│  D3.1  │  Final Report          │  WP3 │ Report │  M60      │  ○ Not Started│
└────────┴────────────────────────┴──────┴────────┴───────────┴───────────┘
```

**Deliverable Detail Panel (slide-in from right):**
```
┌─────────────────────────────────┐
│  D2.1 — Interim Analysis Report │
│  ──────────────────────────────│
│  Work Package:  WP2 — Analysis  │
│  Type:          Report          │
│  Dissemination: Public          │
│  Planned Month: M15             │
│  Responsible:   Dr. Demir (PostDoc-1)│
│                                 │
│  Status: [Submitted ▼]          │
│  Submission Date: [__________]  │
│                                 │
│  Documents:                     │
│  [+ Attach document]            │
│                                 │
│  Notes:                         │
│  [____________________________] │
│                                 │
│  [Cancel]          [Save]       │
└─────────────────────────────────┘
```

---

## 8. Personnel & Person-Month Tracking Screen (M-03)

### 8.1 People Tab

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Personnel                                                               │
│  [People] [Person-Months]                                                │
├──────────────────────────────────────────────────────────────────────────┤
│  Role           │  Type      │  Person Linked     │  Status               │
├─────────────────┼────────────┼────────────────────┼───────────────────────┤
│  PI             │  Pi        │  Prof. Ayşe Kaya   │  ✅ Active             │
│  PostDoc-1      │  PostDoc   │  Dr. Mehmet Demir  │  ✅ Active             │
│  Expert-1       │  Expert    │  —                 │  ⚠ No person linked   │
│  PhdStudent-1   │  PhDStudent│  Zeynep Yılmaz     │  ✅ Active             │
└─────────────────┴────────────┴────────────────────┴───────────────────────┘
│  [+ Link person to role]                                                  │
└──────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Person-Months Tab

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Person-Month Records  —  Period 1 (M1–M18)   [Period 1 ▼]              │
├────────────────┬──────────┬──────────┬──────────┬──────────┬────────────┤
│  Role          │  Planned │  Reported│  Approved│  Cost Est│  Status    │
├────────────────┼──────────┼──────────┼──────────┼──────────┼────────────┤
│  PI            │  12.6 PM │  12.6    │  12.6    │ €67,200  │ ✅ Approved │
│  PostDoc-1     │  18.0 PM │  17.5    │  17.5    │ €82,250  │ ✅ Approved │
│  Expert-1      │  6.0 PM  │  —       │  —       │  —       │ ○ Not rep. │
│  PhdStudent-1  │  12.0 PM │  12.0    │  12.0    │ €36,000  │ ✅ Approved │
├────────────────┼──────────┼──────────┼──────────┼──────────┼────────────┤
│  TOTAL         │  48.6 PM │  42.1    │  42.1    │€185,450  │            │
│  Budget (Plan) │          │          │          │€402,500  │            │
└────────────────┴──────────┴──────────┴──────────┴──────────┴────────────┘
│  [Export Period 1 PM Declaration (Excel)]                                │
└──────────────────────────────────────────────────────────────────────────┘
```

Clicking a row opens a detail panel to enter `reported_months` and see the cost calculation breakdown.

---

## 9. Financial Reporting Screen (M-07)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Financial Report                                    [Export Excel]      │
│  View: [Full Project ▼]   Period: [All Periods ▼]                       │
├──────────────┬────────────┬────────────┬────────────┬────────────────────┤
│  Category    │  Planned   │  Actual    │  Variance  │  % Used            │
├──────────────┼────────────┼────────────┼────────────┼────────────────────┤
│  A — Personnel│ €402,500  │ €185,450   │ -€217,050  │  46%   ●           │
│  B — Subcontr.│      €0   │      €0   │      €0   │   —                │
│  C1 — Travel  │  €21,661  │   €8,200  │ -€13,461   │  38%   ●           │
│  C2 — Equipment│  €2,536  │   €2,500  │     -€36   │  99%  ⚠            │
│  C3 — Other   │  €36,870  │   €5,900  │ -€30,970   │  16%   ●           │
│  E — Indirect │ €115,267  │  €50,462  │ -€64,805   │  44%   ●           │
├──────────────┼────────────┼────────────┼────────────┼────────────────────┤
│  Total Direct │ €463,567  │ €202,050  │ -€261,517  │  44%               │
│  Total Eligible│ €578,834 │ €252,512  │ -€326,322  │  44%               │
│  EU Contrib.  │ €578,834  │ €252,512  │ -€326,322  │  44%               │
└──────────────┴────────────┴────────────┴────────────┴────────────────────┘
│  C2 Equipment is at 99% — nearly fully consumed.                        │
└──────────────────────────────────────────────────────────────────────────┘
```

Toggle between: Full Project / Per Reporting Period / Per Work Package

---

## 10. Risk Register Screen (M-12)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Risk Register                                   [+ Add Risk]           │
│  [List] [Matrix]                                                         │
├──────────────────────────────────────────────────────────────────────────┤
│  Risk Matrix:                                                            │
│                                                                          │
│  Impact →     Low      Medium     High                                  │
│  High     │    —    │   ██ 2   │  ██ 1   │                              │
│  Medium   │    —    │   ██ 2   │    —    │                              │
│  Low      │    —    │    —    │   ██ 1   │                              │
│                                                                          │
├─────────┬──────────────────────────────────────────────┬────────────────┤
│  Score  │  Title                                       │  Status / Owner│
├─────────┼──────────────────────────────────────────────┼────────────────┤
│  🔴  6  │  Key researcher departure                    │  Open · PI     │
│  🟡  4  │  EU rate table change mid-project            │  Open · Coord. │
│  🟡  4  │  Equipment delivery delays                   │  Mitigated     │
│  🟡  3  │  Data access restrictions (ethics)           │  Open · PI     │
│  🟢  2  │  Conference travel cost increases            │  Closed        │
└─────────┴──────────────────────────────────────────────┴────────────────┘
```

---

## 11. Reporting Periods Screen (M-14)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Reporting Periods                              [+ Add Period]           │
├──────────┬───────────┬─────────────┬────────────────┬────────────────────┤
│  Period  │  Months   │  Deadline   │  Technical Rpt │  Financial Rpt     │
├──────────┼───────────┼─────────────┼────────────────┼────────────────────┤
│  P1      │  M1–M18   │  Apr 2027   │  ✅ Submitted  │  ✅ Submitted      │
│  P2      │  M19–M36  │  Oct 2028   │  ○ Not started │  ○ Not started     │
│  P3      │  M37–M60  │  Apr 2031   │  ○ Not started │  ○ Not started     │
└──────────┴───────────┴─────────────┴────────────────┴────────────────────┘
│                                                                          │
│  Clicking P2 opens:                                                      │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  Period 2 — M19 to M36                                            │  │
│  │  Budget Consumed in P2:  €xx,xxx / €xxx,xxx                       │  │
│  │  Deliverables due in P2: 3 (1 submitted, 2 pending)               │  │
│  │  Milestones due in P2:   1 (0 complete)                           │  │
│  │  [Prepare Technical Report ▶]  [Prepare Financial Report ▶]       │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 12. Notification Tray

A persistent tray accessible from the top bar `[▲ Warnings: N]` button:

```
┌──────────────────────────────────────────────────────┐
│  Warnings & Notifications                       [✕]  │
├──────────────────────────────────────────────────────┤
│  🔴  D2.1 Interim Report is overdue (Due M15)        │
│      → Go to Deliverables                            │
├──────────────────────────────────────────────────────┤
│  🟡  C2 Equipment at 99% budget consumed             │
│      → Go to Financial Report                        │
├──────────────────────────────────────────────────────┤
│  🟡  High-priority risk review overdue               │
│      → Go to Risk Register                           │
├──────────────────────────────────────────────────────┤
│  ⚪  Expert-1 has no person linked                   │
│      → Go to Personnel                               │
└──────────────────────────────────────────────────────┘
```

Each notification is clickable and navigates directly to the relevant screen.

---

## 13. Add / Edit Panels (Common Pattern)

All entity creation and editing uses a **right-side slide-in panel**, not a full-page navigation. This keeps context visible and allows users to see the list while editing.

```
Main Content Area                  Edit Panel (360px)
┌─────────────────────────┬────────────────────────────┐
│                         │  Add New Deliverable       │
│  [List continues]       │  ─────────────────────    │
│                         │  Number  [D2.____]         │
│                         │  Title   [________________]│
│                         │  WP      [WP2 — Analysis▼] │
│                         │  Type    [Report ▼]        │
│                         │  Due     [Month ___]       │
│                         │  Resp.   [PostDoc-1 ▼]     │
│                         │  Diss.   [Public ▼]        │
│                         │                            │
│                         │  [Cancel]    [Add]         │
└─────────────────────────┴────────────────────────────┘
```

---

## 14. Status Indicators (Visual Language)

| Indicator | Color | Meaning |
|---|---|---|
| ✅ Green check | `#22c55e` | Complete / Approved / On Track |
| ● Green dot | `#22c55e` | On Track / Active |
| 🟡 Amber dot | `#f59e0b` | Warning / At Risk / Approaching limit |
| 🔴 Red dot | `#ef4444` | Overdue / Overrun / Non-compliant |
| ⚪ Gray dot | `#94a3b8` | Not Started / Not Applicable |
| ⚠ Warning icon | `#f59e0b` | Advisory warning, no action blocked |
| 🚫 Error icon | `#ef4444` | Blocking error, action required |

Progress bars use dual-fill:
- Dark fill: actual
- Light fill to 100%: planned cap
- Red fill: overrun (exceeds planned)

---

## 15. Navigation Structure

```
ERC Execution
├── Dashboard (home)
├── Overview
│   └── Project Info (read-only view of budget file data)
├── Planning
│   ├── Work Packages
│   ├── Deliverables
│   └── Milestones
├── Financial
│   ├── Budget Report (Planned vs. Actual summary)
│   ├── Personnel & PM
│   ├── Travel
│   ├── Equipment
│   ├── Other Costs
│   └── Subcontracting
├── Management
│   ├── Reporting Periods
│   ├── Risk Register
│   ├── Issue Log
│   ├── Meetings          [V2]
│   ├── Action Items      [V2]
│   └── Documents         [V2]
└── Reports & Export
    ├── Financial Report (Excel)
    ├── Technical Report Annex (Excel)
    ├── Project Status (PDF)
    ├── Risk Register (Excel)
    └── PM Declarations (Excel)
```

---

## 16. Key Interaction Patterns

### 16.1 Inline Editing

Status fields (Milestone status, Deliverable status, Issue priority) can be changed directly in the list view via a dropdown — no panel required.

### 16.2 Planned vs. Actual Visual Language

Every financial screen uses a consistent two-column layout:
- **Left column**: Planned (from `.ercbudget`, read-only, shown in muted style)
- **Right column**: Actual (editable)
- **Difference column**: Variance = Actual − Planned (shown with ▲▼ arrows and color)

### 16.3 Period Filter

All financial and tracking screens have a period selector at the top that filters all data to the selected reporting period, or shows the full-project view.

### 16.4 Contextual Help

A `?` icon on every screen opens a slide-in help panel explaining the ERC rules relevant to that screen. This replaces the need to consult the ERC Grant Agreement guide separately.

### 16.5 Empty States

Every list screen with no data shows an empty state card with:
- An icon
- A clear statement of what goes here
- A primary action button to add the first item
- Brief description of why this section matters

---

## 17. Color Palette and Typography

The Execution Application shares the visual identity of the Budget Application:

| Token | Value | Usage |
|---|---|---|
| `--color-primary` | `#1e3a5f` | Headers, navigation active state |
| `--color-accent` | `#3b82f6` | Links, primary buttons, progress fills |
| `--color-success` | `#22c55e` | On track, approved, complete |
| `--color-warning` | `#f59e0b` | Warnings, at-risk states |
| `--color-error` | `#ef4444` | Overdue, overrun, errors |
| `--color-muted` | `#94a3b8` | Not started, read-only planned values |
| `--color-surface` | `#f8fafc` | Panel backgrounds |
| `--color-border` | `#e2e8f0` | Table borders, dividers |

Font: System font stack (`-apple-system, BlinkMacSystemFont, Segoe UI, sans-serif`). Same as Budget Application.

---

## 18. Responsive and Accessibility Considerations

- All interactive elements are keyboard-navigable (Tab order follows visual layout)
- Status indicators always combine color with an icon or label (never color alone)
- All form fields have associated `<label>` elements
- ARIA roles used for dynamic regions (notification tray, slide-in panels)
- Minimum touch target size: 44×44 px for all interactive controls
- Screen reader: all status changes announced via `aria-live` regions

---

## 19. Open Questions

1. Should the navigation be a fixed left sidebar or a top tab bar? (Left sidebar is recommended for the number of sections involved.)
2. Should the period selector be a global app-level filter or per-screen?
3. Should the Gantt chart in Work Packages be interactive (draggable) or purely visual?
4. Should the application support dark mode in the initial release?

---

## 20. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Too many navigation items overwhelm new users | Medium | Progressive disclosure: V2 sections hidden behind "More" by default |
| Planned vs. actual concept confuses non-financial users | Medium | Tooltips explain each column; contextual help panel |
| Slide-in panels on smaller screens cause layout issues | Low | Minimum window width enforced; panels stack on narrow layouts |

---

## 21. Confidence Level

**87%** — Core screens are well-defined. Detail interactions for some V2 modules (Meetings, Documents, Procurement) need refinement during implementation. The visual language is consistent and proven in the Budget App.

---

## 22. Recommended Next Step

**Proceed to Phase 06 — Technical Architecture.**

With UX defined, the architecture phase will specify the technical implementation: layer structure, IPC command design, state management, shared core integration, and all engine designs.
