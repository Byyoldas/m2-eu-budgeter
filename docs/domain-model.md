# Domain Model

**Document:** TASK-04 Deliverable  
**Date:** 2026-07-10  
**Status:** Draft — Awaiting Approval Before Proceeding to TASK-05  
**Source documents:** business-rules.md, excel-analysis.md, project-overview.md

---

> ## ⚠ Current Implementation Notes (as of v1.6.0, 2026-07-17)
>
> The entities below describe the pre-implementation design. The actual Rust structs (`src-tauri/src/domain/entities.rs`) have diverged in these ways:
>
> - **`WorkPackage` (§2.2)** gained `start_month`/`end_month` (1-indexed project months) — it's no longer just an `id`/`name` label. See `docs/developer-guide.md` §8.
> - **`PersonnelRole` (§2.3)**: `active_years: Vec<u8>` was replaced with `start_month: u32, end_month: u32`. The `work_package_ids` field described for this entity no longer exists — a role's WP allocation is now computed automatically (`allocate_personnel_cost_by_wp`), not stored as a direct field.
> - **`EquipmentItem` (§2.4)**: any `project_year`/`year_of_purchase`-style field no longer exists; `work_package_id: u8` (single, required) replaced whatever WP field this section originally described.
> - **`Trip` (§2.5)** and **`OtherDirectCostItem` (§2.6)**: any `project_year` field no longer exists; both now carry `work_package_ids: Vec<u8>` (non-empty, required — cost splits evenly across multiple WPs for the per-WP budget view).
> - **`Subcontracting` (§2.7)**: gained `work_package_id: u8` (required) and is a real, non-placeholder amount — see the note in `business-rules.md`.
> - **`BudgetSummary` (§3.5)**: gained `wp_budgets: Vec<WpBudgetDto>` (one entry per Work Package, summing all five categories) — there is no equivalent "per-year breakdown" entity anywhere in the current implementation; anywhere this document implies year-indexed output arrays for Equipment/Travel/OtherCosts, read it as WP-indexed instead.
> - **`EUTravelRateVersion`/`Country`/`FlightDistanceBand` (Part 1)**: structurally accurate, but the actual bundled data was fabricated placeholder data until v1.4.0, when it was replaced with the real EU Annex 2a/2b figures for all three rate-version tiers.
>
> The current, authoritative field-level definitions live in `src-tauri/src/domain/entities.rs` and `src-tauri/src/domain/dto.rs` directly — treat this document as historical design intent, not a current reference.

---

## How to Read This Document

This document describes every meaningful concept (entity) in the HE Budget application domain. Entities are the things the software creates, stores, computes, and displays. Each entity is described with:

- **Purpose** — what it represents and why it exists
- **Attributes** — the data fields it holds, with types and whether they are user-supplied or computed
- **Relationships** — how it connects to other entities
- **Constraints** — rules that must always be true about this entity
- **Validation Rules** — checks that must pass before the entity is accepted as valid

Entities are grouped into three layers:

- **User-Configured Entities** — created and edited directly by the user
- **Computed Entities** — derived by the calculation engine; never edited directly
- **Reference Data Entities** — built into the application; read-only at runtime

---

## Entity Overview

```
REFERENCE DATA
  EUTravelRateVersion
       ├── Country  (accommodation + subsistence rates)
       └── FlightDistanceBand  (per-distance-band flight cost)

USER-CONFIGURED
  Project
     ├── WorkPackage  (1..10)
     ├── PersonnelRole  (1..N)
     ├── EquipmentItem  (0..N)
     ├── Trip  (0..N)
     ├── OtherDirectCostItem  (0..N)  ← includes CFS when auto-triggered
     └── Subcontracting  (exactly 1, default €0)

COMPUTED
  Project
     ├── PersonnelRole → SalaryProjection  (1 per year)
     │                 → PersonnelCostLine  (1 per year)
     ├── EquipmentItem → EquipmentDepreciation  (1 per item)
     ├── Trip → TripCost  (1 per trip)
     └── BudgetSummary  (1 per project — live aggregate)
```

---

## Part 1 — Reference Data Entities

---

### 1.1 EUTravelRateVersion

**Purpose:**  
Represents a specific published version of the EU Grants Annex 2a/2b travel unit cost table. Rate versions are tied to call opening date ranges. The application bundles all known versions and selects the applicable one based on the grant call opening date stored in the Project.

**Attributes:**

| Attribute | Type | Description |
|---|---|---|
| versionId | String | Unique identifier (e.g., `V1.11-2025-05-13`) |
| label | String | Human-readable label (e.g., "Annex 2a/2b V1.11 — from 13 May 2025") |
| effectiveFromDate | Date | The earliest call opening date to which this version applies |
| effectiveToDate | Date \| null | The last call opening date; null if this is the current version |
| sourceReference | String | Citation (e.g., "EU Grants Annex 2a/2b V1.11, 01.05.2026") |

**Relationships:**  
- One EUTravelRateVersion → many Country rate rows (1 per country per version)
- One EUTravelRateVersion → many FlightDistanceBand rows (1 per band per version)

**Constraints:**  
- Version date ranges must not overlap.
- At any given call opening date, exactly one version must be applicable.
- The currently bundled versions span: before 31 July 2024; 31 July 2024 – 12 May 2025; from 13 May 2025.

**Validation Rules:**  
- `effectiveFromDate` must be a valid date.
- If `effectiveToDate` is not null, it must be strictly after `effectiveFromDate`.
- Each version must have at least one associated Country and at least one FlightDistanceBand.

---

### 1.2 Country

**Purpose:**  
A destination country for which the EU specifies official accommodation and daily subsistence (per diem) unit costs. These rates form the upper eligible limit for travel claims. The application looks up a Country record whenever the user selects a destination on a trip.

**Attributes:**

| Attribute | Type | Description |
|---|---|---|
| countryCode | String (ISO 3166-1 alpha-2) | Standard two-letter country code (e.g., `TR`, `IN`, `AU`) |
| countryName | String | Display name (e.g., "Turkey", "India", "Australia") |
| accommodationRateEUR | Decimal | Maximum eligible accommodation cost per night (€) |
| subsistenceRateEUR | Decimal | Maximum eligible daily subsistence allowance (€) |
| versionId | String → EUTravelRateVersion | The rate version this row belongs to |

**Relationships:**  
- Many Country rows → one EUTravelRateVersion
- One Country ← many Trips (each Itemized trip references one Country)

**Constraints:**  
- Each (countryCode, versionId) pair must be unique — one rate set per country per version.
- Both rates must be positive numbers.

**Validation Rules:**  
- `countryCode` must be a valid ISO 3166-1 alpha-2 code.
- `accommodationRateEUR` > 0.
- `subsistenceRateEUR` > 0.
- `versionId` must reference an existing EUTravelRateVersion.

**Key reference values (version: from 13 May 2025):**

| Country | Accommodation (€/night) | Subsistence (€/day) |
|---|---|---|
| Australia | 135 | 75 |
| Austria | 158 | 131 |
| France | 212 | 127 |
| India | 195 | 50 |
| Spain | 154 | 101 |
| Turkey | 165 | 55 |
| United Kingdom | 209 | 125 |
| United States | 200 | 80 |

---

### 1.3 FlightDistanceBand

**Purpose:**  
Maps a range of one-way flight distances (in kilometres) to an EU official unit cost per round trip. The application selects the correct band automatically from the distance entered by the user on a trip.

**Attributes:**

| Attribute | Type | Description |
|---|---|---|
| bandId | String | Unique identifier per band per version (e.g., `V1.11-band-4501-6000`) |
| minKm | Integer | Lower bound of the distance range (inclusive) |
| maxKm | Integer \| null | Upper bound (inclusive); null for the open-ended top band (10,001+ km) |
| flightUnitCostEUR | Decimal | EU official unit cost per round trip at this distance band (€) |
| versionId | String → EUTravelRateVersion | The rate version this band belongs to |

**Relationships:**  
- Many FlightDistanceBand rows → one EUTravelRateVersion
- One FlightDistanceBand ← many Trips (each Itemized trip with distance ≥ 400 km maps to one band)

**Constraints:**  
- Within a version, bands must together cover all distances from 400 km upward without gaps or overlaps.
- The first band starts at 400 km (distances below 400 km are out of scope for the flight rate system).
- `flightUnitCostEUR` must be > 0.

**Validation Rules:**  
- `minKm` ≥ 400.
- If `maxKm` is not null, `maxKm` > `minKm`.
- `flightUnitCostEUR` > 0.

**Rate table (version: from 13 May 2025):**

| Band | min km | max km | Unit cost (€/trip) |
|---|---|---|---|
| 1 | 400 | 600 | 340 |
| 2 | 601 | 1,600 | 365 |
| 3 | 1,601 | 2,500 | 429 |
| 4 | 2,501 | 3,500 | 541 |
| 5 | 3,501 | 4,500 | 743 |
| 6 | 4,501 | 6,000 | 857 |
| 7 | 6,001 | 7,500 | 1,021 |
| 8 | 7,501 | 10,000 | 1,250 |
| 9 | 10,001 | ∞ | 1,595 |

---

## Part 2 — User-Configured Entities

---

### 2.1 Project

**Purpose:**  
The root entity. Every other entity in the system belongs to a Project. A Project represents a single ERC grant application and holds the project-level structural and financial parameters that govern all calculations.

**Attributes:**

| Attribute | Type | Required | Source | Description |
|---|---|---|---|---|
| projectId | UUID | Auto | System | Unique identifier |
| title | String | No | User | Project title (display only) |
| piName | String | No | User | Principal Investigator name (display only) |
| callReference | String | No | User | Grant call code (e.g., "ERC-CoG") |
| durationYears | Integer | Yes | User | Number of full project years (1–7) |
| numberOfWorkPackages | Integer | Yes | User | Number of Work Packages (1–10) |
| defaultInflationRate | Decimal | Yes | User | Default annual salary inflation rate (%) applied to all roles unless overridden |
| tryEurExchangeRate | Decimal | Yes | User | TRY to EUR conversion rate applied uniformly for the project lifetime |
| indirectCostRate | Decimal | Yes | User | Overhead rate applied to direct costs (default 25%) |
| applicableRateVersionId | String | Yes | User/System | EUTravelRateVersion to use for all travel rate lookups; selected based on the grant call opening date |
| callOpeningDate | Date | No | User | Used to auto-select the applicable EU travel rate version |
| createdAt | DateTime | Auto | System | When the project record was created |
| updatedAt | DateTime | Auto | System | Last modification timestamp |

**Relationships:**  
- One Project → many WorkPackages (1..10)
- One Project → many PersonnelRoles (0..N)
- One Project → many EquipmentItems (0..N)
- One Project → many Trips (0..N)
- One Project → many OtherDirectCostItems (0..N)
- One Project → exactly one Subcontracting record
- One Project → one BudgetSummary (computed)
- One Project → one EUTravelRateVersion (via `applicableRateVersionId`)

**Constraints:**  
- A project must have at least one year and at least one Work Package before any cost entry is permitted.
- `tryEurExchangeRate` and `defaultInflationRate` must be set before PersonnelRoles can be created.
- Only one project exists in v1 of the application (single-project mode).

**Validation Rules:**  
- `durationYears` ≥ 1 and ≤ 7.
- `numberOfWorkPackages` ≥ 1 and ≤ 10.
- `defaultInflationRate` ≥ 0 and ≤ 100 (percent).
- `tryEurExchangeRate` > 0.
- `indirectCostRate` ≥ 0 and ≤ 50 (percent). If ≠ 25, display a deviation warning.

---

### 2.2 WorkPackage

**Purpose:**  
Organises the project's activities into logical groups. In version 1, Work Packages are for labelling and reporting only — they are not connected to cost calculations. Each cost item may optionally be tagged with a Work Package.

**Attributes:**

| Attribute | Type | Required | Source | Description |
|---|---|---|---|---|
| workPackageId | UUID | Auto | System | Unique identifier |
| projectId | UUID | Yes | System | Parent project |
| number | Integer | Auto | System | Sequential number (1, 2, … N) |
| name | String | No | User | Optional descriptive name (e.g., "WP-2: Fieldwork Phase") |

**Relationships:**  
- Many WorkPackages → one Project
- One WorkPackage ← many PersonnelRoles (optional tagging)
- One WorkPackage ← many EquipmentItems (optional tagging)
- One WorkPackage ← many Trips (optional tagging)
- One WorkPackage ← many OtherDirectCostItems (optional tagging)

**Constraints:**  
- WP count must equal `Project.numberOfWorkPackages`.
- WP numbers are auto-generated (1..N) and immutable.
- WP names are optional and have no effect on calculations.

**Validation Rules:**  
- `number` ≥ 1 and ≤ `Project.numberOfWorkPackages`.
- No two WPs in the same project may share the same number.

---

### 2.3 PersonnelRole

**Purpose:**  
Represents one staff position charged to the grant. Each role has a generic label (e.g., PostDoc-1), employment parameters, and a salary basis in TRY. The calculation engine derives all year-by-year EUR costs from this entity.

**Attributes:**

| Attribute | Type | Required | Source | Description |
|---|---|---|---|---|
| roleId | UUID | Auto | System | Unique identifier |
| projectId | UUID | Yes | System | Parent project |
| roleType | Enum | Yes | User | Category: `PI`, `Expert`, `PostDoc`, `Admin` |
| roleLabel | String | Yes | User | Unique display name: `PI`, `Expert-1`, `Expert-2`, `PostDoc-1`, … `Admin-1`, etc. |
| currentMonthlySalaryTRY | Decimal | Yes | User | Gross monthly salary today, in Turkish Lira. The base for all salary projections. |
| fteFraction | Decimal | Yes | User | Fraction of working time dedicated to the grant (0.0–1.0). |
| inflationRate | Decimal | Yes | User | Annual salary increase for this role (%), pre-filled with `Project.defaultInflationRate`. Must be confirmed per role. |
| activeYears | Integer[] | Yes | User | List of project year numbers in which this role is active and charged (e.g., [1, 2, 3]). |
| workPackageIds | Integer[] | No | User | WP numbers this role contributes to (informational only). |

**Relationships:**  
- Many PersonnelRoles → one Project
- One PersonnelRole → many SalaryProjections (one per project year — computed)
- One PersonnelRole → many PersonnelCostLines (one per project year — computed)
- One PersonnelRole → many WorkPackages (optional, via `workPackageIds`)

**Constraints:**  
- Only one role with `roleType = PI` may exist per project.
- `roleLabel` must be unique within a project.
- `activeYears` values must all be valid project year numbers (1 to `Project.durationYears`).
- `fteFraction` must be > 0 and ≤ 1.

**Validation Rules:**  
- `currentMonthlySalaryTRY` > 0.
- `fteFraction` > 0.0 and ≤ 1.0.
- `inflationRate` ≥ 0 and ≤ 100 (percent).
- `activeYears` must not be empty.
- All values in `activeYears` must be integers between 1 and `Project.durationYears` inclusive.
- No duplicate values in `activeYears`.
- `roleLabel` must be unique within the project.

---

### 2.4 EquipmentItem

**Purpose:**  
Represents one piece of equipment purchased during the project. The calculation engine applies the ERC depreciation formula to produce an eligible cost. Multiple items of the same type are registered separately.

**Attributes:**

| Attribute | Type | Required | Source | Description |
|---|---|---|---|---|
| equipmentItemId | UUID | Auto | System | Unique identifier |
| projectId | UUID | Yes | System | Parent project |
| name | String | Yes | User | Descriptive label (e.g., "Laptop – PI", "Audio Recorder 1") |
| purchaseCostEUR | Decimal | Yes | User | Total purchase price in EUR, including import duties if applicable |
| usefulLifetimeMonths | Integer | Yes | User | Standard economic lifetime of this equipment type in months |
| grantUsagePct | Decimal | Yes | User | Proportion of the item's use dedicated to grant activities (0–100%) |
| grantUsageMonths | Integer | Yes | User | Number of months the item is in active use during the grant period |
| yearOfPurchase | Integer | No | User | Project year in which the item is purchased (1 to N); informational |
| workPackageIds | Integer[] | No | User | WP numbers supported by this item (informational only) |

**Relationships:**  
- Many EquipmentItems → one Project
- One EquipmentItem → one EquipmentDepreciation (computed)
- One EquipmentItem → many WorkPackages (optional, via `workPackageIds`)

**Constraints:**  
- `grantUsageMonths` should not exceed the project duration in months. If it does, the application warns the user but does not block entry (late-purchased equipment may still be used post-project in practice).
- `grantUsagePct` must be > 0 (items with 0% grant usage are not eligible and should not be registered).

**Validation Rules:**  
- `purchaseCostEUR` > 0.
- `usefulLifetimeMonths` ≥ 1.
- `grantUsagePct` > 0 and ≤ 100.
- `grantUsageMonths` ≥ 1.
- If `yearOfPurchase` is provided, it must be between 1 and `Project.durationYears`.

---

### 2.5 Trip

**Purpose:**  
Represents one planned trip or group of identical trips. A Trip is either Itemized (costs are built from EU unit rates for flight, accommodation, subsistence, and user-entered domestic transport) or Flat amount (the user enters a single total cost per trip instance). Flat trips are used for conferences where the detailed itinerary is not yet known.

**Attributes:**

| Attribute | Type | Required | Source | Description |
|---|---|---|---|---|
| tripId | UUID | Auto | System | Unique identifier |
| projectId | UUID | Yes | System | Parent project |
| name | String | Yes | User | Descriptive label (e.g., "Fieldwork – India – Year 1") |
| tripType | Enum | Yes | User | `Itemized` or `FlatAmount` |
| projectYear | Integer | Yes | User | Project year in which the trip(s) occur (1 to N) |
| numberOfInstances | Integer | Yes | User | Number of times this trip occurs in the specified year |
| workPackageId | Integer | No | User | WP number this travel supports (informational only) |
| — Itemized fields — | | | | |
| destinationCountryCode | String | Yes (Itemized) | User | ISO country code; used to look up Country rates |
| oneWayDistanceKm | Integer | Yes (Itemized) | User | One-way flight distance in km; 0 if no flight |
| numberOfNights | Integer | Yes (Itemized) | User | Nights accommodation per trip instance |
| numberOfDays | Integer | Yes (Itemized) | User | Days of subsistence per trip instance |
| domesticTransportCostPerInstanceEUR | Decimal | No (Itemized) | User | Flat amount for in-country transport per instance (default 0) |
| — Flat amount fields — | | | | |
| flatAmountPerInstanceEUR | Decimal | Yes (FlatAmount) | User | Total cost per trip instance as entered by the user |

**Relationships:**  
- Many Trips → one Project
- One Trip → one TripCost (computed)
- One Trip → one Country (Itemized only; via `destinationCountryCode`)
- One Trip → one FlightDistanceBand (Itemized only; looked up from `oneWayDistanceKm`)
- One Trip → one WorkPackage (optional, via `workPackageId`)

**Constraints:**  
- `projectYear` must be a valid year within the project (1 to `Project.durationYears`).
- For Itemized trips: `destinationCountryCode` must exist in the Country reference table.
- For Itemized trips with `oneWayDistanceKm` ≥ 400: a FlightDistanceBand must be found.
- For Itemized trips with `oneWayDistanceKm` < 400 or = 0: no flight cost is applied; the application displays an informational note.
- For FlatAmount trips: `destinationCountryCode` and distance fields are not required and are ignored in calculations.
- `numberOfInstances` ≥ 1.

**Validation Rules:**  
- `tripType` must be one of `Itemized`, `FlatAmount`.
- `projectYear` between 1 and `Project.durationYears`.
- `numberOfInstances` ≥ 1.
- Itemized: `numberOfNights` ≥ 1; `numberOfDays` ≥ 1; `oneWayDistanceKm` ≥ 0.
- Itemized: `destinationCountryCode` must match a Country record in the active EUTravelRateVersion.
- Itemized: `domesticTransportCostPerInstanceEUR` ≥ 0 (optional; defaults to 0).
- FlatAmount: `flatAmountPerInstanceEUR` > 0.

---

### 2.6 OtherDirectCostItem

**Purpose:**  
Represents a single C3 "Other Goods, Works and Services" cost item charged to the project in a specific year. Used for software licences, publications, translation services, fieldwork costs, the Certificate on Financial Statements (CFS), and any other direct costs that do not belong to Personnel, Equipment, or Travel. One special sub-type (CFS) may be created automatically by the application when the budget threshold is crossed.

**Attributes:**

| Attribute | Type | Required | Source | Description |
|---|---|---|---|---|
| itemId | UUID | Auto | System | Unique identifier |
| projectId | UUID | Yes | System | Parent project |
| name | String | Yes | User/System | Description of the cost (e.g., "MAXQDA software licence"). Auto-set to "Certificate on Financial Statements (CFS)" for auto-triggered CFS items. |
| amountEUR | Decimal | Yes | User | Cost in EUR for this item in the specified year |
| projectYear | Integer | Yes | User | Project year in which this cost is incurred |
| isCFSItem | Boolean | Auto | System | True if this item was created by the CFS auto-trigger (OC-02); false otherwise. Prevents duplicate CFS entries. |
| notes | String | No | User | Optional justification or notes |
| workPackageId | Integer | No | User | WP this cost is associated with (informational only) |

**Relationships:**  
- Many OtherDirectCostItems → one Project
- One OtherDirectCostItem → one WorkPackage (optional, via `workPackageId`)

**Constraints:**  
- At most one OtherDirectCostItem with `isCFSItem = true` may exist per project.
- `projectYear` must be within the valid range for the project.
- `amountEUR` must be > 0.

**Validation Rules:**  
- `amountEUR` > 0.
- `projectYear` between 1 and `Project.durationYears`.
- `name` must not be blank.
- `isCFSItem = true` is set only by the system (auto-trigger); users cannot set it manually.

---

### 2.7 Subcontracting

**Purpose:**  
Represents the Category B subcontracting budget line. In version 1, this is always €0 (no subcontracting is planned). The entity exists as a placeholder so the data model is complete and the calculation chain is unbroken. A future version may support detailed subcontracting entries.

**Attributes:**

| Attribute | Type | Required | Source | Description |
|---|---|---|---|---|
| subcontractingId | UUID | Auto | System | Unique identifier |
| projectId | UUID | Yes | System | Parent project |
| amountEUR | Decimal | Yes | User | Total subcontracting budget. Default = 0. |

**Relationships:**  
- One Subcontracting → one Project (exactly one per project)

**Constraints:**  
- Exactly one Subcontracting record exists per project; it is created automatically when the project is initialised.
- `amountEUR` ≥ 0.

**Validation Rules:**  
- `amountEUR` ≥ 0.

---

## Part 3 — Computed Entities

---

### 3.1 SalaryProjection

**Purpose:**  
The year-by-year projected monthly salary in EUR for a PersonnelRole. Computed by the calculation engine from the role's TRY base salary, the TRY/EUR exchange rate, and the annual inflation rate. Never entered or edited by the user. Recalculated automatically whenever any input it depends on changes.

**Attributes:**

| Attribute | Type | Source | Description |
|---|---|---|---|
| projectionId | UUID | System | Unique identifier |
| roleId | UUID | System | Parent PersonnelRole |
| yearNumber | Integer | System | Project year (1 to N) |
| projectedMonthlyEUR | Decimal | Computed | Projected monthly gross salary in EUR for this year |

**Computation:**

```
eurBase = PersonnelRole.currentMonthlySalaryTRY ÷ Project.tryEurExchangeRate

Year 1:  projectedMonthlyEUR = eurBase × (1 + inflationRate / 100)
Year Y:  projectedMonthlyEUR = Year(Y-1).projectedMonthlyEUR × (1 + inflationRate / 100)
```

The inflation compounds year-on-year from the TRY base converted to EUR. The chain starts at Year 1 (the base already includes one year of inflation).

**Relationships:**  
- Many SalaryProjections → one PersonnelRole
- One SalaryProjection per (roleId, yearNumber) pair

**Constraints:**  
- One SalaryProjection row must exist for every (role, year) combination — even for years when the role is not active. Inactive-year projections are calculated but not used in cost lines.
- `projectedMonthlyEUR` must always be > 0.
- Year N projection must be ≥ Year N-1 projection when inflation rate ≥ 0%.

**Validation Rules:**  
- `projectedMonthlyEUR` > 0.
- Ordering: `projection[year Y].projectedMonthlyEUR` ≥ `projection[year Y-1].projectedMonthlyEUR` when `inflationRate` ≥ 0%.

---

### 3.2 PersonnelCostLine

**Purpose:**  
The annual eligible personnel cost for a specific role in a specific project year. Computed from the projected monthly salary, the FTE fraction, and whether the role is active in that year. This is the finest-grained personnel cost unit — all higher-level personnel totals are aggregations of PersonnelCostLines.

**Attributes:**

| Attribute | Type | Source | Description |
|---|---|---|---|
| costLineId | UUID | System | Unique identifier |
| roleId | UUID | System | Parent PersonnelRole |
| yearNumber | Integer | System | Project year |
| projectedMonthlyEUR | Decimal | From SalaryProjection | Monthly salary for this year |
| fteFraction | Decimal | From PersonnelRole | Grant-dedicated fraction of working time |
| isActive | Boolean | Computed | True if yearNumber is in PersonnelRole.activeYears |
| activeMonths | Integer | Computed | 12 if isActive = true; 0 otherwise |
| annualCostEUR | Decimal | Computed | Personnel cost charged to the grant for this role in this year |

**Computation:**

```
isActive   = (yearNumber ∈ PersonnelRole.activeYears)
activeMonths = 12 if isActive else 0
annualCostEUR = projectedMonthlyEUR × activeMonths × fteFraction
```

**Relationships:**  
- Many PersonnelCostLines → one PersonnelRole
- One PersonnelCostLine depends on one SalaryProjection (same role, same year)

**Constraints:**  
- One PersonnelCostLine must exist for every (role, year) combination.
- `annualCostEUR` = 0 for inactive years.
- `annualCostEUR` > 0 for active years (since salary and FTE are both > 0).

**Validation Rules:**  
- `activeMonths` ∈ {0, 12} only.
- `annualCostEUR` = `projectedMonthlyEUR` × `activeMonths` × `fteFraction` (exact).
- `annualCostEUR` ≥ 0.

---

### 3.3 EquipmentDepreciation

**Purpose:**  
The calculated eligible depreciation amount for one EquipmentItem. Applies the ERC two-step formula: compute proportional depreciation, then cap it at the full grant-attributable cost if the usage period exceeds the economic lifetime. One EquipmentDepreciation record exists per EquipmentItem.

**Attributes:**

| Attribute | Type | Source | Description |
|---|---|---|---|
| depreciationId | UUID | System | Unique identifier |
| equipmentItemId | UUID | System | Parent EquipmentItem |
| theoreticalEligibleEUR | Decimal | Computed | Uncapped depreciation: (cost ÷ lifetime) × usage% × months |
| maximumEligibleEUR | Decimal | Computed | Cap: cost × usage% |
| isCapped | Boolean | Computed | True if theoretical ≥ maximum (cap is applied) |
| eligibleDepreciationEUR | Decimal | Computed | Final eligible amount: min(theoretical, maximum) |

**Computation:**

```
theoreticalEligibleEUR = (purchaseCostEUR ÷ usefulLifetimeMonths)
                         × (grantUsagePct / 100)
                         × grantUsageMonths

maximumEligibleEUR     = purchaseCostEUR × (grantUsagePct / 100)

isCapped               = theoreticalEligibleEUR ≥ maximumEligibleEUR

eligibleDepreciationEUR = min(theoreticalEligibleEUR, maximumEligibleEUR)
```

**Relationships:**  
- One EquipmentDepreciation → one EquipmentItem

**Constraints:**  
- `eligibleDepreciationEUR` must never exceed `maximumEligibleEUR`.
- `eligibleDepreciationEUR` must never exceed `EquipmentItem.purchaseCostEUR`.
- `eligibleDepreciationEUR` must be > 0 (since all inputs are > 0).

**Validation Rules:**  
- `eligibleDepreciationEUR` = `min(theoreticalEligibleEUR, maximumEligibleEUR)` (exact).
- `eligibleDepreciationEUR` ≤ `maximumEligibleEUR`.
- `eligibleDepreciationEUR` ≤ `purchaseCostEUR`.
- `eligibleDepreciationEUR` > 0.

---

### 3.4 TripCost

**Purpose:**  
The computed eligible cost for a Trip, broken down by component (flight, accommodation, subsistence, domestic transport) and aggregated across all instances. One TripCost record exists per Trip.

**Attributes:**

| Attribute | Type | Source | Description |
|---|---|---|---|
| tripCostId | UUID | System | Unique identifier |
| tripId | UUID | System | Parent Trip |
| flightCostPerInstanceEUR | Decimal | Computed (Itemized) / 0 (FlatAmount) | EU band rate for the one-way distance; 0 if distance < 400 km or trip is FlatAmount |
| accommodationCostPerInstanceEUR | Decimal | Computed (Itemized) / 0 (FlatAmount) | Country rate × nights |
| subsistenceCostPerInstanceEUR | Decimal | Computed (Itemized) / 0 (FlatAmount) | Country rate × days |
| domesticTransportPerInstanceEUR | Decimal | User (Itemized) / 0 (FlatAmount) | As entered by user; 0 if not applicable |
| perInstanceTotalEUR | Decimal | Computed | Sum of all per-instance components (or flat amount for FlatAmount trips) |
| totalTripCostEUR | Decimal | Computed | `perInstanceTotalEUR` × `numberOfInstances` |

**Computation — Itemized:**

```
flightCostPerInstanceEUR      = FlightDistanceBand.flightUnitCostEUR
                                (or 0 if oneWayDistanceKm < 400)
accommodationCostPerInstanceEUR = Country.accommodationRateEUR × numberOfNights
subsistenceCostPerInstanceEUR  = Country.subsistenceRateEUR × numberOfDays
domesticTransportPerInstanceEUR = Trip.domesticTransportCostPerInstanceEUR (user-entered)

perInstanceTotalEUR = flight + accommodation + subsistence + domestic
totalTripCostEUR    = perInstanceTotalEUR × numberOfInstances
```

**Computation — FlatAmount:**

```
perInstanceTotalEUR = Trip.flatAmountPerInstanceEUR
totalTripCostEUR    = perInstanceTotalEUR × numberOfInstances

(all component breakdowns are 0 or null)
```

**Relationships:**  
- One TripCost → one Trip
- One TripCost references one Country (Itemized only)
- One TripCost references one FlightDistanceBand (Itemized, distance ≥ 400 km only)

**Constraints:**  
- For Itemized trips, the sum of components must equal `perInstanceTotalEUR`.
- `totalTripCostEUR` = `perInstanceTotalEUR` × `Trip.numberOfInstances` (exact).
- `totalTripCostEUR` ≥ 0 (all components are non-negative).

**Validation Rules:**  
- `perInstanceTotalEUR` ≥ 0.
- `totalTripCostEUR` = `perInstanceTotalEUR` × `numberOfInstances` (exact).
- For Itemized trips: individual components are non-negative and sum to `perInstanceTotalEUR`.
- For FlatAmount trips: component breakdown fields are all 0 (or null); `perInstanceTotalEUR` = `flatAmountPerInstanceEUR`.

---

### 3.5 BudgetSummary

**Purpose:**  
A live, continuously recomputed aggregation of all cost categories — both per year and as a project total. This is the central output entity that drives the budget dashboard and the final submission table. It is never stored as a static record; it is always re-derived from the current state of all other entities.

**Attributes:**

| Attribute | Type | Source | Description |
|---|---|---|---|
| projectId | UUID | System | Parent project |
| — Per-year breakdown — | | | (repeated for each Year Y = 1 to N) |
| personnelCostByYear[Y] | Decimal | Computed | Sum of PersonnelCostLine.annualCostEUR for all roles in Year Y |
| travelCostByYear[Y] | Decimal | Computed | Sum of TripCost.totalTripCostEUR for all trips in Year Y |
| equipmentCostByYear[Y] | Decimal | Computed | Sum of EquipmentDepreciation.eligibleDepreciationEUR (equipment cost is not year-assigned in the current model; allocated proportionally or as a single total — see note) |
| otherDirectCostByYear[Y] | Decimal | Computed | Sum of OtherDirectCostItem.amountEUR for items in Year Y |
| subcontractingByYear[Y] | Decimal | Computed | Subcontracting.amountEUR ÷ durationYears (uniform spread; or full amount if registered per year in future versions) |
| directCostByYear[Y] | Decimal | Computed | Sum of the five per-year category figures |
| indirectCostByYear[Y] | Decimal | Computed | (personnel + travel + equipment + otherDirect) × indirectCostRate ÷ 100 |
| totalByYear[Y] | Decimal | Computed | directCostByYear[Y] + indirectCostByYear[Y] |
| — Project totals — | | | |
| totalPersonnelCostEUR | Decimal | Computed | Sum of all PersonnelCostLine.annualCostEUR |
| totalTravelCostEUR | Decimal | Computed | Sum of all TripCost.totalTripCostEUR |
| totalEquipmentCostEUR | Decimal | Computed | Sum of all EquipmentDepreciation.eligibleDepreciationEUR |
| totalOtherDirectCostEUR | Decimal | Computed | Sum of all OtherDirectCostItem.amountEUR |
| totalSubcontractingEUR | Decimal | Computed | Subcontracting.amountEUR |
| totalDirectCostsEUR | Decimal | Computed | Sum of the five category totals |
| indirectCostBaseEUR | Decimal | Computed | Personnel + Travel + Equipment + OtherDirect (excludes Subcontracting) |
| totalIndirectCostsEUR | Decimal | Computed | indirectCostBaseEUR × indirectCostRate ÷ 100 |
| totalEligibleCostsEUR | Decimal | Computed | totalDirectCostsEUR + totalIndirectCostsEUR |
| requestedEUContributionEUR | Decimal | Computed | = totalEligibleCostsEUR (100% EU funding for Actual Costs grant) |
| cfsThresholdExceeded | Boolean | Computed | True if requestedEUContributionEUR > 430,000 |
| cfsItemPresent | Boolean | Computed | True if any OtherDirectCostItem with isCFSItem = true exists |
| cfsWarningActive | Boolean | Computed | True if cfsThresholdExceeded = true AND cfsItemPresent = false |

> **Note on equipment year allocation:** Equipment depreciation in v1 is computed as a single total per item (not split across years). For the per-year budget breakdown, equipment cost is shown as a project-level line. A future version may support year-of-purchase-based allocation.

**Relationships:**  
- One BudgetSummary → one Project
- BudgetSummary aggregates: PersonnelCostLines, TripCosts, EquipmentDepreciations, OtherDirectCostItems, Subcontracting

**Constraints:**  
- `requestedEUContributionEUR` = `totalEligibleCostsEUR` (no deduction for Actual Costs grants).
- `totalIndirectCostsEUR` base must exclude Subcontracting and any Category D items.
- `totalDirectCostsEUR` = A + B + C1 + C2 + C3 (exact arithmetic — no rounding until display).
- `cfsWarningActive` triggers a persistent UI warning badge.

**Validation Rules:**  
- All totals must be ≥ 0.
- `totalEligibleCostsEUR` = `totalDirectCostsEUR` + `totalIndirectCostsEUR` (exact).
- `requestedEUContributionEUR` = `totalEligibleCostsEUR` (exact).
- `totalIndirectCostsEUR` = `indirectCostBaseEUR` × `Project.indirectCostRate` ÷ 100 (exact).
- `indirectCostBaseEUR` = Personnel + Travel + Equipment + OtherDirect (excludes Subcontracting).

---

## Part 4 — Entity Relationship Summary

```
EUTravelRateVersion ─────────────────────────────────────────────────────────────┐
   ├── Country  [accommodationRate, subsistenceRate]                              │
   └── FlightDistanceBand  [minKm, maxKm, flightCost]                            │
                                                                                  │
Project ──────────────────────────────────────────────────────── references ──────┘
   │  .durationYears, .numberOfWorkPackages
   │  .defaultInflationRate, .tryEurExchangeRate, .indirectCostRate
   │
   ├── WorkPackage (1..10)  [number, name?]
   │
   ├── PersonnelRole (0..N)  [roleLabel, salaryTRY, fte, inflationRate, activeYears]
   │      ├── SalaryProjection (1 per year)  [projectedMonthlyEUR]  ← computed
   │      └── PersonnelCostLine (1 per year)  [annualCostEUR]  ← computed
   │
   ├── EquipmentItem (0..N)  [name, cost, lifetime, usagePct, usageMonths]
   │      └── EquipmentDepreciation (1 per item)  [eligibleEUR]  ← computed
   │
   ├── Trip (0..N)  [type=Itemized|FlatAmount, year, instances, ...]
   │      └── TripCost (1 per trip)  [perInstance, total]  ← computed
   │               ├── references Country  (Itemized only)
   │               └── references FlightDistanceBand  (Itemized, ≥400 km only)
   │
   ├── OtherDirectCostItem (0..N)  [name, amount, year, isCFSItem]
   │      (max 1 with isCFSItem=true)
   │
   ├── Subcontracting (exactly 1)  [amountEUR = 0 by default]
   │
   └── BudgetSummary (1 per project — live computed)
          [personnelTotal, travelTotal, equipmentTotal, otherDirectTotal,
           subcontractingTotal, indirectTotal, directCostsTotal,
           eligibleCostsTotal, requestedContribution,
           cfsThresholdExceeded, cfsWarningActive]
```

---

## Part 5 — Entity Count and Classification

| Entity | Layer | Count per project |
|---|---|---|
| EUTravelRateVersion | Reference | Fixed (3 versions bundled) |
| Country | Reference | Fixed (~100+ per version) |
| FlightDistanceBand | Reference | Fixed (9 bands per version) |
| Project | User-configured | 1 |
| WorkPackage | User-configured | 1–10 |
| PersonnelRole | User-configured | 0–N (typically 5–15) |
| EquipmentItem | User-configured | 0–N (typically 1–20) |
| Trip | User-configured | 0–N (typically 5–30) |
| OtherDirectCostItem | User-configured | 0–N (typically 2–15) |
| Subcontracting | User-configured | Exactly 1 |
| SalaryProjection | Computed | 1 per (role × year) |
| PersonnelCostLine | Computed | 1 per (role × year) |
| EquipmentDepreciation | Computed | 1 per item |
| TripCost | Computed | 1 per trip |
| BudgetSummary | Computed | 1 (live aggregate) |

---

## Open Questions

No open questions. All design decisions required for the domain model are resolved in the business rules (business-rules.md).

---

**Confidence Level: 96%**

High confidence on all entities, attributes, and relationships — these are derived directly from the fully resolved business rules. Residual 4%: the year-level allocation of equipment cost (currently treated as a project total rather than year-assigned) may need revision in TASK-05 (Input Catalog) or TASK-08 (Calculation Engine) if a year-based equipment budget view is required by the UI design in TASK-06.

**Recommended Next Step:**  
Await PI approval. Once approved, proceed to TASK-05 (Input Catalog).
