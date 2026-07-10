# Input Catalog

**Document:** TASK-05 Deliverable  
**Date:** 2026-07-10  
**Status:** Draft — Awaiting Approval Before Proceeding to TASK-06  
**Source documents:** business-rules.md, domain-model.md

---

## How to Read This Document

This catalog lists every field the user must or may enter that has an effect on any budget calculation. Fields that are purely informational (project title, PI name, notes) are excluded, except where noted in the Administrative Data section.

Each entry specifies:

| Column | Meaning |
|---|---|
| **ID** | Unique input identifier used for cross-referencing |
| **Name** | Display label as it will appear in the UI |
| **Purpose** | Why this input exists and what it drives |
| **Type** | Data type: Integer, Decimal, Percent, Text, Date, Enum, Multi-select, Boolean |
| **Required** | Yes / No / Conditional |
| **Default** | Pre-filled value, if any |
| **Validation** | Rules that must pass before the value is accepted |
| **Dependencies** | Other inputs that must exist before this one |
| **Visibility** | When this field is shown or hidden |

Inputs are organised into eight groups matching the domain model. Inputs marked **[R]** are asked once at project setup; inputs marked **[M]** repeat for each item in a list (per role, per item, per trip, etc.).

---

## Group 1 — Project Information

These inputs define the project's structural parameters. They must all be provided before any cost entry is permitted.

---

**PI-01** [R]  
**Name:** Project Duration  
**Purpose:** Sets the number of full grant years. Determines how many year columns appear across all cost entry screens and how the budget is structured.  
**Type:** Integer  
**Required:** Yes  
**Default:** 5  
**Validation:** ≥ 1 and ≤ 7  
**Dependencies:** None — first input collected  
**Visibility:** Always visible in Project Setup

---

**PI-02** [R]  
**Name:** Number of Work Packages  
**Purpose:** Determines how many Work Package slots are created. Each cost item may be optionally tagged to a WP for planning visibility.  
**Type:** Integer  
**Required:** Yes  
**Default:** 5  
**Validation:** ≥ 1 and ≤ 10  
**Dependencies:** None  
**Visibility:** Always visible in Project Setup

---

**PI-03** [R]  
**Name:** Grant Call Opening Date  
**Purpose:** Used to automatically select the applicable EU travel unit cost rate version (Annex 2a/2b). Different rate tables apply depending on when the call opened. If not provided, the user must select the rate version manually (PI-04).  
**Type:** Date  
**Required:** No (if PI-04 is provided instead)  
**Default:** None  
**Validation:** Must be a valid calendar date; must not be in the future  
**Dependencies:** None  
**Visibility:** Always visible in Project Setup

---

**PI-04** [R]  
**Name:** EU Travel Rate Version  
**Purpose:** The Annex 2a/2b version to use for all travel unit cost lookups (accommodation, subsistence, flight bands). Auto-selected from PI-03 if provided; otherwise the user selects manually.  
**Type:** Enum  
**Required:** Yes  
**Default:** Auto-selected based on PI-03; falls back to "from 13 May 2025" (current version) if PI-03 is blank  
**Validation:** Must be one of the bundled versions: "before 31 July 2024", "31 July 2024 – 12 May 2025", "from 13 May 2025"  
**Dependencies:** PI-03 (drives auto-selection; manual override always available)  
**Visibility:** Shown in Project Setup; collapses to a read-only confirmation badge if PI-03 determines it unambiguously

---

## Group 2 — Budget Settings

Financial parameters that apply across all cost calculations.

---

**BS-01** [R]  
**Name:** Default Annual Salary Inflation Rate  
**Purpose:** The year-on-year percentage by which salaries are expected to grow. Pre-fills the per-role inflation rate field (PE-03) for every new PersonnelRole. Users may override per role. Affects the entire salary projection chain.  
**Type:** Percent  
**Required:** Yes  
**Default:** None (must be entered by user)  
**Validation:** ≥ 0% and ≤ 100%  
**Dependencies:** None  
**Visibility:** Always visible in Project Setup / Budget Settings

---

**BS-02** [R]  
**Name:** TRY / EUR Exchange Rate  
**Purpose:** Converts all TRY-denominated salaries to EUR before the inflation chain is applied. A single rate is used for the entire project duration. Affects every PersonnelRole calculation.  
**Type:** Decimal  
**Required:** Yes  
**Default:** None (must be entered by user)  
**Validation:** > 0; must be a positive number  
**Dependencies:** None  
**Visibility:** Always visible in Project Setup / Budget Settings

---

**BS-03** [R]  
**Name:** Indirect Cost Rate  
**Purpose:** The overhead percentage applied to total direct eligible costs (Personnel + Travel + Equipment + Other Direct Costs). Under ERC Actual Costs rules the standard rate is 25%. Drives the entire Category E (Indirect Costs) calculation.  
**Type:** Percent  
**Required:** Yes  
**Default:** 25  
**Validation:** ≥ 0% and ≤ 50%. If the value differs from 25%, the application displays a deviation warning and requires explicit confirmation.  
**Dependencies:** None  
**Visibility:** Always visible in Budget Settings; deviation from 25% triggers a prominent warning

---

## Group 3 — Work Packages

Inputs for labelling each Work Package. Optional; do not affect calculations.

---

**WP-01** [M — one per Work Package]  
**Name:** Work Package Name  
**Purpose:** Optional descriptive title for a Work Package (e.g., "WP-2: Fieldwork Phase"). Used only for display and reporting labels. Has no effect on any cost calculation.  
**Type:** Text  
**Required:** No  
**Default:** Auto-generated label: "WP-1", "WP-2", … "WP-N"  
**Validation:** Maximum 100 characters; must be unique within the project if provided  
**Dependencies:** PI-02 (WP count must be set before WP names are collected)  
**Visibility:** Shown in Work Package setup screen; WP count determines how many rows appear

> Note: WP names are the only Work Package input. WP numbers are auto-generated and are not user inputs.

---

## Group 4 — Personnel

Inputs collected once per registered staff role. All fields in this group directly drive the salary projection and annual cost calculations.

---

**PE-01** [M — once per role]  
**Name:** Role Type  
**Purpose:** Categorises the role for classification in the final budget table (Personnel sub-categories: PI, Experts, Post-Docs, Administrative Staff). Controls which label prefixes are available and enforces the one-PI-per-project rule.  
**Type:** Enum  
**Required:** Yes  
**Default:** None  
**Validation:** Must be one of: `PI`, `Expert`, `PostDoc`, `Admin`. Only one role with type `PI` may exist per project.  
**Dependencies:** None  
**Visibility:** Always shown in the Add Role form

---

**PE-02** [M — once per role]  
**Name:** Role Label  
**Purpose:** The unique display name for this role throughout the application (e.g., "Expert-1", "PostDoc-3", "Admin-1"). Appears in all budget tables, dashboards, and exports. Does not affect calculations; used only for identification.  
**Type:** Text  
**Required:** Yes  
**Default:** Auto-suggested based on role type and count (e.g., if two PostDocs already exist, the next is suggested as "PostDoc-3")  
**Validation:** Must be unique within the project; maximum 30 characters; must not be blank  
**Dependencies:** PE-01 (role type determines the label prefix)  
**Visibility:** Always shown in the Add Role form

---

**PE-03** [M — once per role]  
**Name:** Current Monthly Gross Salary (TRY)  
**Purpose:** The gross monthly salary as paid today, in Turkish Lira. This is the starting point of the entire salary projection chain. The application divides this by BS-02 (TRY/EUR rate) to get the EUR base, then applies the inflation chain year by year.  
**Type:** Decimal  
**Required:** Yes  
**Default:** None  
**Validation:** > 0  
**Dependencies:** BS-02 (TRY/EUR rate must be set so the application can display the EUR equivalent in real time)  
**Visibility:** Always shown in the Add Role form

---

**PE-04** [M — once per role]  
**Name:** FTE Fraction  
**Purpose:** The proportion of this person's total working time dedicated to the grant. Multiplied by the monthly salary and months active to produce the annual charged cost. Example: 0.7 = 70% of working time is on the grant.  
**Type:** Decimal  
**Required:** Yes  
**Default:** 1.0 (100% FTE)  
**Validation:** > 0.0 and ≤ 1.0  
**Dependencies:** None  
**Visibility:** Always shown in the Add Role form

---

**PE-05** [M — once per role]  
**Name:** Annual Inflation Rate  
**Purpose:** The expected year-on-year salary increase for this specific role, as a percentage. Drives the compounding salary chain: Year 1 = EUR base × (1 + rate), Year 2 = Year 1 × (1 + rate), and so on. Pre-filled with BS-01 but must be confirmed or overridden by the user for each role.  
**Type:** Percent  
**Required:** Yes  
**Default:** Value of BS-01 (project-level default inflation rate)  
**Validation:** ≥ 0% and ≤ 100%  
**Dependencies:** BS-01 (provides the pre-fill value; can be changed independently per role)  
**Visibility:** Always shown in the Add Role form

---

**PE-06** [M — once per role]  
**Name:** Active Project Years  
**Purpose:** The project year(s) in which this role is charged to the grant. In active years, the full 12-month salary × FTE is charged. In inactive years, the cost is zero. Selecting non-consecutive years (e.g., Year 1 and Year 3) is valid.  
**Type:** Multi-select (checkboxes for Year 1 … Year N)  
**Required:** Yes  
**Default:** None (no years pre-selected)  
**Validation:** At least one year must be selected; all selected values must be between 1 and PI-01 (project duration)  
**Dependencies:** PI-01 (project duration determines which years appear as options)  
**Visibility:** Always shown in the Add Role form; number of checkboxes equals PI-01

---

**PE-07** [M — once per role, optional]  
**Name:** Work Package Assignment (Personnel)  
**Purpose:** Tags this role to one or more Work Packages for planning and reporting visibility. Does not affect cost calculations.  
**Type:** Multi-select  
**Required:** No  
**Default:** None  
**Validation:** Selected values must be valid WP numbers (1 to PI-02)  
**Dependencies:** PI-02 (WP count must be set)  
**Visibility:** Shown in the Add Role form; hidden if PI-02 = 0 (not applicable)

---

## Group 5 — Equipment

Inputs collected once per registered equipment item. All fields directly drive the depreciation calculation.

---

**EQ-01** [M — once per item]  
**Name:** Equipment Name  
**Purpose:** Descriptive label for this item (e.g., "Laptop – PI", "Audio Recorder 2"). Used for identification in the budget table. Does not affect calculations.  
**Type:** Text  
**Required:** Yes  
**Default:** None  
**Validation:** Must not be blank; maximum 100 characters  
**Dependencies:** None  
**Visibility:** Always shown in the Add Equipment form

---

**EQ-02** [M — once per item]  
**Name:** Purchase Cost (EUR)  
**Purpose:** The total purchase price of the item in EUR, including import duties or taxes if applicable. This is the base amount from which the eligible depreciation is calculated. Also serves as the upper cap on what can be claimed.  
**Type:** Decimal  
**Required:** Yes  
**Default:** None  
**Validation:** > 0  
**Dependencies:** None  
**Visibility:** Always shown in the Add Equipment form

---

**EQ-03** [M — once per item]  
**Name:** Useful Economic Lifetime (months)  
**Purpose:** The standard economic useful life of this type of equipment, in months. Used as the denominator in the depreciation formula: (cost ÷ lifetime) × usage% × months used. Typical values: 48 months for laptops; 60 months for audio equipment.  
**Type:** Integer  
**Required:** Yes  
**Default:** None  
**Validation:** ≥ 1  
**Dependencies:** None  
**Visibility:** Always shown in the Add Equipment form

---

**EQ-04** [M — once per item]  
**Name:** Grant Usage Percentage  
**Purpose:** The fraction of the item's total use that is dedicated to grant activities, expressed as a percentage. If the laptop is used exclusively for the project, enter 100%. If shared with other projects, enter the proportionate share. Determines the eligibility cap (cost × usage%) and weights the depreciation.  
**Type:** Percent  
**Required:** Yes  
**Default:** 100  
**Validation:** > 0% and ≤ 100%  
**Dependencies:** None  
**Visibility:** Always shown in the Add Equipment form

---

**EQ-05** [M — once per item]  
**Name:** Grant Usage Months  
**Purpose:** The number of months between the purchase of this item and the end of the grant period during which the item is actively in use. Used in the depreciation formula as the numerator of the time fraction. If this value meets or exceeds the useful lifetime (EQ-03), the application applies the cap (full eligible cost = purchase cost × usage%).  
**Type:** Integer  
**Required:** Yes  
**Default:** None  
**Validation:** ≥ 1; if > (PI-01 × 12), the application displays a warning that the usage period exceeds the project duration (still accepted)  
**Dependencies:** PI-01 (project duration informs the reasonable upper bound)  
**Visibility:** Always shown in the Add Equipment form; application shows a note when the value ≥ EQ-03 ("full cost eligible — cap will apply")

---

**EQ-06** [M — once per item, optional]  
**Name:** Year of Purchase  
**Purpose:** The project year in which this item is purchased. Informational — used for planning visibility only. Does not affect the depreciation calculation in v1.  
**Type:** Integer  
**Required:** No  
**Default:** None  
**Validation:** Between 1 and PI-01 if provided  
**Dependencies:** PI-01  
**Visibility:** Shown in the Add Equipment form

---

**EQ-07** [M — once per item, optional]  
**Name:** Work Package Assignment (Equipment)  
**Purpose:** Tags this item to one or more WPs. Informational only; does not affect calculations.  
**Type:** Multi-select  
**Required:** No  
**Default:** None  
**Validation:** Selected values must be valid WP numbers (1 to PI-02)  
**Dependencies:** PI-02  
**Visibility:** Shown in the Add Equipment form

---

## Group 6 — Travel

Inputs collected once per registered trip. Fields are split between those common to all trip types and those specific to Itemized or Flat Amount trips.

### 6a — Common to All Trips

---

**TR-01** [M — once per trip]  
**Name:** Trip Name / Purpose  
**Purpose:** Descriptive label for this trip (e.g., "Fieldwork – India – Year 1", "Conference – Paris"). Used for identification in the travel budget table. Does not affect calculations.  
**Type:** Text  
**Required:** Yes  
**Default:** None  
**Validation:** Must not be blank; maximum 100 characters  
**Dependencies:** None  
**Visibility:** Always shown in the Add Trip form

---

**TR-02** [M — once per trip]  
**Name:** Trip Type  
**Purpose:** Determines the cost calculation method. **Itemized** trips are priced using EU official unit rates (flight distance band + accommodation rate + subsistence rate + domestic transport). **Flat Amount** trips use a single total entered by the user — used for conferences or when the itinerary is not yet known.  
**Type:** Enum  
**Required:** Yes  
**Default:** Itemized  
**Validation:** Must be one of: `Itemized`, `FlatAmount`  
**Dependencies:** None  
**Visibility:** Always shown in the Add Trip form; selection controls which subsequent fields are shown

---

**TR-03** [M — once per trip]  
**Name:** Project Year  
**Purpose:** The grant year in which this trip (or group of identical trips) occurs. Used to allocate the travel cost to the correct year in the annual budget breakdown. One trip entry covers one year — trips recurring across multiple years require separate entries.  
**Type:** Integer (select from Year 1 … Year N)  
**Required:** Yes  
**Default:** None  
**Validation:** Between 1 and PI-01  
**Dependencies:** PI-01  
**Visibility:** Always shown in the Add Trip form

---

**TR-04** [M — once per trip]  
**Name:** Number of Trip Instances  
**Purpose:** How many times this identical trip occurs within the specified project year. The per-instance cost (computed from all trip parameters) is multiplied by this number to produce the total trip cost.  
**Type:** Integer  
**Required:** Yes  
**Default:** 1  
**Validation:** ≥ 1  
**Dependencies:** None  
**Visibility:** Always shown in the Add Trip form

---

**TR-05** [M — once per trip, optional]  
**Name:** Work Package (Travel)  
**Purpose:** Tags this trip to a WP for planning and reporting. Informational only; does not affect calculations.  
**Type:** Select  
**Required:** No  
**Default:** None  
**Validation:** Must be a valid WP number (1 to PI-02)  
**Dependencies:** PI-02  
**Visibility:** Shown in the Add Trip form

### 6b — Itemized Trips Only

Shown only when TR-02 = `Itemized`.

---

**TR-06** [M — once per Itemized trip]  
**Name:** Destination Country  
**Purpose:** The country of destination. Used to look up the official EU accommodation rate (€/night) and subsistence rate (€/day) from the applicable Annex 2a/2b version. The looked-up rates are displayed to the user alongside the form as reference.  
**Type:** Select (country dropdown from EU rate table)  
**Required:** Yes (Itemized)  
**Default:** None  
**Validation:** Must be a country present in the applicable EUTravelRateVersion rate table (PI-04)  
**Dependencies:** PI-04 (rate version determines the available country list)  
**Visibility:** Shown only when TR-02 = `Itemized`

---

**TR-07** [M — once per Itemized trip]  
**Name:** One-Way Flight Distance (km)  
**Purpose:** The approximate one-way flight distance between the departure city and the destination city. Used to select the applicable EU distance band and look up the official flight unit cost per round trip. Enter 0 if no flight is required for this trip.  
**Type:** Integer  
**Required:** Yes (Itemized)  
**Default:** None  
**Validation:** ≥ 0. If < 400 and > 0, the application displays an informational note: "Distance under 400 km — no EU flight unit cost applies. Flight cost will be €0."  
**Dependencies:** PI-04 (rate version determines the distance band table)  
**Visibility:** Shown only when TR-02 = `Itemized`

---

**TR-08** [M — once per Itemized trip]  
**Name:** Number of Nights  
**Purpose:** The number of nights accommodation required per trip instance. Multiplied by the country accommodation rate to produce the accommodation cost component.  
**Type:** Integer  
**Required:** Yes (Itemized)  
**Default:** None  
**Validation:** ≥ 1  
**Dependencies:** TR-06 (country must be selected so the rate can be displayed)  
**Visibility:** Shown only when TR-02 = `Itemized`

---

**TR-09** [M — once per Itemized trip]  
**Name:** Number of Days  
**Purpose:** The number of days for which the daily subsistence (per diem) allowance is claimed per trip instance. Typically equals number of nights + 1 (to cover travel days) but may differ. Multiplied by the country subsistence rate.  
**Type:** Integer  
**Required:** Yes (Itemized)  
**Default:** None  
**Validation:** ≥ 1  
**Dependencies:** TR-06  
**Visibility:** Shown only when TR-02 = `Itemized`

---

**TR-10** [M — once per Itemized trip, optional]  
**Name:** Domestic Transport Cost per Instance (EUR)  
**Purpose:** A flat amount for in-country transport within the destination country per trip instance (e.g., local flights, trains, or taxis after arrival). Entered directly by the user as a known or estimated amount. Added to the other components in the per-instance cost sum.  
**Type:** Decimal  
**Required:** No (Itemized)  
**Default:** 0  
**Validation:** ≥ 0  
**Dependencies:** None  
**Visibility:** Shown only when TR-02 = `Itemized`; defaults to 0 if left blank

### 6c — Flat Amount Trips Only

Shown only when TR-02 = `FlatAmount`.

---

**TR-11** [M — once per Flat Amount trip]  
**Name:** Flat Amount per Instance (EUR)  
**Purpose:** The total cost per trip instance, entered directly by the user. No breakdown into flight, accommodation, or subsistence is required. Used for conferences or trips where the detailed itinerary is not yet known. Multiplied by TR-04 (number of instances) to produce the total trip cost.  
**Type:** Decimal  
**Required:** Yes (FlatAmount)  
**Default:** None  
**Validation:** > 0  
**Dependencies:** None  
**Visibility:** Shown only when TR-02 = `FlatAmount`

---

## Group 7 — Other Direct Costs (C3)

Inputs collected once per registered C3 cost item. Items are entered individually, one per year per cost type.

---

**OC-01** [M — once per item]  
**Name:** Item Name / Description  
**Purpose:** What this C3 cost is for (e.g., "MAXQDA software licence", "Open-access publication charges", "Translation services"). Used for identification and budget justification. Does not affect calculations directly but is required for complete documentation.  
**Type:** Text  
**Required:** Yes  
**Default:** Auto-set to "Certificate on Financial Statements (CFS)" for CFS auto-triggered items  
**Validation:** Must not be blank; maximum 150 characters  
**Dependencies:** None  
**Visibility:** Always shown; read-only and pre-filled for system-generated CFS items

---

**OC-02** [M — once per item]  
**Name:** Amount (EUR)  
**Purpose:** The total cost of this item in EUR for the specified project year. Added to the C3 total for that year, which feeds into direct costs, indirect cost base, and the overall budget total.  
**Type:** Decimal  
**Required:** Yes  
**Default:** None (pre-filled from user prompt for CFS auto-triggered items)  
**Validation:** > 0  
**Dependencies:** None  
**Visibility:** Always shown; editable for all items including CFS

---

**OC-03** [M — once per item]  
**Name:** Project Year  
**Purpose:** The grant year in which this cost will be incurred. Used to allocate the cost to the correct year in the annual budget breakdown. If the same cost occurs in multiple years, register a separate item per year.  
**Type:** Integer (select from Year 1 … Year N)  
**Required:** Yes  
**Default:** None (pre-filled from user selection in CFS prompt)  
**Validation:** Between 1 and PI-01  
**Dependencies:** PI-01  
**Visibility:** Always shown; editable for all items including CFS

---

**OC-04** [M — once per item, optional]  
**Name:** Notes / Justification  
**Purpose:** Free text field for budget narrative or justification. Supports the grant writer in documenting why each cost is needed. Informational only; does not affect calculations.  
**Type:** Text (multiline)  
**Required:** No  
**Default:** None  
**Validation:** Maximum 500 characters  
**Dependencies:** None  
**Visibility:** Shown in the Add C3 Item form; collapsed by default, expandable

---

**OC-05** [M — once per item, optional]  
**Name:** Work Package (C3)  
**Purpose:** Tags this cost to a WP. Informational only; does not affect calculations.  
**Type:** Select  
**Required:** No  
**Default:** None  
**Validation:** Must be a valid WP number (1 to PI-02)  
**Dependencies:** PI-02  
**Visibility:** Shown in the Add C3 Item form

### 7a — CFS Auto-Trigger Prompt

The following inputs appear in an auto-triggered prompt (not the normal Add C3 Item form) when the live total budget (BudgetSummary.requestedEUContributionEUR) first crosses €430,000.

---

**OC-06** [Conditional — appears once when CFS threshold crossed]  
**Name:** CFS Audit Fee (EUR)  
**Purpose:** The amount charged by the institution's external auditor for the Certificate on Financial Statements. Required by ERC rules when the requested grant exceeds €430,000. Entered as a flat amount by the user based on the expected audit cost.  
**Type:** Decimal  
**Required:** Conditional (required if user chooses to add the CFS item; may be declined with a persistent warning)  
**Default:** None  
**Validation:** > 0 if entered  
**Dependencies:** BudgetSummary.requestedEUContributionEUR > 430,000  
**Visibility:** Appears in a modal prompt only when the budget first crosses €430,000; does not appear if a CFS item already exists

---

**OC-07** [Conditional — appears alongside OC-06]  
**Name:** CFS Project Year  
**Purpose:** The grant year in which the audit is expected to take place. The CFS fee is allocated to this year in the annual budget breakdown.  
**Type:** Integer (select from Year 1 … Year N)  
**Required:** Conditional (required if OC-06 is entered)  
**Default:** The final project year (Year N), as audits typically occur near the end of the grant  
**Validation:** Between 1 and PI-01  
**Dependencies:** PI-01; OC-06 (year is only needed if the fee is entered)  
**Visibility:** Shown alongside OC-06 in the CFS prompt modal

---

## Group 8 — Subcontracting

---

**SC-01** [R — once per project]  
**Name:** Subcontracting Amount (EUR)  
**Purpose:** The total Category B subcontracting budget. Included in total direct costs but excluded from the indirect cost base (ERC rule). In version 1, this defaults to €0 as no subcontracting is planned. The field is present to allow a user to record a value if applicable.  
**Type:** Decimal  
**Required:** No  
**Default:** 0  
**Validation:** ≥ 0  
**Dependencies:** None  
**Visibility:** Shown in the Budget Settings or Subcontracting section; collapses if value remains 0

---

## Group 9 — Partner Information

**Not applicable in Version 1.**

This application supports a single-beneficiary project (one institution: Ozyegin University). There are no partner organisation entries in v1. If multi-beneficiary support is added in a future version, a Partner entity would require: partner name, partner country, partner type, and per-partner cost entries for each category.

---

## Group 10 — Administrative Data

The following fields are collected for labelling and export purposes only. They have no effect on any calculation. They are listed here for completeness.

| ID | Name | Type | Required | Notes |
|---|---|---|---|---|
| AD-01 | Project Title | Text | No | Display and export label only |
| AD-02 | PI Name | Text | No | Display and export label only |
| AD-03 | Grant Call Reference | Text | No | e.g., "ERC-CoG"; display only |
| AD-04 | Preparation Date | Date | No | Date the budget was prepared; display only |
| AD-05 | Institution Name | Text | No | Pre-filled "Ozyegin University" in v1; display only |

---

## Input Summary

| Group | Input IDs | Count |
|---|---|---|
| Project Information | PI-01 to PI-04 | 4 |
| Budget Settings | BS-01 to BS-03 | 3 |
| Work Packages | WP-01 | 1 (per WP) |
| Personnel | PE-01 to PE-07 | 7 (per role) |
| Equipment | EQ-01 to EQ-07 | 7 (per item) |
| Travel — Common | TR-01 to TR-05 | 5 (per trip) |
| Travel — Itemized | TR-06 to TR-10 | 5 (per Itemized trip) |
| Travel — Flat Amount | TR-11 | 1 (per Flat trip) |
| Other Direct Costs | OC-01 to OC-05 | 5 (per item) |
| CFS Auto-Trigger | OC-06 to OC-07 | 2 (conditional, once) |
| Subcontracting | SC-01 | 1 |
| Partner Information | — | Not applicable in v1 |
| Administrative Data | AD-01 to AD-05 | 5 (non-calculation) |
| **Total calculation-affecting inputs** | | **40** |
| **Total non-calculation inputs** | | **5** |
| **Grand total** | | **45** |

---

## Input Dependency Graph

```
PI-01 (Duration)  ────────────────────────────────────────────► PE-06 (Active Years)
                   └──────────────────────────────────────────► TR-03 (Project Year)
                   └──────────────────────────────────────────► OC-03 (Project Year)
                   └──────────────────────────────────────────► OC-07 (CFS Year)
                   └──────────────────────────────────────────► EQ-06 (Year of Purchase)
                   └──────────────────────────────────────────► EQ-05 (informs upper bound)

PI-02 (WP Count)  ────────────────────────────────────────────► PE-07, EQ-07, TR-05, OC-05

PI-03 (Call Date)  ──────────────────────────────────────────► PI-04 (auto-selects version)

PI-04 (Rate Version)  ───────────────────────────────────────► TR-06 (country list)
                       └─────────────────────────────────────► TR-07 (distance band table)

BS-01 (Default Inflation)  ──────────────────────────────────► PE-05 (pre-fills per-role rate)

BS-02 (TRY/EUR Rate)  ───────────────────────────────────────► PE-03 (enables live EUR preview)

TR-02 (Trip Type)  ──────────────────────────────────────────► TR-06 to TR-10 (Itemized only)
                    └────────────────────────────────────────► TR-11 (FlatAmount only)

TR-06 (Country)  ────────────────────────────────────────────► TR-08, TR-09 (shows EU rates)

BudgetSummary.requestedContribution > €430,000  ─────────────► OC-06, OC-07 (CFS prompt)
```

---

## Removed Spreadsheet Fields

The following fields from the source workbook are **not present** in the application, as they do not affect calculations or are superseded by the new model:

| Removed field | Original location | Reason |
|---|---|---|
| Real person names (PI, collaborators) | Salary Estimation, Personnel Costs, Gantt | Replaced by generic role labels (PE-02) |
| TRY salary display rows | Salary Estimation (informational rows) | TRY entry is now the primary salary input (PE-03); EUR equivalent shown as a live preview |
| Turkish classification codes and economic life codes | Equipment C2 | Informational/regulatory reference only; not used in calculations |
| Per-person inflation hardcoded per-cell (5 cells each) | Salary Estimation | Replaced by a single per-role inflation rate field (PE-05) |
| Static 5-year period count | Details!D2:H2 = 12 | Replaced by PI-01 (variable project duration) |
| 5-period average for travel | Travel!K6 = total/5 | Replaced by year-specific trip assignment (TR-03) |
| Work Package names in Gantt | Gantt Chart (WP row labels) | Replaced by optional WP-01 name field |
| Preparation date | Overview!C4 | Moved to Administrative Data (AD-04); non-calculation field |
| Funding type label | Overview!C8 = "Actual Costs" | Fixed assumption in v1; not a user input |
| Industry 4.0 Center (Category D) | Details row 32 | Excluded from v1 |

---

**Confidence Level: 97%**

Every input in this catalog is directly traceable to a business rule (business-rules.md) and a domain entity (domain-model.md). The input IDs and groupings are designed to map one-to-one with form screens in TASK-06 (UX Design). The single residual uncertainty: whether the Subcontracting field (SC-01) should be a single total or support multiple line items — left as a single total for v1 unless TASK-06 UX review indicates otherwise.

**Recommended Next Step:**  
Await PI approval. Once approved, proceed to TASK-06 (UX Design).
