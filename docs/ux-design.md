# UX Design

**Document:** TASK-06 Deliverable  
**Date:** 2026-07-10  
**Status:** Draft — Awaiting Approval Before Proceeding to TASK-07  
**Source documents:** business-rules.md, domain-model.md, input-catalog.md

---

> ## ⚠ Current Implementation Notes (as of v1.6.0, 2026-07-17)
>
> The overall shell, step order (Work Packages already correctly appears before Personnel in this document), and Add/Edit-form interaction pattern described here are still accurate. The following details have changed:
>
> - **§4b (Personnel form)**: "Active project years" (checkboxes) is now a **Start Month / End Month** field pair instead. The "Work Package (optional, multi-select)" field no longer exists on this form — a role's WP breakdown is computed automatically and shown read-only (per-WP cost) rather than picked by the user.
> - **§5b/§6b/§7c (Equipment/Travel/Other Costs forms)**: any "Project Year" field no longer exists. Work Package is **required** (single-select for Equipment, multi-select for Travel/Other Costs) rather than optional.
> - **§8a (Budget Summary Table)**: the columns are now **one per Work Package**, not one per project year — "Year 1 … Year 5" should read "WP1 … WPn". Equipment is no longer a special-cased "project total, not split" row; every category (including Equipment) now has a real per-WP breakdown, since WP is now load-bearing rather than informational.
> - **New, not described anywhere in this document**: a "Check for Updates" control on the Welcome screen (Screen 0), and an in-app "Update Available" modal that can appear on any screen (background-checked on launch).
> - A subtlety worth knowing if you're extending an Add/Edit form: the backend's `*DetailDto` for that entity must carry every field the Add form collects, or the Edit form will silently show it blank — this exact bug shipped for Equipment, Travel, and Personnel and was fixed in v1.6.0. See `docs/developer-guide.md` §5.

---

## Design Principles

These principles govern every decision in this document and must be respected throughout implementation.

**1. The user answers questions about their project — not about spreadsheets.**  
Every label, prompt, and tooltip is written in plain research-grant language. No cell references, no formula logic, no technical budget terminology that an ERC applicant would not naturally use.

**2. All calculation complexity is invisible.**  
The user never sees a formula, a multiplication, or an intermediate result they did not ask for. They see only: what they entered, and what the total is. Year-by-year salary projections, depreciation computations, and overhead uplift happen silently.

**3. The budget updates live.**  
Every keystroke on the left panel immediately recalculates and re-renders the right panel. There is no "calculate" button. The user always sees the current total.

**4. Progress is always visible.**  
The user knows exactly how complete their budget is, which sections still need attention, and whether anything is blocking export.

**5. The application guides, not interrogates.**  
Validation errors are constructive and specific. Warnings explain consequences. The app never blocks the user without explaining why and how to resolve it.

**6. Empty is a valid state.**  
A section with no items entered is shown gracefully with a helpful call to action — never as an error, blank space, or zero row that confuses the user.

---

## Application Shell

The application runs as a full-screen desktop window (minimum 1,280 × 800 px). The window is divided into three permanent zones:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  TOP BAR  [App name]  [Project name]  [Save indicator]  [Export button] │
├──────────────────────────────┬──────────────────────────────────────────┤
│                              │                                          │
│   LEFT PANEL  (38% width)    │   RIGHT PANEL  (62% width)              │
│                              │                                          │
│   Step navigator (top)       │   Live Budget Dashboard                 │
│   ─────────────────────      │                                          │
│   Active form / list         │   Always visible                        │
│   (scrollable)               │   Updates on every input change         │
│                              │                                          │
│                              │                                          │
│   [Back]  [Next / Save]      │                                          │
└──────────────────────────────┴──────────────────────────────────────────┘
```

### Top Bar

| Element | Description |
|---|---|
| App name | "ERC Budget" — fixed left |
| Project name | Editable inline label; shows "Untitled Project" until set |
| Save indicator | "Saved" / "Saving…" / "Unsaved changes" — auto-saves on every valid change |
| CFS warning badge | Red badge "⚠ CFS Required" appears when budget > €430,000 and no CFS item exists |
| Export button | Disabled until all required fields are complete; becomes active on the Review screen |

### Left Panel — Step Navigator

The top of the left panel shows a vertical step list. Each step has a status badge:

| Badge | Meaning |
|---|---|
| ○ Not started | No data entered yet |
| ◑ In progress | Some items entered, section not fully reviewed |
| ● Complete | All required fields present and valid |
| ⚠ Warning | Valid but has a non-blocking issue (e.g., CFS missing) |

Steps (in order):

```
  ● 1  Project Setup
  ● 2  Budget Settings
  ○ 3  Work Packages
  ◑ 4  Personnel
  ○ 5  Equipment
  ○ 6  Travel
  ○ 7  Other Costs
  ○ 8  Review & Export
```

Clicking any step navigates directly to it. Steps 4–7 are accessible at any time once Step 1 and Step 2 are complete. Step 8 (Review & Export) is accessible only when Steps 1–2 are complete (not all cost sections need data — a zero budget is valid).

### Right Panel — Live Budget Dashboard

Always visible. Refreshes in real time as the user types or saves. Divided into four sub-sections stacked vertically:

**A. Budget Ring Chart (top)**  
A donut chart showing the budget split by category as a proportion of the total. Segments: Personnel (A), Travel (C1), Equipment (C2), Other Direct Costs (C3), Indirect Costs (E). Subcontracting (B) shown only if > 0. Hovering a segment shows the category name and EUR amount.

**B. Category Totals Panel (middle)**  
A vertical list of each cost category with its current EUR total:

```
  A  Personnel                    € ___,___
  B  Subcontracting               €       0
  C1 Travel & Subsistence         € ___,___
  C2 Equipment                    € ___,___
  C3 Other Direct Costs           € ___,___
  ─────────────────────────────────────────
     Total Direct Costs           € ___,___
  E  Indirect Costs (25%)         € ___,___
  ═════════════════════════════════════════
     Total Eligible Costs         € ___,___
     Requested EU Contribution    € ___,___
```

Each category line is clickable — clicking navigates to that section's wizard step.

**C. Year-by-Year Bar Chart (lower middle)**  
A grouped or stacked bar chart showing the budget by project year. Each bar represents one year, stacked by cost category. Hovering a bar segment shows the year, category, and amount.

**D. Status & Warnings Strip (bottom)**  
A compact strip showing:
- Number of registered items per section (e.g., "4 roles · 8 items · 6 trips · 5 costs")
- Any active warnings as amber/red chips (e.g., "⚠ CFS Required — budget exceeds €430,000")
- Completion percentage: "Budget 74% complete"

---

## User Journey

The typical flow for a new budget:

```
Launch app
    │
    ▼
Welcome Screen
    │  "Start New Budget"
    ▼
Step 1: Project Setup  ──► fills in duration, WP count, call date
    │
    ▼
Step 2: Budget Settings  ──► TRY/EUR rate, inflation, indirect rate
    │
    ▼
Step 3: Work Packages  ──► optional WP names (can skip)
    │
    ▼
Step 4: Personnel  ──► add roles one by one
    │                   [Add Role] → Role Form → save → back to list
    ▼
Step 5: Equipment  ──► add items one by one
    │
    ▼
Step 6: Travel  ──► add trips one by one
    │                [trip type toggle: Itemized / Flat Amount]
    ▼
Step 7: Other Costs  ──► add C3 items
    │                    [CFS auto-prompt if budget > €430k]
    ▼
Step 8: Review & Export  ──► complete budget summary → Export
```

The user may navigate non-linearly. The right panel always reflects the current state regardless of where in the wizard the user is.

---

## Screen Specifications

---

### Screen 0 — Welcome

**Shown:** On first launch and when no project is open.

**Purpose:** Orient the user, explain what the app does, initiate a new project.

**Layout:** Centred card on a neutral background. No left/right split yet.

**Content:**

```
┌────────────────────────────────────────────┐
│                                            │
│        ERC Budget                          │
│                                            │
│  Build your ERC Consolidator Grant         │
│  budget step by step.                      │
│                                            │
│  No spreadsheets. No formulas.             │
│  Just your project details.                │
│                                            │
│  ┌──────────────────────────────────────┐  │
│  │      Start New Budget                │  │
│  └──────────────────────────────────────┘  │
│                                            │
│  Or open an existing project file ↗        │
│                                            │
└────────────────────────────────────────────┘
```

**Interactions:**
- "Start New Budget" → navigates to Step 1 (Project Setup) and opens the split-panel shell
- "Open existing project file" → file picker for a saved `.ercbudget` project file

**Empty state:** This screen is itself the empty state for the application.

---

### Screen 1 — Project Setup

**Step:** 1 of 8  
**Left panel title:** "Project Setup"  
**Subtitle:** "Tell us the basic shape of your project. This determines how the rest of the budget is structured."

**Form fields (in order):**

```
How many years does your project run?
[ 5  ▲▼ ]   (integer stepper, default 5, range 1–7)

How many Work Packages does your project have?
[ 5  ▲▼ ]   (integer stepper, default 5, range 1–10)

What is the ERC call opening date?  (optional)
[ __ / __ / ____  📅 ]
↳ helper text: "Used to select the correct EU travel cost rates.
  Leave blank to select manually below."

EU Travel Rate Version
[ from 13 May 2025  ▼ ]
↳ auto-selected from the call date above; editable
↳ helper text: "This determines the official accommodation,
  subsistence, and flight rates used for travel calculations."
```

**Right panel behaviour:** Dashboard shows all zeros; the ring chart is empty with a "Start entering data to see your budget" placeholder.

**Navigation:**
- "Next: Budget Settings →" (primary button)
- No "Back" button on Step 1

**Validation:**
- Project duration must be set before Next is enabled
- Number of WPs must be set before Next is enabled
- EU Rate Version must be selected (auto-selected if call date is given)

**Empty state:** N/A — this screen is the starting point.

---

### Screen 2 — Budget Settings

**Step:** 2 of 8  
**Left panel title:** "Budget Settings"  
**Subtitle:** "Set the financial parameters that apply across your entire budget."

**Form fields:**

```
Current TRY / EUR exchange rate
[ _________ ]   e.g. 50.62
↳ helper text: "Enter today's Turkish Lira to Euro conversion rate.
  This converts all salaries from TRY to EUR."

Default annual salary inflation rate  (%)
[ _____ % ]   e.g. 15
↳ helper text: "The expected year-on-year raise applied to all staff.
  You can set a different rate for each person in the Personnel section."

Indirect cost rate  (%)
[ 25 % ]   (default 25, editable)
↳ helper text: "ERC applies 25% overhead on all direct costs.
  Change only if a different rate has been agreed with your institution."
↳ if changed from 25%: amber warning inline:
  "⚠ Non-standard rate. ERC default is 25%. Please confirm this has been approved."
  [ Confirm non-standard rate ]  checkbox
```

**Right panel behaviour:** Dashboard still shows zeros; category labels and structure now visible.

**Navigation:**
- "← Back" (secondary, returns to Step 1)
- "Next: Work Packages →" (primary)

**Validation:**
- TRY/EUR rate must be > 0
- Inflation rate must be ≥ 0 and ≤ 100
- Indirect cost rate must be ≥ 0 and ≤ 50
- If indirect rate ≠ 25, confirmation checkbox must be checked before Next is enabled

---

### Screen 3 — Work Packages

**Step:** 3 of 8  
**Left panel title:** "Work Packages"  
**Subtitle:** "Optionally name your Work Packages. This is for labelling only — it does not affect your budget totals."

**Layout:** A simple table with N rows (one per WP) auto-generated based on PI-02.

```
WP    Name (optional)
────  ──────────────────────────────────
WP-1  [ ________________________________ ]
WP-2  [ ________________________________ ]
WP-3  [ ________________________________ ]
WP-4  [ ________________________________ ]
WP-5  [ ________________________________ ]

You can leave these blank — WP numbers will be used as labels.
```

**Navigation:**
- "← Back"
- "Next: Personnel →"

**Validation:** None — all fields optional. Character limit 100 per name.

**Right panel behaviour:** No change. Dashboard still shows zeros.

---

### Screen 4 — Personnel

**Step:** 4 of 8  
**Left panel title:** "Personnel"  
**Subtitle:** "Register every staff member whose salary will be charged to the grant."

#### 4a — Personnel List View (default)

The main view shows a list of registered roles with a summary card per role.

**Empty state:**

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│  No staff roles added yet.                          │
│                                                     │
│  Each person who works on this grant — even         │
│  part-time — needs to be registered here so         │
│  their salary cost can be calculated.               │
│                                                     │
│  [ + Add First Role ]                               │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Populated state:** A vertical list of role cards:

```
┌─────────────────────────────────── [Edit] [Delete] ┐
│  PI                                                 │
│  FTE: 70%  ·  Active: Years 1–5  ·  Inflation: 20% │
│  Current salary: 227,900 TRY/mo  ≈  €4,500/mo      │
│  Total grant cost: €________                        │
└─────────────────────────────────────────────────────┘
┌─────────────────────────────────── [Edit] [Delete] ┐
│  Expert-1                                           │
│  FTE: 40%  ·  Active: Year 1 only  ·  Inflation: 15%│
│  Current salary: 164,515 TRY/mo  ≈  €3,250/mo      │
│  Total grant cost: €________                        │
└─────────────────────────────────────────────────────┘

[ + Add Another Role ]
```

The "Total grant cost" shown on each card is the sum of PE-03 outputs across all active years for that role. It updates live as the user edits any input.

#### 4b — Add / Edit Role Form (inline slide-in panel or modal)

Opens when the user clicks "+ Add Role" or "Edit" on a card. Replaces or overlays the list view.

**Form title:** "Add Staff Role" / "Edit Role"

```
Role type
○ PI     ○ Expert     ● PostDoc     ○ Admin
  (radio buttons; only one PI allowed)

Role label
[ PostDoc-1 ]   ← auto-suggested, editable
↳ validation: must be unique

Current monthly gross salary
[ ___________ ] TRY/month
↳ live preview: "≈ €_,___ / month at current exchange rate"
   (uses BS-02 TRY/EUR rate)

FTE — share of working time on this grant
[ 100 % ]   (slider 10%–100%, or type directly)
↳ helper: "100% = full time on the grant.
  70% = 70% of working time dedicated to this grant."

Annual salary inflation rate
[ 15 % ]   ← pre-filled from BS-01, editable
↳ helper: "Expected yearly salary raise for this person."

Active project years
☑ Year 1   ☑ Year 2   ☑ Year 3   ☑ Year 4   ☑ Year 5
↳ at least one must be checked

Work Package  (optional)
[ WP-1  ▼ ]   multi-select allowed

─────────────────────────────────────────────────
Projected salary cost for this role:

  Year 1:  €__,___    Year 2:  €__,___
  Year 3:  €__,___    Year 4:  €__,___    Year 5:  €__,___
  ─────────────────────────────────────────────────
  Total:   €___,___
  (updates live as you type)

[ Cancel ]   [ Save Role ]
```

The "Projected salary cost" preview at the bottom shows the output of PE-02 and PE-03 — year-by-year costs — updating live as the user adjusts salary, FTE, inflation, or active years. Labels are "Year 1", "Year 2", etc. — never "€/month × 12 × FTE" or any formula. The user sees only the result.

**Right panel behaviour:** As the user fills the Add Role form, the Personnel bar in the dashboard updates live to reflect the role being composed (before save, shown in a lighter/dashed style to indicate "in progress").

---

### Screen 5 — Equipment

**Step:** 5 of 8  
**Left panel title:** "Equipment"  
**Subtitle:** "Register equipment purchased for the project. The application calculates how much of each item's cost is eligible."

#### 5a — Equipment List View (default)

**Empty state:**

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│  No equipment registered yet.                       │
│                                                     │
│  Add laptops, audio recorders, or any other         │
│  equipment your project will purchase. Only the     │
│  portion used during the grant period is eligible.  │
│                                                     │
│  [ + Add Equipment Item ]                           │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Populated state:** Item cards:

```
┌─────────────────────────────────── [Edit] [Delete] ┐
│  Laptop – PI                                        │
│  Purchase cost: €2,500  ·  Grant usage: 100%        │
│  Used 55 months of 48-month lifetime                │
│  ✓ Full cost eligible (cap applied)                 │
│  Eligible depreciation: €2,500                      │
└─────────────────────────────────────────────────────┘
┌─────────────────────────────────── [Edit] [Delete] ┐
│  Audio Recorder 1                                   │
│  Purchase cost: €60  ·  Grant usage: 100%           │
│  Used 36 months of 60-month lifetime                │
│  Eligible depreciation: €36                         │
└─────────────────────────────────────────────────────┘
```

#### 5b — Add / Edit Equipment Form

```
Item name
[ ___________________________ ]   e.g. "Laptop – PI"

Purchase cost
[ _________ ] EUR
↳ helper: "Total price including import duties, if applicable."

Useful economic lifetime
[ 48 ] months
↳ helper: "The standard lifespan of this type of equipment.
  Typical values: 48 months for laptops, 60 months for audio devices."

Share of use dedicated to this grant
[ 100 ] %
↳ helper: "If this item is used exclusively for the grant, enter 100%.
  If shared with other work, enter the proportionate share."

Number of months used during the grant
[ 55 ] months
↳ helper: "From purchase to end of grant, how many months is this
  item in active use? Cannot be more than the project duration."

Year of purchase  (optional)
[ Year 1  ▼ ]

Work Package  (optional)
[ WP-1  ▼ ]

─────────────────────────────────────────────────
Eligible depreciation for this item: €_,___

  ↳ Note (shown when grant months ≥ lifetime):
    "This item is used for longer than its economic lifetime,
    so the full grant-attributable cost is eligible."

[ Cancel ]   [ Save Item ]
```

The eligible depreciation preview updates live. The cap note appears automatically when grant usage months ≥ useful lifetime. No formula is shown — only the result and a plain-language explanation.

---

### Screen 6 — Travel

**Step:** 6 of 8  
**Left panel title:** "Travel & Subsistence"  
**Subtitle:** "Register each planned trip. The application looks up official EU travel rates for each destination."

#### 6a — Travel List View (default)

**Empty state:**

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│  No trips registered yet.                           │
│                                                     │
│  Add fieldwork visits, conferences, and             │
│  collaboration trips. Costs are calculated          │
│  using official EU unit rates.                      │
│                                                     │
│  [ + Add Trip ]                                     │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Populated state:** Trip cards grouped by year:

```
Year 1
┌─────────────────────────────────── [Edit] [Delete] ┐
│  Fieldwork – India                                  │
│  4 instances  ·  Itemized  ·  India                 │
│  Flight: €857  ·  Accommodation: €780               │
│  Subsistence: €250  ·  Domestic: €340               │
│  Per trip: €2,227  ·  Total (×4): €8,908            │
└─────────────────────────────────────────────────────┘

Year 2
┌─────────────────────────────────── [Edit] [Delete] ┐
│  Conference – Paris (Flat amount)                   │
│  3 instances  ·  Flat amount                        │
│  Per trip: €2,000  ·  Total (×3): €6,000            │
└─────────────────────────────────────────────────────┘

Year total cards with sub-totals shown between year groups.

[ + Add Another Trip ]
```

#### 6b — Add / Edit Trip Form

```
Trip name / purpose
[ ___________________________ ]   e.g. "Fieldwork – India – Year 1"

Trip type
( ● ) Itemized — use EU official rates
( ○ ) Flat amount — enter a total directly

Project year
[ Year 1  ▼ ]

Number of times this trip occurs in this year
[ 1  ▲▼ ]
```

**If Itemized:**

```
Destination country
[ India  ▼ ]   (search/select from full country list)

↳ EU rates for India (Annex 2a/2b, from 13 May 2025):
   Accommodation:  €195 / night
   Subsistence:    €50 / day

One-way flight distance
[ 5,800 ] km   ← from departure city to destination
↳ auto-shows: "Distance band: 4,501–6,000 km → Flight cost: €857 / trip"
  (if distance < 400 km: "Under 400 km — no EU flight cost applies")

Number of nights
[ 4 ]

Number of days
[ 5 ]

Domestic transport within destination country  (optional)
[ 340 ] EUR per trip
↳ helper: "In-country flights, trains, or taxis after arrival."

Work Package  (optional)
[ WP-2  ▼ ]

─────────────────────────────────────────────────
Cost per trip:
  Flight:              €857
  Accommodation:       €780  (€195 × 4 nights)
  Subsistence:         €250  (€50 × 5 days)
  Domestic transport:  €340
  ───────────────────────────
  Per trip total:      €2,227

Total for this entry (× 4 trips):   €8,908
```

**If Flat Amount:**

```
Cost per trip instance
[ _________ ] EUR
↳ helper: "Total cost per occurrence — flights, accommodation,
  and all other expenses combined."

Work Package  (optional)
[ WP-3  ▼ ]

─────────────────────────────────────────────────
Total for this entry (× ___ trips):   €___,___
```

**Common footer:**

```
[ Cancel ]   [ Save Trip ]
```

The cost preview updates live as the user types. EU rates are shown inline as reference values whenever a country is selected — the user never needs to look them up. The flight distance band is resolved and displayed automatically.

---

### Screen 7 — Other Costs

**Step:** 7 of 8  
**Left panel title:** "Other Direct Costs"  
**Subtitle:** "Register publications, software, services, and any other direct costs not covered by the categories above."

#### 7a — Other Costs List View (default)

**Empty state:**

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│  No other costs registered yet.                     │
│                                                     │
│  Add open-access publication charges, software      │
│  licences, translation services, fieldwork costs,   │
│  audit certificates, and similar items.             │
│                                                     │
│  [ + Add Cost Item ]                               │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Populated state:** Item cards grouped by year:

```
Year 1
  Fieldwork costs              €20,000
  MAXQDA software licence      € 9,870
  Fireflies AI subscription    € 1,140
                               ───────
  Year 1 subtotal              €31,010

Year 3
  Open-access publications     € 5,000
  Translation services         € 3,000
                               ───────
  Year 3 subtotal              € 8,000

[ + Add Another Cost Item ]
```

#### 7b — CFS Auto-Trigger Modal

Triggered automatically when the live budget total first crosses €430,000. Overlays the current screen.

```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│  ⚠  Certificate on Financial Statements Required         │
│                                                          │
│  Your total budget has reached €___,___.                 │
│  ERC rules require a Certificate on Financial            │
│  Statements (CFS) when the requested grant               │
│  exceeds €430,000.                                       │
│                                                          │
│  Please enter the estimated audit fee from your          │
│  institution's external auditor.                         │
│                                                          │
│  CFS audit fee                                           │
│  [ ___________ ] EUR                                     │
│                                                          │
│  Year the audit will take place                          │
│  [ Year 4  ▼ ]  ← defaults to final project year         │
│                                                          │
│  [ Add CFS to Budget ]   [ Remind Me Later ]             │
│                                                          │
│  Note: If you dismiss this, a warning badge will         │
│  remain visible until the CFS is added.                  │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

- "Add CFS to Budget" → creates the OtherDirectCostItem with `isCFSItem = true`; modal closes; the item appears in the Year list
- "Remind Me Later" → modal closes; red "⚠ CFS Required" badge remains in the top bar and dashboard strip; re-opening Screen 7 shows a persistent amber banner

#### 7c — Add / Edit Cost Item Form

```
What is this cost for?
[ ___________________________ ]
↳ examples: "Open-access publication", "MAXQDA licence", "Translation"

Amount
[ _________ ] EUR

In which project year will this cost be incurred?
[ Year 1  ▼ ]

Notes / justification  (optional)
[ __________________________ ]

Work Package  (optional)
[ WP-1  ▼ ]

[ Cancel ]   [ Save Item ]
```

---

### Screen 8 — Review & Export

**Step:** 8 of 8  
**Left panel title:** "Review & Export"  
**Subtitle:** "Your complete budget is shown below. Review, then export for your grant application."

The left panel on this screen expands to show a full budget table (overrides the narrow wizard format; the right panel shrinks to a summary-only strip or collapses).

#### 8a — Budget Summary Table

```
                      Year 1      Year 2      Year 3      Year 4      Year 5      TOTAL
─────────────────────────────────────────────────────────────────────────────────────────
A   Personnel         € __,___    € __,___    € __,___    € __,___    € __,___    € ___,___
B   Subcontracting    €       0   €       0   €       0   €       0   €       0   €       0
C1  Travel            € __,___    € __,___    € __,___    € __,___    € __,___    € ___,___
C2  Equipment         —           —           —           —           —           € ___,___  *
C3  Other Costs       € __,___    € __,___    € __,___    € __,___    € __,___    € ___,___
────────────────────────────────────────────────────────────────────────────────────────
    Direct Costs      € __,___    € __,___    € __,___    € __,___    € __,___    € ___,___
E   Indirect (25%)    € __,___    € __,___    € __,___    € __,___    € __,___    € ___,___
════════════════════════════════════════════════════════════════════════════════════════
    Total Eligible    € __,___    € __,___    € __,___    € __,___    € __,___    € ___,___
    EU Contribution   € __,___    € __,___    € __,___    € __,___    € __,___    € ___,___

* Equipment cost shown as a project total; not split by year.
```

Each category row is expandable (click to expand) to show the individual items within it.

#### 8b — Pre-Export Checklist

Below the table, a checklist summarises the budget's readiness:

```
Export Readiness

✓  Project setup complete
✓  Budget settings confirmed
✓  Personnel: 10 roles registered
✓  Equipment: 8 items registered
✓  Travel: 12 trips registered
✓  Other costs: 6 items registered
⚠  Certificate on Financial Statements: not added (budget exceeds €430,000)

[ Export Budget ]   ← disabled if any ✗ items remain; amber if ⚠ items present
```

#### 8c — Export Options

When "Export Budget" is clicked:

```
┌──────────────────────────────────────────┐
│  Export Budget                           │
│                                          │
│  Choose export format:                   │
│                                          │
│  ● Excel Workbook (.xlsx)               │
│    Full budget with per-year breakdown   │
│                                          │
│  ○ PDF Summary                          │
│    One-page budget overview              │
│                                          │
│  ○ CSV (raw data)                       │
│    For import into other tools           │
│                                          │
│  [ Cancel ]   [ Export ]                 │
└──────────────────────────────────────────┘
```

---

## Shared Components

### FormField

Standard labelled input wrapper used across all forms.

```
[Label text]
[Helper text — one line, grey, 12px]
[ Input element                      ]
[Error message — red, 12px, icon ⚠ ]  ← shown only on validation failure
```

### RoleCard / ItemCard / TripCard

Standard summary card used in list views. Contains:
- Title (role label, item name, trip name)
- Two or three key facts in a secondary row (FTE + years + inflation; or cost + lifetime + usage; etc.)
- Computed result in a highlighted row (total cost, eligible depreciation, trip total)
- Edit and Delete actions in the top-right corner

### LivePreviewBox

Shown at the bottom of Add/Edit forms. A light-coloured bordered box containing the computed result(s) for the item being edited. Updates on every keystroke. Never shows intermediate formula steps — only the final result(s) in plain language.

### EmptyStateCard

Full-width card shown when a list section has no items. Contains:
- A short explanation of what this section is for
- A single call-to-action button ("+ Add …")
- No zero values, no empty table rows

### WarningBanner

Amber full-width banner shown inside a screen when a non-blocking issue needs attention. Used for:
- CFS not added after threshold crossed
- Indirect rate differs from 25%

```
⚠  [Warning message text]   [ Action button ]   [Dismiss ×]
```

### ErrorBanner

Red full-width banner for blocking errors that prevent export. Not dismissable until resolved.

### ProgressStepper

The vertical step list in the left panel. Steps are clickable. Each step shows its status badge (○ ◑ ● ⚠). Active step is highlighted.

### SplitRowTotal

Used in list views to show a running subtotal by year or by category. Non-interactive, always computed.

---

## Validation & Error Handling

### When Validation Runs

| Trigger | Behaviour |
|---|---|
| User types in a field | Show inline helper text; suppress errors until first blur |
| User leaves a field (blur) | Show field-level error immediately if invalid |
| User clicks Save on a form | Validate all fields; scroll to first error if any |
| User clicks Next on a step | Validate all required fields for this step; block navigation if invalid |
| Any input change | Re-run live preview calculations; update right panel |

### Field-Level Error Messages

Errors appear below the relevant field in red. They are specific and actionable.

| Situation | Message |
|---|---|
| Required field left blank | "This field is required." |
| TRY/EUR rate = 0 | "Exchange rate must be greater than zero." |
| Inflation rate > 100% | "Inflation rate cannot exceed 100%." |
| FTE fraction = 0 | "FTE must be greater than 0%. At least some time must be on the grant." |
| No active years selected | "Select at least one project year when this role is active." |
| Duplicate role label | "This label is already used. Try 'Expert-2' or another unique name." |
| Flight distance < 0 | "Distance cannot be negative." |
| Nights = 0 on Itemized trip | "Enter at least 1 night." |
| Flat amount = 0 | "Amount must be greater than zero." |
| Equipment lifetime = 0 | "Useful lifetime must be at least 1 month." |
| Grant usage % = 0 | "Grant usage must be greater than 0%." |

### Warnings (Non-Blocking)

Warnings are shown as amber banners or inline amber text. They do not prevent saving or navigation.

| Situation | Warning |
|---|---|
| Grant usage months > project duration | "This exceeds the project's total duration of X months. Double-check the value." |
| Indirect rate ≠ 25% | "ERC standard rate is 25%. Confirm this has been approved if using a different rate." |
| Budget > €430,000 and no CFS | "ERC requires a Certificate on Financial Statements for budgets over €430,000. Please add the audit fee in Other Costs." |
| Flight distance < 400 km on Itemized trip | "Under 400 km — no EU flight unit cost applies. Flight cost set to €0. Accommodation and subsistence still apply." |

### Form-Level Save Errors

If a form cannot be saved due to multiple errors, a summary is shown above the form:

```
⚠  Please fix the following before saving:
   • Role label is required
   • At least one active year must be selected
```

### Unsaved Changes Guard

If the user navigates away from an open form with unsaved changes:

```
┌────────────────────────────────────┐
│  Unsaved changes                   │
│                                    │
│  You have unsaved changes in this  │
│  form. What would you like to do?  │
│                                    │
│  [ Discard changes ]  [ Keep editing ] │
└────────────────────────────────────┘
```

---

## Empty State Patterns

| Section | Empty State Text | CTA |
|---|---|---|
| Personnel | "No staff roles added yet. Each person who works on this grant needs to be registered here." | + Add First Role |
| Equipment | "No equipment registered yet. Add laptops, audio devices, or any items your project will purchase." | + Add Equipment Item |
| Travel | "No trips registered yet. Add fieldwork visits, conferences, and collaboration travel." | + Add Trip |
| Other Costs | "No other costs registered yet. Add publications, software, services, and similar items." | + Add Cost Item |
| Review (budget all zeros) | "Your budget is empty. Go back to the previous sections to add costs." | ← Go to Personnel |

All empty states use a centred layout with a short description paragraph and a single call-to-action button. No zero-filled tables, no placeholder rows.

---

## Navigation Map

```
Welcome
  └── [Start New Budget] ──► Step 1: Project Setup
                                  │
                             [Next] ──► Step 2: Budget Settings
                                              │
                                         [Next] ──► Step 3: Work Packages
                                                          │
                                                     [Next] ──► Step 4: Personnel
                                                                      │ ┌──────────────┐
                                                                      │ │  Add/Edit    │
                                                                      │ │  Role Form   │
                                                                      │ └──────────────┘
                                                                      │
                                                                 [Next] ──► Step 5: Equipment
                                                                                  │
                                                                             [Next] ──► Step 6: Travel
                                                                                              │
                                                                                         [Next] ──► Step 7: Other Costs
                                                                                                          │   [CFS Modal — conditional]
                                                                                                     [Next] ──► Step 8: Review & Export
                                                                                                                      │
                                                                                                                 [Export] ──► Format picker
```

Non-linear navigation is also available by clicking any step in the navigator once Steps 1 and 2 are complete.

---

## Accessibility & UX Notes

- All form fields must have visible labels (no placeholder-only labels).
- Tab order follows the visual top-to-bottom, left-to-right layout.
- All computed values displayed to the user are rounded to the nearest euro (display only; internal calculations maintain full precision until the final display step).
- The EU rate reference values shown in the Trip form are read-only context, not inputs — they must be visually distinct from editable fields (e.g., shown in a shaded info box, not an input field).
- The live preview box in forms is clearly labelled "Estimated cost based on your inputs" to signal it is a result, not an input.
- Keyboard shortcut: Ctrl/Cmd + S saves the current form at any time.
- The right panel is read-only — clicking on it navigates to the relevant section but does not allow direct editing.

---

## Open Questions

No open questions. All UX decisions are grounded in the approved business rules, domain model, and input catalog. UI copy and visual theming (colours, typography, icons) are deferred to the implementation phase (TASK-10).

---

**Confidence Level: 92%**

High confidence on the overall structure, all screen flows, all form fields (directly derived from input-catalog.md), and all validation patterns. Residual 8%: the exact layout of the Review screen (left-panel-only vs. full-width table expansion) and the precise format of the export output (Excel structure, PDF layout) should be confirmed during TASK-07 Architecture and TASK-12 Documentation phases, once the technology stack is chosen.

**Recommended Next Step:**  
Await PI approval. Once approved, proceed to TASK-07 (Architecture).
