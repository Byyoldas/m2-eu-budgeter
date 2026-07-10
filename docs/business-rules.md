# Business Rules

**Document:** TASK-03 Deliverable  
**Date:** 2026-07-10  
**Status:** Draft — Open Questions Resolved — Awaiting Approval Before Proceeding to TASK-04  
**Source documents:** project-overview.md, excel-analysis.md, EU Grants Annex 2a/2b V1.11 (01.05.2026)

---

## How to Read This Document

Each rule is written in plain business language. Rules describe **what the application must do and why** — not how the spreadsheet implemented it. Rules are grouped by domain and numbered with a prefix:

- **PS** — Project Setup
- **PE** — Personnel
- **EQ** — Equipment
- **TR** — Travel & Subsistence
- **OC** — Other Direct Costs (C3)
- **SC** — Subcontracting
- **IC** — Indirect Costs
- **PT** — Project Totals

---

## Part 1 — Project Setup

---

### PS-01 — Project Structure Definition

**Purpose:**  
Before any cost can be entered, the user must define the project's structural parameters. These parameters govern how many calculation periods exist, how the budget is organised, and which shared assumptions apply throughout the project.

**Inputs:**

| Field | Type | Required | Description |
|---|---|---|---|
| Project duration (years) | Integer | Yes | Number of full grant years. Minimum 1, maximum 7. Each year = 12 months. |
| Number of Work Packages | Integer | Yes | Number of Work Packages (WPs) the project is divided into. Minimum 1, maximum 10. |
| Annual salary inflation rate | Decimal (%) | Yes | Default year-on-year salary increase assumption applied to all personnel unless overridden at the role level. Expressed as a percentage (e.g., 15 for 15%). |
| TRY/EUR exchange rate | Decimal | Yes | Current Turkish Lira to Euro conversion rate. Used to convert TRY-denominated salaries to EUR. Applied uniformly for the entire project duration. |
| Indirect cost rate | Decimal (%) | Yes | Percentage applied to total direct costs to calculate indirect (overhead) costs. Default = 25 per ERC rules. Configurable to support other rates if needed. |
| Project title | Text | No | For labelling purposes only; does not affect calculations. |
| PI name | Text | No | For labelling purposes only; does not affect calculations. |
| Grant call reference | Text | No | For labelling purposes only (e.g., ERC-CoG). |

**Outputs:**  
- A project configuration record that all cost rules reference.
- A list of project years (Year 1 … Year N) used to structure all budget inputs.
- A list of Work Packages (WP-1 … WP-M) used for allocation tracking.

**Dependencies:**  
None. This is the root rule — all other rules depend on it.

**Conditions:**  
- The project configuration must be completed before any cost entries are created.
- If the user changes the project duration after costs have been entered, the application must warn that existing entries outside the new range will be excluded from totals.

**Exceptions:**  
- If the indirect cost rate is changed from 25%, the application must display a warning that this deviates from the ERC standard and require explicit user confirmation.

**Examples:**  
- 5-year project, 5 WPs, 15% inflation, 50.62 TRY/EUR, 25% indirect rate → produces Year 1 through Year 5, WP-1 through WP-5.
- 3-year project, 3 WPs → only Year 1–3 are available for cost entry; no year 4 or beyond appears.

**Validation:**  
- Duration ≥ 1 and ≤ 7.
- Number of WPs ≥ 1 and ≤ 10.
- Inflation rate ≥ 0% and ≤ 100%.
- TRY/EUR rate must be a positive number greater than zero.
- Indirect cost rate ≥ 0% and ≤ 50%.

---

## Part 2 — Personnel Costs

---

### PE-01 — Personnel Role Registration

**Purpose:**  
Each staff member who will be charged to the grant must be registered as a named role. The application uses generic role names (not real person names) to keep the model reusable and privacy-safe. Each role has a set of employment parameters that determine how much it costs the project.

**Inputs:**

| Field | Type | Required | Description |
|---|---|---|---|
| Role name | Enum / text | Yes | Generic label: PI, Expert-1, Expert-2, PostDoc-1, PostDoc-2, … PostDoc-N, Admin-1, Admin-2, … etc. |
| Current monthly gross salary | Decimal (TRY) | Yes | The gross monthly salary as paid today, in Turkish Lira. Used as the base for all year-by-year projections. |
| FTE fraction | Decimal (0.0–1.0) | Yes | The share of this person's working time dedicated to the grant. Example: 0.7 = 70% of working time. |
| Active project years | Multi-select | Yes | The project year(s) in which this role is charged. Example: Year 1 only, or Years 1–5, or Years 2–3. |
| Annual inflation rate | Decimal (%) | Yes | The expected annual salary raise for this role, expressed as a percentage (e.g., 20 for 20%). Each role has its own rate. The application pre-fills this field with the project-level default from PS-01, but the user must confirm or override it per role. |
| Work Package assignment | Multi-select | No | Which WP(s) this role works on. Used for planning and tracking; does not currently affect cost calculations. |

**Outputs:**  
- A registered role record with all employment parameters.
- This record is consumed by PE-02 and PE-03.

**Dependencies:**  
- PS-01 (project structure must exist).

**Conditions:**  
- Multiple roles of the same type are allowed (e.g., PostDoc-1 and PostDoc-2 are separate registrations).
- The PI role is unique; only one PI may be registered.
- A role registered for Year 1 only will contribute to cost calculations only in Year 1 and will show zero in all other years.

**Exceptions:**  
- If a role's active years span a gap (e.g., active in Year 1 and Year 3 but not Year 2), each active year contributes its projected salary to totals and Year 2 contributes zero.

**Examples:**  
- PI, salary 227,900 TRY/month, 0.7 FTE, active all 5 years, 20% inflation (role-specific, higher than project default).
- PostDoc-1, salary 151,860 TRY/month, 1.0 FTE, active Year 2 only, 15% inflation (matches project default, confirmed by user).
- Expert-1, salary 164,515 TRY/month, 0.4 FTE, active Year 1 only, 15% inflation.

**Validation:**  
- Salary must be a positive number greater than zero.
- FTE fraction must be between 0.0 (exclusive) and 1.0 (inclusive).
- At least one active year must be selected.
- Role name must be unique within the project.

---

### PE-02 — Annual Salary Projection (EUR)

**Purpose:**  
Starting from today's salary in TRY, the application must calculate the expected monthly salary for each project year in EUR. Salaries grow year-on-year due to inflation, and are converted to EUR using the project exchange rate. This projection is the foundation of all personnel cost calculations.

**Inputs:**

| Field | Source |
|---|---|
| Current monthly salary (TRY) | PE-01 — role record |
| TRY/EUR exchange rate | PS-01 — project setup |
| Annual inflation rate (%) | PE-01 — role-specific rate (required per role; pre-filled with PS-01 default) |
| Number of project years | PS-01 — project setup |

**Outputs:**  
- A table of projected monthly salaries in EUR for each project year: Year 1 … Year N.

**Algorithm:**

Step 1 — Convert base salary to EUR:

> EUR Base Monthly Salary = Current Monthly Salary (TRY) ÷ TRY/EUR Exchange Rate

Step 2 — Apply annual inflation compounding, year by year:

> Year 1 Monthly Salary (EUR) = EUR Base Monthly Salary × (1 + Inflation Rate)  
> Year 2 Monthly Salary (EUR) = Year 1 Monthly Salary × (1 + Inflation Rate)  
> Year N Monthly Salary (EUR) = Year N-1 Monthly Salary × (1 + Inflation Rate)

Each year's salary is derived from the immediately preceding year (compounding chain). The base salary is not Year 0 of the chain — the first project year already includes one full year of inflation.

**Dependencies:**  
- PE-01 (role record with base salary and inflation rate).
- PS-01 (TRY/EUR rate and project duration).

**Conditions:**  
- If a role is only active in specific years (e.g., Year 2 only), the salary projection still covers all years — but only the relevant year's salary is used in PE-03. The unused year projections are calculated silently.
- The inflation chain is computed from the base salary regardless of which years are active.

**Exceptions:**  
- If the inflation rate is 0%, all year projections equal the EUR base salary (no growth).

**Examples:**  
- Base: 227,900 TRY ÷ 50.62 = €4,500.20/month. Inflation 20%.
  - Year 1: €4,500 × 1.20 = €5,400/month
  - Year 2: €5,400 × 1.20 = €6,480/month
  - Year 3: €6,480 × 1.20 = €7,776/month
  - Year 4: €7,776 × 1.20 = €9,331/month
  - Year 5: €9,331 × 1.20 = €11,197/month

- Base: 151,860 TRY ÷ 50.62 = €3,000/month. Inflation 15%.
  - Year 1: €3,000 × 1.15 = €3,450/month
  - Year 2: €3,450 × 1.15 = €3,968/month

**Validation:**  
- All projected salaries must be positive numbers.
- Year N salary must always be greater than or equal to Year N-1 salary when inflation rate ≥ 0%.

---

### PE-03 — Annual Personnel Cost per Role

**Purpose:**  
Convert the year-specific monthly salary into an annual cost charged to the grant, by accounting for the number of months the role is active in that year and the fraction of their time dedicated to the project.

**Inputs:**

| Field | Source |
|---|---|
| Monthly salary (EUR) for Year Y | PE-02 — salary projection for this year |
| FTE fraction | PE-01 — role record |
| Active months in Year Y | Fixed: 12 if this year is in the role's active years; 0 otherwise. No partial-year entry — all staff work the full 12 months in each active year. |

**Outputs:**  
- Personnel cost (EUR) for this role in Year Y.
- Total personnel cost for this role across all years (sum).

**Algorithm:**

> Annual Cost (role, year Y) = Monthly Salary (EUR, year Y) × 12 months × FTE Fraction

If the role is not active in Year Y:

> Annual Cost (role, year Y) = 0

Total cost for the role:

> Total Personnel Cost (role) = Sum of Annual Cost (role, year Y) for all Y in active years

**Dependencies:**  
- PE-02 (salary projection per year).
- PE-01 (FTE fraction, active years).

**Conditions:**  
- All active staff work the full 12 months in each of their registered active years. There is no partial-year or mid-year start/end entry. Active months is always 12 for active years and 0 for inactive years.
- If two roles have the same monthly salary but different FTE fractions, their annual costs differ proportionally.

**Exceptions:**  
- None. A role with zero active months always contributes zero cost.

**Examples:**  
- PI, Year 1 salary €5,400/month, FTE 0.7, 12 months active:
  - Year 1 cost = €5,400 × 12 × 0.7 = **€45,360**

- PostDoc-1, Year 2 salary €3,968/month, FTE 1.0, active Year 2 only (12 months):
  - Year 2 cost = €3,968 × 12 × 1.0 = **€47,616**
  - All other years = €0

- Expert-1, Year 1 salary €3,737/month, FTE 0.4, active Year 1 only:
  - Year 1 cost = €3,737 × 12 × 0.4 = **€17,940**

**Validation:**  
- Annual cost must be a non-negative number.
- Annual cost must be zero for any year where the role is not active.
- FTE × months must not exceed 1.0 × 12 = 12 person-months per year per role.

---

### PE-04 — Total Personnel Cost (Category A)

**Purpose:**  
Sum all individual personnel costs across all roles and all project years to produce the total Category A figure used in the final budget.

**Inputs:**  
- PE-03 — Annual cost per role per year, for all registered roles.

**Outputs:**  
- Total Category A Personnel Cost (EUR) — a single number representing the entire personnel budget across the project.
- Optionally: personnel cost per year and per role for the budget dashboard.

**Algorithm:**

> Category A Total = Sum of Annual Cost (role, year) for all roles and all years

**Dependencies:**  
- PE-03 (must be computed for all roles before this total can be produced).

**Conditions:**  
- If no personnel roles have been registered, Category A = 0.

**Exceptions:**  
- None.

**Examples:**  
- 1 PI (5 years) + 6 Post-Docs (1 year each) + 1 Admin (5 years) + 2 Experts (1 year each) → sum of all PE-03 outputs.

**Validation:**  
- Must equal the arithmetic sum of all PE-03 outputs — no rounding shortcut.
- Must be ≥ 0.

---

## Part 3 — Equipment Costs

---

### EQ-01 — Equipment Item Registration

**Purpose:**  
Each piece of equipment purchased during the project must be registered so that its eligible depreciation cost can be calculated. Only the portion of the equipment's value that corresponds to its use during the grant period — and its use for the grant's purposes — is eligible for reimbursement.

**Inputs:**

| Field | Type | Required | Description |
|---|---|---|---|
| Item name / description | Text | Yes | Descriptive label (e.g., "Laptop – PI", "Audio Recorder 1"). |
| Purchase cost | Decimal (EUR) | Yes | Total purchase price of the item, including any applicable taxes or import duties. |
| Useful economic lifetime | Integer (months) | Yes | The standard economic useful life of this type of equipment. For laptops this is typically 48 months; for audio equipment 60 months. |
| Grant usage percentage | Decimal (%) | Yes | The share of the equipment's total use dedicated to grant activities. If a laptop is used exclusively for the grant, this is 100%. If shared with other projects, enter the proportionate share. |
| Number of months used for the grant | Integer (months) | Yes | The number of months between equipment purchase and end of the grant period during which the item is in use. Cannot exceed the project duration in months. |
| Year of purchase | Integer | No | For planning purposes; does not affect the depreciation calculation directly. |
| Work Package assignment | Multi-select | No | WP(s) the equipment supports. |

**Outputs:**  
- An equipment record consumed by EQ-02.

**Dependencies:**  
- PS-01 (project duration in months must be known).

**Conditions:**  
- Multiple items of the same type (e.g., three identical laptops) must be registered as separate items. Each contributes independently to the total.
- Purchase cost should reflect the actual paid cost in EUR. If the invoice is in another currency, convert to EUR before entry.

**Exceptions:**  
- Equipment items with a grant usage percentage of 0% contribute zero eligible cost and should not be registered.

**Examples:**  
- Laptop: purchase cost €2,500, useful lifetime 48 months, grant usage 100%, used 55 months during the grant.
- Audio recorder: purchase cost €60, useful lifetime 60 months, grant usage 100%, used 36 months during the grant.

**Validation:**  
- Purchase cost > 0.
- Useful lifetime ≥ 1 month.
- Grant usage percentage > 0% and ≤ 100%.
- Months used for the grant > 0 and ≤ (project duration in months + reasonable post-project period for late purchases, capped at 60 months).

---

### EQ-02 — Equipment Eligible Depreciation

**Purpose:**  
Calculate the euro amount that can be claimed for each equipment item. The eligible amount represents the fraction of the equipment's value consumed during the grant, weighted by the share of use dedicated to grant activities. The eligible amount can never exceed the item's full grant-attributable value.

**Inputs:**

| Field | Source |
|---|---|
| Purchase cost (EUR) | EQ-01 |
| Useful economic lifetime (months) | EQ-01 |
| Grant usage percentage | EQ-01 |
| Months used for the grant | EQ-01 |

**Outputs:**  
- Eligible depreciation amount (EUR) for this equipment item.

**Algorithm:**

Step 1 — Calculate the theoretical depreciation:

> Theoretical Eligible Amount = (Purchase Cost ÷ Useful Lifetime in Months) × Grant Usage % × Months Used for the Grant

Step 2 — Calculate the maximum eligible amount (cap):

> Maximum Eligible Amount = Purchase Cost × Grant Usage %

Step 3 — Apply the cap:

> Eligible Depreciation = the lesser of Theoretical Eligible Amount and Maximum Eligible Amount

In plain terms: if the equipment is used for longer than its economic lifetime (e.g., a laptop used for 55 months with a 48-month lifetime), the full purchase cost at the grant usage percentage is claimable — you cannot claim more than you paid. If the equipment is only used for part of its lifetime, you claim only the proportionate depreciation.

**Dependencies:**  
- EQ-01 (item inputs).

**Conditions:**  
- If Months Used ≥ Useful Lifetime, the cap applies automatically. The application should display a note indicating the item is fully depreciated within the grant period.
- If Grant Usage % = 100%, the cap equals the purchase cost.

**Exceptions:**  
- If the item data is incomplete (any required field missing or zero), no eligible amount is calculated and the row shows an error.

**Examples:**  
- Laptop: (€2,500 ÷ 48) × 100% × 55 months = €2,864. Cap = €2,500 × 100% = €2,500. Since €2,864 > €2,500, **eligible = €2,500** (capped).
- Audio recorder: (€60 ÷ 60) × 100% × 36 months = €36. Cap = €60. Since €36 ≤ €60, **eligible = €36**.
- Laptop at 80% grant usage: (€2,500 ÷ 48) × 80% × 55 = €2,292. Cap = €2,500 × 80% = €2,000. Since €2,292 > €2,000, **eligible = €2,000** (capped).

**Validation:**  
- Eligible amount must be > 0.
- Eligible amount must not exceed Purchase Cost × Grant Usage %.
- Eligible amount must not exceed Purchase Cost.

---

### EQ-03 — Total Equipment Cost (Category C2)

**Purpose:**  
Sum all eligible depreciation amounts across all registered equipment items to produce the total Category C2 figure used in the final budget.

**Inputs:**  
- EQ-02 — eligible depreciation for all registered equipment items.

**Outputs:**  
- Total Category C2 Equipment Cost (EUR).

**Algorithm:**

> Category C2 Total = Sum of Eligible Depreciation for all equipment items

**Dependencies:**  
- EQ-02 (must be computed for all items).

**Conditions:**  
- If no equipment items are registered, Category C2 = 0.

**Exceptions:**  
- None.

**Validation:**  
- Must equal the arithmetic sum of all EQ-02 outputs.
- Must be ≥ 0.

---

## Part 4 — Travel & Subsistence

---

### TR-01 — Trip Registration

**Purpose:**  
Each planned trip must be registered individually. A trip is a single journey to one destination for a specific purpose (fieldwork visit, conference, collaboration meeting). The user defines the parameters of the trip — where, how long, how far, and how many times it will occur. The application then calculates the eligible cost automatically from official EU unit rates.

**Inputs:**

| Field | Type | Required | Description |
|---|---|---|---|
| Trip name / purpose | Text | Yes | Brief description (e.g., "Fieldwork – India – Year 1", "Conference – France"). |
| Trip type | Enum | Yes | **Itemized** — costs are broken down by flight, accommodation, and subsistence (default). **Flat amount** — user enters a single total cost directly. Use for conferences or trips where the detailed breakdown is not yet known. |
| Destination country | Select (country list) | Yes (Itemized) | The country of destination. Used to look up accommodation and subsistence rates. Not required for flat-amount trips. |
| One-way flight distance (km) | Integer | Yes (Itemized) | The approximate one-way flight distance in kilometres. Used to determine the applicable distance band for the EU flight unit cost. Enter 0 if no flight is required. |
| Number of nights | Integer | Yes (Itemized) | Number of nights accommodation required per trip instance. |
| Number of days | Integer | Yes (Itemized) | Number of days for which the daily subsistence allowance is claimed per trip instance. Typically equals nights + 1, but may differ. |
| Domestic transport cost (per instance) | Decimal (EUR) | No (Itemized) | Flat amount for in-country transport within the destination country (e.g., internal flights, trains, taxis). Entered by the user as a known or estimated cost. |
| Flat amount (per instance) | Decimal (EUR) | Yes (Flat only) | Total cost per trip instance entered directly by the user. No breakdown is required. Used for conferences or when the destination and itinerary are not yet fixed. |
| Number of trip instances | Integer | Yes | How many times this trip will occur. |
| Project year | Select | Yes | The project year in which this trip occurs. One entry per year — register separate entries for trips in different years. |
| Work Package | Select | No | The WP this travel supports. |

**Outputs:**  
- A trip record consumed by TR-02, TR-03, TR-04, and TR-05.

**Dependencies:**  
- PS-01 (project structure — year selection must be within valid project years).
- EU rate table (TR-02 through TR-04) — required for Itemized trips.

**Conditions:**  
- If a trip repeats across multiple years (e.g., 2 fieldwork visits in Year 1 and 3 visits in Year 3), register two separate trip entries — one per year — each with the correct number of instances.
- For Itemized trips, the application displays the official EU unit cost rates (accommodation, subsistence, flight band) for the selected country alongside the form.
- For Flat amount trips, the application accepts the entered amount without applying EU rate lookups. The user is responsible for ensuring the amount is within eligible limits.
- Domestic transport cost is always entered as a flat amount by the user. The application does not automatically calculate in-country transport costs.

**Exceptions:**  
- Trips shorter than 400 km do not qualify for the EU flight distance band rates. If a flight distance under 400 km is entered, the flight cost component is set to €0 and the user is informed. Accommodation and subsistence still apply.

**Examples:**  
- Fieldwork visit to India (Itemized): destination India, one-way 5,800 km, 4 nights, 5 days, domestic transport €340/instance, 4 instances, Year 1.
- Conference in France (Itemized): destination France, one-way 2,100 km, 5 nights, 6 days, no domestic transport, 3 instances, Year 2.
- Conference with unknown itinerary (Flat amount): flat cost €2,000/instance, 3 instances, Year 3.

**Validation:**  
- Trip type must be selected.
- For Itemized trips: destination country required; flight distance ≥ 0; nights ≥ 1; days ≥ 1.
- For Flat trips: flat amount per instance > 0.
- Number of instances ≥ 1.
- One project year must be selected per trip entry.
- Destination country must exist in the EU rate table (Itemized only).

---

### TR-02 — Flight Cost Lookup

**Purpose:**  
Determine the eligible flight cost per trip instance by looking up the official EU distance band unit cost that applies to the flight distance. The EU provides fixed unit costs for air travel based on the one-way distance between origin and destination.

**Inputs:**

| Field | Source |
|---|---|
| One-way flight distance (km) | TR-01 — trip record |

**Outputs:**  
- EU official flight unit cost (EUR per round trip).

**Rate Table** (source: EU Grants Annex 2a/2b, V1.11, effective 13 May 2025 — for ERC calls with opening date from this date):

| One-way distance band | Flight unit cost per trip |
|---|---|
| Less than 400 km | Not applicable — use rail/other transport rates |
| 400 – 600 km | €340 |
| 601 – 1,600 km | €365 |
| 1,601 – 2,500 km | €429 |
| 2,501 – 3,500 km | €541 |
| 3,501 – 4,500 km | €743 |
| 4,501 – 6,000 km | €857 |
| 6,001 – 7,500 km | €1,021 |
| 7,501 – 10,000 km | €1,250 |
| 10,001 km and above | €1,595 |

> **Important:** The application must store the rate table internally and must not rely on the user to know or enter these amounts. The applicable rate is determined automatically from the entered flight distance.

**Dependencies:**  
- TR-01 (flight distance is required to select the correct band).

**Conditions:**  
- The distance used is always the **one-way** distance. The rate covers the full round trip.
- If the entered distance falls exactly on a band boundary (e.g., exactly 600 km), apply the lower band's rate (400–600).
- For distances under 400 km, flag the entry and advise the user to use rail transport or enter the distance as 0 to exclude a flight cost from this trip.

**Exceptions:**  
- Trips that are local/domestic (within the same country) typically do not involve a flight. In this case the flight distance should be entered as 0 and no flight cost applies.
- The EU rate table is versioned. The rate version that applies is determined by the ERC call opening date. The application must store the version date and display which version is active.

**Examples:**  
- Istanbul to London: ~2,500 km one-way → band 1,601–2,500 km → flight cost = **€429/trip**.
- Istanbul to Mumbai: ~5,800 km one-way → band 4,501–6,000 km → flight cost = **€857/trip**.
- Istanbul to Melbourne: ~13,800 km one-way → band 10,001+ km → flight cost = **€1,595/trip**.
- Istanbul to Vienna: ~1,500 km one-way → band 601–1,600 km → flight cost = **€365/trip**.
- Istanbul to Ankara: ~350 km one-way → under 400 km → no flight cost applicable.

**Validation:**  
- Flight distance ≥ 0.
- If distance ≥ 400 km, a rate must be found in the table.
- Rate must be a positive number.

---

### TR-03 — Accommodation Cost

**Purpose:**  
Calculate the eligible accommodation cost for a trip based on the number of nights and the EU official maximum accommodation rate for the destination country. The EU sets an upper limit on how much can be claimed per night for each country.

**Inputs:**

| Field | Source |
|---|---|
| Destination country | TR-01 — trip record |
| Number of nights per trip instance | TR-01 — trip record |

**Outputs:**  
- Accommodation cost per trip instance (EUR).

**Algorithm:**

> Accommodation Cost per Instance = Accommodation Rate (€/night, by country) × Number of Nights

**Rate Table** (source: EU Grants Annex 2a/2b V1.11, effective from 13 May 2025 — selected countries):

| Country | Accommodation rate (€/night) |
|---|---|
| Australia | €135 |
| Austria | €158 |
| France | €212 |
| India | €195 |
| Spain | €154 |
| Turkey | €165 |
| United Kingdom | €209 |
| United States | €200 |

> The full country list from Annex 2a/2b is embedded in the application. Only a representative selection is shown here. The application must contain all countries listed in the official Annex.

**Dependencies:**  
- TR-01 (country and nights).
- EU rate table (accommodation rates by country).

**Conditions:**  
- The rate shown and used is the EU official upper limit. In practice, actual accommodation cost should not exceed this rate to remain fully eligible.
- The application displays the rate as a reference alongside the trip entry.

**Exceptions:**  
- If the destination country is not found in the Annex 2a/2b table, the application must alert the user and ask them to enter an accommodation cost manually with a note that it cannot be pre-validated against EU limits.

**Examples:**  
- France, 5 nights: €212 × 5 = **€1,060 accommodation cost**.
- India, 4 nights: €195 × 4 = **€780 accommodation cost**.
- Turkey, 3 nights: €165 × 3 = **€495 accommodation cost**.

**Validation:**  
- Accommodation rate > 0.
- Number of nights ≥ 1.
- Accommodation cost per instance > 0.

---

### TR-04 — Daily Subsistence Allowance

**Purpose:**  
Calculate the eligible daily subsistence (per diem) allowance for a trip. This covers meals and incidental expenses. The EU specifies a maximum daily rate by country.

**Inputs:**

| Field | Source |
|---|---|
| Destination country | TR-01 — trip record |
| Number of days per trip instance | TR-01 — trip record |

**Outputs:**  
- Subsistence cost per trip instance (EUR).

**Algorithm:**

> Subsistence Cost per Instance = Subsistence Rate (€/day, by country) × Number of Days

**Rate Table** (source: EU Grants Annex 2a/2b V1.11, effective from 13 May 2025 — selected countries):

| Country | Daily subsistence rate (€/day) |
|---|---|
| Australia | €75 |
| Austria | €131 |
| France | €127 |
| India | €50 |
| Spain | €101 |
| Turkey | €55 |
| United Kingdom | €125 |
| United States | €80 |

**Dependencies:**  
- TR-01 (country and days).
- EU rate table (subsistence rates by country).

**Conditions:**  
- The number of days eligible for subsistence may differ from the number of nights. Travel days (arrival/departure) may count as half-days under some grant rules. For simplicity in v1, the user enters the number of claimable full days and the app applies the full daily rate.
- The rate displayed is the EU upper limit.

**Exceptions:**  
- Same as TR-03: if the country is not in the table, alert the user.

**Examples:**  
- France, 6 days: €127 × 6 = **€762 subsistence**.
- India, 5 days (4 trips): €50 × 5 = €250 per instance.
- Austria, 6 days: €131 × 6 = **€786 subsistence**.

**Validation:**  
- Subsistence rate > 0.
- Number of days ≥ 1.
- Subsistence cost per instance > 0.

---

### TR-05 — Per-Trip Total Cost

**Purpose:**  
Combine all cost components for one trip instance into a single per-instance cost, then multiply by the number of trip instances to produce the total cost for that trip entry. The calculation differs depending on whether the trip is Itemized or Flat amount.

**Inputs:**

| Field | Source | Applies to |
|---|---|---|
| Flight cost per trip instance | TR-02 | Itemized only |
| Accommodation cost per trip instance | TR-03 | Itemized only |
| Subsistence cost per trip instance | TR-04 | Itemized only |
| Domestic transport cost per trip instance | TR-01 (user-entered flat amount) | Itemized only (optional) |
| Flat amount per trip instance | TR-01 (user-entered) | Flat amount only |
| Number of trip instances | TR-01 | Both |

**Outputs:**  
- Total trip cost (EUR) for this trip entry, across all instances.
- Per-instance cost breakdown for transparency display.

**Algorithm — Itemized trips:**

> Per-Instance Cost = Flight Cost + Accommodation Cost + Subsistence Cost + Domestic Transport Cost  
> Total Trip Cost = Per-Instance Cost × Number of Trip Instances

**Algorithm — Flat amount trips:**

> Per-Instance Cost = Flat Amount (as entered by user)  
> Total Trip Cost = Per-Instance Cost × Number of Trip Instances

**Dependencies:**  
- TR-02, TR-03, TR-04 for Itemized trips.
- TR-01 for all trips (number of instances; flat amount for flat-type trips; domestic transport for itemized trips).

**Conditions:**  
- If flight distance is 0 km (no air travel), the flight cost component is €0; the rest of the sum still applies.
- If no domestic transport cost was entered for an Itemized trip, that component is treated as €0.
- The cost breakdown must be stored and displayable for auditing purposes.

**Exceptions:**  
- None.

**Examples:**  
- India fieldwork visit (Itemized, 1 instance): flight €857 + accommodation €780 (4 nights × €195) + subsistence €250 (5 days × €50) + domestic transport €340 = **€2,227 per instance**.
  - 4 instances: total = **€8,908**.
- France conference (Itemized, 1 instance): flight €429 + accommodation €1,060 (5 nights × €212) + subsistence €762 (6 days × €127) + domestic transport €0 = **€2,251 per instance**.
  - 3 instances: total = **€6,753**.
- Conference flat amount: €2,000 per instance × 3 instances = **€6,000 total**.

**Validation:**  
- Per-instance cost must be ≥ 0.
- Total trip cost = per-instance cost × number of instances (must match exactly).
- For Flat trips: per-instance cost = the flat amount entered in TR-01.

---

### TR-06 — Annual Travel Budget (Category C1)

**Purpose:**  
Aggregate all individual trip costs by project year to produce the annual and total Category C1 travel budget. Unlike the workbook (which averaged all travel equally across all years), the application assigns travel costs to the specific years in which trips are planned. This provides an accurate per-year breakdown.

**Inputs:**  
- TR-05 — total cost for each registered trip, with year assignment.

**Outputs:**  
- Travel cost per project year (Year 1, Year 2, … Year N).
- Total Category C1 Travel Cost across all years (EUR).

**Algorithm:**

> Travel Cost (Year Y) = Sum of Total Trip Cost for all trip entries assigned to Year Y  
> Category C1 Total = Sum of Travel Cost (Year Y) for all project years

**Dependencies:**  
- TR-05 (all trip costs must be computed).
- PS-01 (year structure).

**Conditions:**  
- A trip entry assigned to multiple years contributes its full total cost to each assigned year independently, OR (preferred design) the user specifies how many instances fall in each year, and the total instances sum to the number specified in TR-01.
- If no trips are registered, Category C1 = 0.

**Exceptions:**  
- None.

**Validation:**  
- Sum of per-year travel costs must equal the total of all TR-05 outputs.
- No year may have a negative travel cost.

---

## Part 5 — Other Direct Costs (C3)

---

### OC-01 — Other Direct Cost Item Registration

**Purpose:**  
Register each item in the "Other Goods, Works and Services" category (C3). These are costs that do not fit into Personnel, Equipment, or Travel but are directly needed for the project — such as software subscriptions, open-access publication charges, translation services, financial audit certificates, and fieldwork costs.

**Inputs:**

| Field | Type | Required | Description |
|---|---|---|---|
| Item name / description | Text | Yes | What the cost is for (e.g., "MAXQDA software licence", "Open-access publication charges"). |
| Amount | Decimal (EUR) | Yes | Total amount for this cost item over the project. |
| Project year | Select | Yes | The year in which this cost will be incurred. If spread across multiple years, create one entry per year. |
| Work Package | Select | No | The WP this cost is associated with. |
| Notes / justification | Text | No | Optional free text for budget justification purposes. |

**Outputs:**  
- A registered cost item record used by OC-02.

**Dependencies:**  
- PS-01 (year must be a valid project year).

**Conditions:**  
- If a cost spans multiple years (e.g., €5,000/year for publications in years 3, 4, and 5), register three separate items — one per year — with the amount for that year.
- Amounts must be in EUR. If invoiced in another currency, the user converts before entry.

**Exceptions:**  
- Category D (internally invoiced goods and services) is excluded from this category and from the application entirely in version 1.

**Examples:**  
- MAXQDA software: €9,870, Year 1.
- Open-access publications: €5,000, Year 3; €5,000, Year 4; €5,000, Year 5 (three entries).
- Translation services: €3,000, Year 3.
- Certificate on Financial Statements (CFS): €12,000, Year 4.
- Fieldwork costs: €20,000, Year 1.

**Validation:**  
- Amount > 0.
- A valid project year must be selected.
- Item name must not be blank.

---

### OC-02 — Certificate on Financial Statements (CFS) Auto-Trigger

**Purpose:**  
ERC rules require a Certificate on Financial Statements (CFS) when the total requested EU contribution for a project exceeds €430,000. The application must monitor the running total and, once this threshold is crossed, automatically prompt the user to register the CFS cost as a C3 item. The CFS amount is not calculated — it is entered by the user as a flat amount based on their institution's actual audit fee.

**Inputs:**

| Field | Source |
|---|---|
| Total Requested EU Contribution (live running total) | PT-03 — computed continuously as costs are entered |
| CFS eligibility threshold | Fixed = €430,000 |
| CFS amount (EUR) | User entry — entered as a flat amount when prompted |
| CFS project year | User selection — the year in which the audit is expected to occur |

**Outputs:**  
- A CFS cost item automatically added to the C3 list (equivalent to an OC-01 registration).
- A warning/prompt displayed in the UI when the threshold is first crossed.

**Algorithm:**

Step 1 — At any point during budget entry, evaluate:

> If Total Requested EU Contribution > €430,000 → CFS Required = True

Step 2 — If CFS Required = True and no CFS item has yet been registered:

> Display prompt: "Your total budget has exceeded €430,000. A Certificate on Financial Statements (CFS) is required by ERC rules. Please enter the audit fee amount and the year it will be incurred."

Step 3 — User enters the CFS flat amount and selects the year. The application creates a C3 line item automatically labelled "Certificate on Financial Statements (CFS)".

Step 4 — If the budget later drops back below €430,000 (due to edits), the application must warn the user that the CFS requirement may no longer apply and offer to remove the CFS line.

**Dependencies:**  
- PT-03 (live total must be available throughout the budget entry session, not only at the end).
- OC-01 (CFS is registered as a standard C3 item once confirmed by the user).

**Conditions:**  
- The CFS prompt must appear exactly once — it must not repeat each time the user saves or navigates. Once the user has entered the CFS amount, the prompt is dismissed.
- Only one CFS item may exist at a time. If the user has already registered a CFS manually in OC-01, the auto-trigger must detect this and not create a duplicate.
- The CFS item is labelled "Certificate on Financial Statements (CFS)" and is included in the C3 total and the indirect cost base like any other C3 item.

**Exceptions:**  
- If the user explicitly declines to add a CFS after the prompt, the application must record this decision and display a persistent warning until either a CFS is added or the budget drops back below the threshold.

**Examples:**  
- Budget grows to €450,000 → prompt appears → user enters €12,000 CFS fee for Year 4 → CFS item created → C3 total increases by €12,000 → indirect costs recalculate → total budget is now €465,000 (note: adding CFS may push the total further above the threshold, which is expected and correct).
- Budget is €425,000 → no prompt. User adds a new PostDoc → budget crosses €430,000 → prompt appears immediately.

**Validation:**  
- CFS amount must be > 0 if entered.
- CFS item must be assigned to a valid project year.
- No more than one CFS item may exist in the C3 list.
- If Total Requested EU Contribution > €430,000 and no CFS item exists, the application must display a persistent warning badge on the budget dashboard.

---

### OC-03 — Total Other Direct Costs (Category C3)

**Purpose:**  
Sum all registered C3 cost items — including any CFS item added via OC-02 — by project year and in total to produce the Category C3 figure used in the final budget.

**Inputs:**  
- OC-01 — all manually registered C3 items with amounts and years.
- OC-02 — the CFS item (if triggered and confirmed by the user).

**Outputs:**  
- C3 cost per project year.
- Total Category C3 Cost (EUR).

**Algorithm:**

> C3 Cost (Year Y) = Sum of all OC-01 and OC-02 item amounts assigned to Year Y  
> Category C3 Total = Sum of C3 Cost (Year Y) for all years

**Dependencies:**  
- OC-01, OC-02.

**Conditions:**  
- If no C3 items are registered, Category C3 = 0.

**Exceptions:**  
- None.

**Validation:**  
- Category C3 Total = sum of all individual OC-01 and OC-02 item amounts.
- No year may have a negative C3 cost.

---

## Part 6 — Subcontracting (Category B)

---

### SC-01 — Subcontracting (Placeholder)

**Purpose:**  
Record any subcontracting costs charged to the project. Subcontracting refers to tasks delegated to a third party that form an integral part of the project but are not performed by grant staff.

**Status in Version 1:** This category is included as a registered line item with a zero value. The interface will allow a user to enter a subcontracting amount if applicable, but no calculation logic is required. The zero value flows through to project totals.

**Inputs:**

| Field | Type | Required | Description |
|---|---|---|---|
| Subcontracting amount | Decimal (EUR) | No | Total subcontracting cost. Default = 0. |

**Outputs:**  
- Category B Total = entered amount (default €0).

**Validation:**  
- Amount ≥ 0.

---

## Part 7 — Work Packages

---

### WP-01 — Work Package Structure

**Purpose:**  
Work Packages organise the project's activities into logical groups. In version 1, Work Packages are used for labelling and allocation purposes only — they do not affect any cost calculations. Each cost item (personnel role, equipment, trip, C3 item) may be optionally tagged with a WP to support a per-WP budget view.

**Inputs:**

| Field | Source |
|---|---|
| Number of Work Packages | PS-01 — project setup |

**Outputs:**  
- A list of WP labels: WP-1, WP-2, … WP-N.

**Dependencies:**  
- PS-01.

**Conditions:**  
- WP names are auto-generated as "WP-1", "WP-2", etc. The user may optionally assign a descriptive name to each WP (e.g., "WP-3: Fieldwork Phase").
- WP assignment is optional on all cost items. Cost totals are calculated regardless of WP assignment.

**Exceptions:**  
- None. WP data is informational only in v1.

**Validation:**  
- WP count matches the value set in PS-01.

---

## Part 8 — Indirect Costs

---

### IC-01 — Indirect Cost Calculation (Category E)

**Purpose:**  
Calculate the overhead costs charged to the project as a fixed percentage of total direct costs. Under ERC rules, indirect costs (overheads) are set at 25% of direct eligible costs. Indirect costs are not charged on subcontracting (Category B) or on internally invoiced goods (Category D, excluded in v1).

**Inputs:**

| Field | Source |
|---|---|
| Total Personnel Cost (A) | PE-04 |
| Total Travel Cost (C1) | TR-06 |
| Total Equipment Cost (C2) | EQ-03 |
| Total Other Direct Costs (C3) | OC-03 |
| Indirect cost rate (%) | PS-01 — project setup (default 25%) |

**Outputs:**  
- Total Indirect Cost (EUR) — Category E.
- Indirect cost per project year (for the annual budget dashboard).

**Algorithm:**

> Indirect Cost Base = Personnel (A) + Travel (C1) + Equipment (C2) + Other Direct Costs (C3)  
> Total Indirect Costs (E) = Indirect Cost Base × Indirect Cost Rate

Note: Category B (Subcontracting) is explicitly excluded from the base. Category D (Internally Invoiced) is excluded in v1.

**Dependencies:**  
- PE-04, TR-06, EQ-03, OC-03 — all direct cost totals must be known.
- PS-01 (indirect cost rate).

**Conditions:**  
- The indirect cost rate is a single project-level parameter. It applies uniformly to the entire project and every reporting period.
- The application must compute the indirect cost in total AND per project year (using the year-level cost figures from each category) for accurate annual budget display.
- Per-year indirect cost = (Year Y personnel + Year Y travel + Year Y equipment + Year Y C3 from OC-03) × indirect rate.

**Exceptions:**  
- If the indirect cost rate is set to 0%, indirect costs are €0 but the calculation still runs (result is zero).

**Examples:**  
- Year 1 direct costs: Personnel €78,000, Travel €14,000, Equipment €12,750, C3 €30,870 → base = €135,620 → indirect at 25% = **€33,905**.

**Validation:**  
- Indirect costs = Indirect Cost Base × Indirect Cost Rate (exact — no rounding until final display).
- Indirect costs ≥ 0.
- Indirect cost base must exclude Category B and D.

---

## Part 9 — Project Totals

---

### PT-01 — Total Direct Costs

**Purpose:**  
Sum all direct cost categories to produce the Total Direct Costs figure. This is the sum of what the project directly spends before the overhead uplift.

**Inputs:**

| Category | Source |
|---|---|
| A — Personnel | PE-04 |
| B — Subcontracting | SC-01 |
| C1 — Travel | TR-06 |
| C2 — Equipment | EQ-03 |
| C3 — Other Direct Costs | OC-03 |

**Outputs:**  
- Total Direct Costs (EUR).

**Algorithm:**

> Total Direct Costs = Personnel (A) + Subcontracting (B) + Travel (C1) + Equipment (C2) + Other Direct Costs (C3)

**Dependencies:**  
- PE-04, SC-01, TR-06, EQ-03, OC-03.

**Conditions:**  
- Subcontracting is included in Total Direct Costs even though it is excluded from the indirect cost base.

**Exceptions:**  
- None.

**Validation:**  
- Total Direct Costs = A + B + C1 + C2 + C3 (exact arithmetic).
- Must be ≥ 0.

---

### PT-02 — Total Eligible Costs

**Purpose:**  
Add indirect costs to total direct costs to produce the Total Eligible Costs — the full budget that can be submitted to the funding agency.

**Inputs:**

| Field | Source |
|---|---|
| Total Direct Costs | PT-01 |
| Total Indirect Costs (E) | IC-01 |

**Outputs:**  
- Total Eligible Costs (EUR).

**Algorithm:**

> Total Eligible Costs = Total Direct Costs + Indirect Costs (E)

**Dependencies:**  
- PT-01, IC-01.

**Conditions:**  
- None.

**Exceptions:**  
- None.

**Validation:**  
- Total Eligible Costs = Total Direct Costs + Indirect Costs (exact arithmetic).
- Must be > 0 for a non-empty project.

---

### PT-03 — Total Requested EU Contribution

**Purpose:**  
Determine the amount the project is requesting from the European Commission. For ERC Actual Costs grants, the requested EU contribution equals the Total Eligible Costs — the EC funds 100% of eligible costs.

**Inputs:**

| Field | Source |
|---|---|
| Total Eligible Costs | PT-02 |

**Outputs:**  
- Total Requested EU Contribution (EUR).

**Algorithm:**

> Requested EU Contribution = Total Eligible Costs

**Dependencies:**  
- PT-02.

**Conditions:**  
- This rule applies to Actual Costs grants. If a co-funding model were used, a co-funding rate would be applied here. In v1, Actual Costs (100% EU funding) is the only supported model.

**Exceptions:**  
- None.

**Validation:**  
- Requested EU Contribution = Total Eligible Costs (exactly, no deductions).
- Must be > 0.

---

## Appendix A — Rule Dependency Map

```
PS-01 (Project Setup)
   │
   ├── PE-01 (Role Registration — inflation rate required per role)
   │      └── PE-02 (Salary Projection — TRY→EUR + compounding chain)
   │             └── PE-03 (Annual Cost — 12 months × FTE, active years only)
   │                    └── PE-04 ──────────────────────────────────────────┐
   │                                                                        │
   ├── EQ-01 (Equipment Registration)                                       │
   │      └── EQ-02 (Depreciation with cap)                                │
   │             └── EQ-03 ────────────────────────────────────────────── ─┤
   │                                                                        │
   ├── TR-01 (Trip Registration — Itemized or Flat; domestic transport)     │
   │      ├── TR-02 (Flight Cost Lookup — distance band)                    │
   │      ├── TR-03 (Accommodation — rate by country)                       │
   │      └── TR-04 (Subsistence — rate by country)                         │
   │             └── TR-05 (Per-Trip Total — incl. domestic transport)      │
   │                    └── TR-06 (Year-assigned C1 totals) ─────────────  ┤
   │                                                                        │
   ├── OC-01 (C3 Item Registration)                                         │
   │      └── OC-02 (CFS Auto-Trigger — monitors PT-03 vs €430k threshold) │
   │             └── OC-03 (Total C3) ─────────────────────────────────── ─┤
   │                                                                        │
   └── SC-01 (Subcontracting) ─────────────────────────────────────────── ─┤
                                                                            │
                                                           IC-01 (Indirect Costs: 25% of A+C1+C2+C3)
                                                                │
                                                           PT-01 (Total Direct Costs: A+B+C1+C2+C3)
                                                                │
                                                           PT-02 (Total Eligible Costs)
                                                                │
                                                           PT-03 (Requested EU Contribution)
                                                                │
                                                        [feeds back to OC-02 threshold check]
```

---

## Appendix B — EU Travel Rate Versions

The EU Annex 2a/2b rates are versioned and updated periodically. The application must store the rate version relevant to the grant call's opening date and make it visible to the user.

| Version | Applicable period |
|---|---|
| Version before 31 July 2024 | Calls opening before 31 July 2024 |
| Version 31 July 2024 – 12 May 2025 | Calls opening in this period |
| Version from 13 May 2025 | Calls opening from 13 May 2025 onward ← **current version for ERC-CoG** |

Source: EU Grants: Additional Information on Unit Costs and Contributions, Annex 2a and 2b, V1.11, 01.05.2026.

---

## Appendix C — Design Corrections from Excel Workbook

The following issues in the source workbook have been intentionally corrected in the business rules above:

| Issue (from excel-analysis.md) | Correction Applied |
|---|---|
| E-01: String dash `-` in inactive period cells | All inactive periods are numeric zero; no string placeholders. |
| E-02: Austria accommodation at €170 (exceeds EU limit €158) | Rule TR-03 uses the official EU rate (€158) for Austria. |
| E-03: Travel averaged equally across all years | Rule TR-06 assigns travel costs to specific years based on trip registration. |
| DUP-01: Indirect cost rate hardcoded in 7 places | IC-01 uses a single configurable indirect cost rate from PS-01. |
| DUP-02: FTE fraction stored in three places | PE-01 is the single source of truth for FTE; all downstream rules read from it. |
| DUP-04: Publications cost duplicated without link | OC-01 is the single source of truth for all C3 items; no parallel entry in a different table. |
| E-08: Overhead base in Details includes Category D | IC-01 explicitly excludes Category D from the overhead base (aligned with ERC rules). |

---

## Summary

| Group | Rules | Total |
|---|---|---|
| Project Setup | PS-01 | 1 |
| Personnel | PE-01, PE-02, PE-03, PE-04 | 4 |
| Equipment | EQ-01, EQ-02, EQ-03 | 3 |
| Travel | TR-01, TR-02, TR-03, TR-04, TR-05, TR-06 | 6 |
| Other Direct Costs | OC-01, OC-02 (CFS trigger), OC-03 | 3 |
| Subcontracting | SC-01 | 1 |
| Work Packages | WP-01 | 1 |
| Indirect Costs | IC-01 | 1 |
| Project Totals | PT-01, PT-02, PT-03 | 3 |
| **Total** | | **23 rules** |

---

## Resolved Questions

All open questions from the initial draft have been answered and incorporated into the rules above.

| ID | Question | Resolution | Rule(s) Updated |
|---|---|---|---|
| OQ-01 | Can staff start or leave mid-year? | No. All staff work full months in each active year. Active months = 12 for active years, 0 otherwise. No partial-year entry. | PE-03 |
| OQ-02 | Single vs. per-role inflation rate? | Per-role inflation rate is required. The project-level default (PS-01) is pre-filled but must be confirmed or overridden per role. | PE-01, PE-02 |
| OQ-03 | Domestic transport within destination country? | Yes. A flat domestic transport amount (user-entered, per trip instance) is added as an optional field in TR-01 and included in the TR-05 per-trip sum. | TR-01, TR-05 |
| OQ-04 | Conference costs as flat amounts? | Yes. TR-01 now supports a "Flat amount" trip type where the user enters the total cost per instance directly, bypassing the EU rate lookups. | TR-01, TR-05 |
| OQ-05 | CFS auto-trigger? | Yes. When total budget (PT-03) exceeds €430,000, the app automatically prompts the user to enter a CFS fee as a flat amount. The CFS is added to C3 and included in all downstream totals. | OC-02 (new rule) |

---

**Confidence Level: 98%**

All business rules are now fully derived from verified workbook formulas, user-confirmed design decisions, and PI-resolved open questions. Residual 2% uncertainty: EU travel rate table version applicability (assumed "from 13 May 2025" applies to this ERC-CoG call — to be confirmed against the actual call opening date before implementation).

**Recommended Next Step:**  
Await PI approval. Once approved, proceed to TASK-04 (Domain Model).
