# Excel Analysis

**Document:** TASK-02 Deliverable  
**Date:** 2026-07-10  
**Source file:** `fc5a3220-6301-4179-ba12-92943050b341.xlsx` (303 KB)  
**Status:** Draft — Awaiting Approval Before Proceeding to TASK-03

---

## 1. Workbook-Level Properties

| Property | Value |
|---|---|
| Total sheets | 10 |
| Named ranges | None |
| Excel tables (ListObjects) | None |
| Data validation rules | None |
| Conditional formatting | 1 rule (Gantt Chart visual only) |
| Hidden sheets | None |
| VBA / macros | None |
| Total formulas (all sheets) | ~348 |
| Total literal numeric inputs (all sheets) | ~223 |
| Languages | English (primary) + Turkish (notes, labels, comments) |

---

## 2. Worksheet Inventory

### 2.1 Sheet: Overview

**Role:** Final one-page summary output. This is the document the PI reviews; it shows total budget by category and the total EU contribution requested.

**Dimensions:** B2:E17 (small, presentation-only)

**Cells of interest:**

| Cell | Type | Content |
|---|---|---|
| C4 | Input (date) | Preparation date: 2025-12-25 |
| C6 | Input (text) | PI name: Candan Türkkan Ghosh |
| C7 | Input (text) | Call: ERC-CoG |
| C8 | Input (text) | Funding Type: Actual Costs |
| C11 | Formula | `='Budget (final)'!G10` — Total Personnel Cost |
| C12 | Formula | `='Budget (final)'!G12` — Travel & Subsistence |
| C13 | Formula | `='Budget (final)'!G13` — Equipment |
| C14 | Formula | `='Budget (final)'!G19` — Other Goods, Works & Services |
| C15 | Formula | `=SUM(C11:C14)` — Total Direct Costs |
| C16 | Formula | `=C15*0.25` — Indirect Costs (25% of direct) |
| C17 | Formula | `=SUM(C15,C16)` — Total Requested EU Contribution |

**Inputs:** 0 numeric inputs. All values are pulled from Budget (final).  
**Outputs:** C17 — Total requested EU contribution (primary KPI).  
**Cross-sheet refs out:** `Budget (final)` (4 cells read: G10, G12, G13, G19).  
**Formula count:** 7 | **Literal numeric inputs:** 0

---

### 2.2 Sheet: Budget (final)

**Role:** Consolidated budget table. Aggregates costs from Details into the standard ERC submission format (cost categories A–E). Acts as the bridge between the internal calculation layer (Details) and the summary output (Overview).

**Dimensions:** B4:G29

**Structure:**
- Rows 6–9: Personnel sub-categories (PI, Experts, Post-Docs, Admin)
- Row 10: Total Personnel (SUM)
- Row 11: Subcontracting (zero)
- Rows 12–19: Purchase Costs (C1 Travel, C2 Equipment, C3 sub-items)
- Row 20: Total Purchase Costs
- Row 21: Internally Invoiced Goods (D)
- Row 22: Indirect Costs (E = 25% × (A + C))
- Row 23: Total Eligible Costs
- Row 24: Requested EU Contribution (= Total Eligible Costs)
- Rows 27–29: Beneficiary Summary (Ozyegin University)

**Key formulas referencing Details:**

| Cell | Formula | What it reads |
|---|---|---|
| G6 | `=Details!I6` | PI total cost |
| G7 | `=Details!I7+Details!I12` | Expert-1 (yr1) + Expert-Aus (yr3) |
| G8 | `=Details!I8+I9+I10+I11+I13+I14` | All 6 Post-Doc totals |
| G9 | `=Details!I15` | Admin Staff total |
| G12 | `=Details!I28` | Travel total |
| G13 | `=SUM(Details!I19:I24)` | Equipment total (only I19 populated — see Issues) |
| G14 | `=Details!I26` | Consumables (empty — zero) |
| G16 | `=Details!I29` | Publications |
| G18 | `=Details!I30` | Other Direct Costs |
| G21 | `=Details!I32` | Internally Invoiced (empty — zero) |
| G22 | `=0.25*(G10+G20)` | Indirect costs: 25% × (Personnel + Purchase) |
| G23 | `=G10+G11+G20+G21+G22` | Total Eligible Costs |

**Literal numeric inputs:** G11 = 0 (Subcontracting, hardcoded zero).  
**Formula count:** 18 | **Literal numeric inputs:** 1

---

### 2.3 Sheet: Details

**Role:** Central calculation hub. The most complex sheet. Computes cost by staff member/cost line and by reporting period (5 × 12-month periods). Aggregates into the totals read by Budget (final).

**Dimensions:** A2:L37

**Structure:**

| Rows | Content |
|---|---|
| Row 2 | Period month counts (D2:H2 = 12, 12, 12, 12, 12) |
| Row 3 | Column headers (Period 1–5, Total, Total PMs, PM/month) |
| Row 4 | Label: Direct Costs |
| Row 5 | Label: A. Personnel |
| Rows 6–15 | Personnel cost rows (one per staff member) |
| Row 16 | Total Personnel subtotal per period |
| Row 17 | Label: Other Direct Costs |
| Row 18 | Label: C2 Equipment |
| Row 19 | Equipment total (pulled from Equipment C2 sheet) |
| Rows 20–24 | Empty rows (reserved for additional equipment categories) |
| Row 25 | Label: Consumables |
| Row 26 | Materials (empty — zero) |
| Row 27 | Label: Other |
| Row 28 | C1 Travel (same annual average across all 5 periods) |
| Row 29 | Publications (€5,000/yr in years 3–5 only; hardcoded) |
| Row 30 | C3 Other Direct Costs (piecemeal pull from C3 Other Goods) |
| Row 31 | Total Other Direct Costs |
| Row 32 | D. Internally Invoiced (empty) |
| Row 33 | Total Direct Costs |
| Row 34 | E. Indirect Costs (25% per period) |
| Row 35 | B. Subcontracting (all zeros) |
| Row 36 | Total Project Costs |
| Row 37 | Requested Grant |

**Personnel cost formula pattern (rows 6–15):**

Each cell in columns D–H (periods 1–5) follows:
```
= 'Salary Estimation'!<cell> × Details!<period_col>2 × Details!K<row>
  i.e. = monthly_salary_in_period × 12_months × FTE_fraction
```

Cells that contain `'-'` (string) instead of a formula indicate "this person is not active in this period." Excel's `SUM()` silently treats these as zero.

**Column K (FTE fractions):**

| Row | Person | K value |
|---|---|---|
| K6 | PI | 0.7 |
| K7 | Expert-1 | 0.4 |
| K8–K11, K13–K14 | Post-Docs (various) | 1.0 |
| K12 | Expert-Aus | 0.4 |
| K15 | Admin Staff | 1.0 |

**Cross-sheet refs out:** Salary Estimation, Personnel Costs, Equipment C2, C3 Other Goods, Travel and Subsist.  
**Formula count:** 92 | **Literal numeric inputs:** 30

---

### 2.4 Sheet: Salary Estimation

**Role:** Bottom-level salary input sheet. Captures base monthly salary per person in TRY-equivalent, converts to EUR at a fixed exchange rate, then projects year-by-year with an annual inflation multiplier. Produces an average monthly cost per year, which is the value consumed by Personnel Costs.

**Dimensions:** A1:J70

**Structure:** Each staff member occupies a self-contained block of ~7 rows:

| Sub-row | Content |
|---|---|
| Row n | Person name + base monthly salary (B = TRY equiv, B*rate = EUR base) |
| Row n+1 | Year-by-year salary: C=yr1×1.2 (or 1.15), D=C×1.15, E=D×1.15, … |
| Row n+2 | Annual inflation multiplier row (C7, D7, etc.) |
| Row n+3 | TRY value display (=C2 = exchange rate) |
| Header row | Column labels (Year 1–5, Average, period label) |

**Staff blocks and key inputs:**

| Block | Person | Base (EUR/mo) | PI Inflation | Expert Inflation | Period Used |
|---|---|---|---|---|---|
| Row 6–8 | PI | 4,500 | 1.20 (yr1 only) | 1.20 chain | Average of all 5 yrs |
| Row 11–13 | Expert-1 (TR) | 3,250 | 1.15 | 1.15 | Year 1 only (C11) |
| Row 17–19 | Post-Doc-1 (TR) | 3,000 | 1.15 | 1.15 | Year 2 (D17) |
| Row 23–25 | Post-Doc-2 (TR) | 3,000 | ref→C18 | ref | Year 2 (D23) |
| Row 30–32 | Post-Doc-3 (IN) | 3,000 | 1.15 hardcoded | 1.15 | Year 1 (C30) |
| Row 35–37 | Post-Doc-4 (IN) | 3,000 | 1.15 hardcoded | 1.15 | Year 1 (C35) |
| Row 41–43 | Expert (Aus) | 3,250 | 1.15 | 1.15 | Year 3 (E41) |
| Row 48–50 | Post-Doc-5 (Aus) | 3,000 | 1.15 | 1.15 | Year 3 (E48) |
| Row 55–57 | Post-Doc-6 (Aus) | 3,000 | 1.15 | 1.15 | Year 3 (E55) |
| Row 62–64 | Admin Staff | 3,000 | 1.15 | 1.15 | All 5 years (avg) |

**Hardcoded global constant:**
- `C2 = 50.62` — TRY/EUR exchange rate (used only for display row in TRY, not in EUR calculations)

**Inflation logic:**  
PI uses `C7 = 1.20` and chains `D7 = C7`, creating 20% inflation every year for all 5 years.  
All others use `1.15` per year, but Post-Docs 3 & 4 (IN) and Expert/Post-Docs (Aus) have these hardcoded per-cell rather than chaining from the first year's multiplier.

**Formula count:** 111 | **Literal numeric inputs:** 42

---

### 2.5 Sheet: Personnel Costs

**Role:** Intermediate aggregation layer. Reads average monthly salary from Salary Estimation and multiplies by total person-months to produce a total cost per individual. Acts as a named lookup for Details sheet column C (average cost per month label).

**Dimensions:** A1:E13

| Row | Person | Average Monthly Cost Source | Total PMs |
|---|---|---|---|
| 3 | PI | `='Salary Estimation'!H6` (5-yr average) | `=60*0.7 = 42` |
| 4 | Samarjit Hoca (Expert-1) | `='Salary Estimation'!I11` (yr1 only) | `=12*0.4 = 4.8` |
| 5 | Post Doc1 (y2) | `='Salary Estimation'!I17` (yr2 only) | 12 |
| 6 | Post Doc2 (y2) | `='Salary Estimation'!I23` (yr2 only) | 12 |
| 7 | Post Doc4 (y1) | `='Salary Estimation'!I30` (yr1 only) | 12 |
| 8 | Post Doc5 (y1) | `='Salary Estimation'!I35` (yr1 only) | 12 |
| 9 | Expert (Aus) | `='Salary Estimation'!I41` (yr3 only) | 4.8 |
| 10 | Post Doc6 (y3) | `='Salary Estimation'!I48` (yr3 only) | 12 |
| 11 | Post Doc7 (y3) | `='Salary Estimation'!I55` (yr3 only) | 12 |
| 12 | Admin Staff | `='Salary Estimation'!I62` (5-yr average) | 60 |

**Note:** This sheet provides reference values used as labels in Details!C6:C15 but the actual cost calculations in Details use the Salary Estimation year-specific values directly (not the averages from this sheet, except for PI and Admin Staff).

**Formula count:** 24 | **Literal numeric inputs:** 8

---

### 2.6 Sheet: Gantt Chart

**Role:** Visual staff allocation plan. Maps each staff member to active months across the 60-month project duration, grouped by 5 Work Packages. No calculations — purely visual/informational. Contains one unresolved note in Turkish.

**Dimensions:** B2:BJ20 (62 columns for months 1–60 + label)

**Content:**
- Row 5: Month numbers 1–60 (all literal integers)
- Row 6: PI — all 60 months marked `'x'`
- Row 7: Samarjit Hoca — months 1–12 only
- Row 8: Post Doc1 (y2) — months 13–24
- Row 9: Post Doc2 (y2) — months 13–24
- Row 10: Post Doc3 (y4) — months 37–48
- Row 11: Post Doc4 (y1) — months 1–12
- Row 12: Post Doc5 (y1) — months 1–12
- Row 13: Expert (Aus) — months 25–36
- Row 14: Post Doc6 (y3) — months 25–36
- Row 15: Post Doc7 (y3) — months 25–36
- Row 16: Admin Staff — all 60 months
- Row 4: WP labels: WP#1 (C4:N4), WP#2 (O4:Z4), WP#3 (AA4:AL4), WP#4 (AM4:AX4), WP#5 (AY4:BJ4)

**Note in F2:** `'Hocaya sorulacak kısım'` (Turkish: "section to be asked to professor") — indicates the WP structure is not yet finalised.

**Conditional formatting:** One rule over C6:BJ16 (presumably to colour `'x'` cells).

**Formula count:** 0 | **Literal numeric inputs:** 60 (month number row only)

---

### 2.7 Sheet: Equipment C2

**Role:** Calculates eligible depreciation cost for each equipment item purchased during the project. Applies the ERC rule: only the portion of the equipment's useful life spent on the project is eligible.

**Dimensions:** A2:Q26

**Depreciation formula (column I, rows 7–14):**

```
=IF(
  AND(F>0, ISNUMBER(D), ISNUMBER(F), ISNUMBER(G), ISNUMBER(H)),
    IF(
      OR( (D/F)*G*H >= D, F < H ),
        D * IF(G > 100%, 1, G),           -- cap at purchase cost
        (D/F) * G * H                      -- normal depreciation
    ),
  ""
)

Where:
  D = Purchase cost per item (€)
  F = Useful lifetime (months)
  G = Usage percentage for the grant (fraction, e.g. 1 = 100%)
  H = Expected usage time during the grant (months)
```

**Equipment items:**

| Row | Item | Cost (€) | Lifetime (mo) | Usage % | Grant months | Eligible (formula) |
|---|---|---|---|---|---|---|
| 7 | Laptop – PI | 2,500 | 48 | 100% | 55 | capped at 2,500 |
| 8 | Laptop – Expert | 2,500 | 48 | 100% | 55 | capped at 2,500 |
| 9 | Laptop – Post Doc | 2,500 | 48 | 100% | 55 | capped at 2,500 |
| 10 | Laptop – Post Doc | 2,500 | 48 | 100% | 55 | capped at 2,500 |
| 11 | Laptop – Admin | 2,500 | 48 | 100% | 55 | capped at 2,500 |
| 12 | Audio recorder 1 | 60 | 60 | 100% | 36 | (60/60)×1×36 = 36 |
| 13 | Audio recorder 2 | 60 | 60 | 100% | 36 | 36 |
| 14 | Audio recorder 3 | 60 | 60 | 100% | 36 | 36 |

**Notes embedded in worksheet (rows 18–26):**
- Note 1 (B18:H22): ERC equipment depreciation rule explanation (English)
- Note 2 (B23:I26): Import customs surcharge guidance — recommends adding ≥20% buffer for imported equipment (Turkish origin)

**Formula count:** 11 | **Literal numeric inputs:** 32

---

### 2.8 Sheet: C3 Other Goods

**Role:** Lists all C3 "other goods, works and services" cost items with amounts and justifications.

**Dimensions:** A1:R8 (small — 6 data rows + header + total)

| Row | Item | Amount (€) | Period in Details |
|---|---|---|---|
| 2 | Publications (open access) | 15,000 | Yr 3–5 via Details!F29:H29 (5k/yr) |
| 3 | Translation services | 3,000 | Year 3 via Details!G30 |
| 4 | Certificate on Financial Statements (CFS) | 12,000 | Year 4 via Details!H30 |
| 5 | Fieldwork costs (Saha Çalışması) | 20,000 | Year 1 via Details!D30 |
| 6 | MAXQDA software | 9,870 | Year 1 via Details!D30 |
| 7 | Fireflies AI subscription | 1,140 | Year 1 via Details!D30 |
| 8 | **Total** | `=SUM(B2:B7)` = 61,010 | |

**Formula count:** 1 | **Literal numeric inputs:** 6

---

### 2.9 Sheet: Travel and Subsist.

**Role:** Itemised travel cost calculations for all planned trips. Builds up per-destination costs (accommodation + flights + daily allowances + visa + domestic travel), applies multipliers (number of trips), sums to a grand total, then divides by 5 to produce an annual average that is applied uniformly across all project years.

**Dimensions:** A1:K94

**Destinations and multipliers:**

| Destination | One-trip cost source | Multiplier | Total |
|---|---|---|---|
| England | B8 (=1,059) | ×1 | ~1,059 |
| France | B16 (~1,841) | ×3 | ~5,523 |
| Australia | B54 | ×3 | |
| Spain | B24 | ×3 | |
| India | B46 | ×6 | |
| Austria | B32 | ×3 | |
| Turkey | B38 | ×6 | |
| USA | B61 | ×1 | |
| Conference – PI | F12 = €2,000 flat | ×3 | 6,000 |
| Conference – w/Samarjit | F13 = €4,000 flat | ×1 | 4,000 |
| Conference – Post-Doc | F14 = €2,000 flat | ×6 | 12,000 |
| Conference – w/Dilek Hoca | F15 = €4,000 (literal) | ×1 | 4,000 |

**Key output:**
- `G16` = `=SUM(H4:H15)` — Grand total travel budget (all trips × multipliers)
- `K6` = `=G16/5` — **Annual average** (divides by 5 years, hardcoded divisor)

**EU daily/accommodation limits (column C):** Reference values only; not used in any formulas. The actual rates used in column B formulas are embedded inline.

**Formula count:** 84 | **Literal numeric inputs:** 44

---

### 2.10 Sheet: Other costs

**Role:** Unknown. Contains no data whatsoever — no labels, no values, no formulas.

**Status:** Empty placeholder. May be vestigial from a template or intended for future use.

**Formula count:** 0 | **Literal numeric inputs:** 0

---

## 3. Cross-Sheet Dependency Map

```
Salary Estimation  (no external refs — pure input)
       │
       ▼
Personnel Costs  (reads Salary Estimation!H6, I11, I17, I23, I30, I35, I41, I48, I55, I62)
       │
       ▼
Details  ◄─── Equipment C2  (no external refs — pure input)
       │ ◄─── C3 Other Goods  (no external refs — pure input)
       │ ◄─── Travel and Subsist.  (no external refs — pure input)
       │
       ▼
Budget (final)  (reads Details!I6, I7, I12, I8–I11, I13–I14, I15, I28, I19:I24, I26, I29, I30, I32)
       │
       ▼
Overview  (reads Budget (final)!G10, G12, G13, G19)

Gantt Chart  ──  standalone (no refs in or out)
Other costs  ──  standalone (no refs in or out)
```

**Dependency layers:**

| Layer | Sheets |
|---|---|
| 0 (pure inputs) | Salary Estimation, Equipment C2, C3 Other Goods, Travel and Subsist. |
| 1 (first aggregation) | Personnel Costs |
| 2 (central hub) | Details |
| 3 (submission format) | Budget (final) |
| 4 (final output) | Overview |
| Disconnected | Gantt Chart, Other costs |

---

## 4. Inputs

All user-supplied data (literal values that drive calculations):

### Project-Level Inputs

| Sheet | Cell | Value | Description |
|---|---|---|---|
| Overview | C4 | 2025-12-25 | Preparation date |
| Overview | C6 | Candan Türkkan Ghosh | PI name |
| Overview | C7 | ERC-CoG | Call identifier |
| Overview | C8 | Actual Costs | Funding type |
| Salary Estimation | C2 | 50.62 | TRY/EUR exchange rate |

### Personnel Inputs

| Sheet | Cell | Value | Description |
|---|---|---|---|
| Salary Estimation | B6 | 4,500 | PI base monthly salary (EUR) |
| Salary Estimation | C7 | 1.20 | PI annual inflation multiplier |
| Salary Estimation | B11 | 3,250 | Expert-1 base salary (EUR) |
| Salary Estimation | C12, D12 | 1.15 | Expert-1 inflation multiplier |
| Salary Estimation | B17 | 3,000 | Post-Doc-1 base salary (EUR) |
| Salary Estimation | B23 | `=B17` | Post-Doc-2 salary (same as PD1) |
| Salary Estimation | B30 | `=B17` | Post-Doc-3 salary (same as PD1) |
| Salary Estimation | B35 | `=B17` | Post-Doc-4 salary (same as PD1) |
| Salary Estimation | B41 | 3,250 | Expert (Aus) base salary (EUR) |
| Salary Estimation | B48 | `=B17` | Post-Doc-5 salary |
| Salary Estimation | B55 | `=B17` | Post-Doc-6 salary |
| Salary Estimation | B62 | 3,000 | Admin Staff base salary (EUR) |
| Salary Estimation | C31–G31 | 1.15 (×5) | Post-Doc-3 inflation (hardcoded all years) |
| Salary Estimation | C36–G36 | 1.15 (×5) | Post-Doc-4 inflation (hardcoded all years) |
| Details | K6 | 0.7 | PI FTE fraction |
| Details | K7, K12 | 0.4 | Expert FTE fraction |
| Details | K8–K11, K13–K15 | 1.0 | Post-Doc / Admin FTE fraction |
| Details | D2:H2 | 12 (×5) | Months per reporting period |

### Equipment Inputs

| Sheet | Cell | Value | Description |
|---|---|---|---|
| Equipment C2 | D7–D11 | 2,500 | Laptop purchase cost (€) per unit |
| Equipment C2 | F7–F11 | 48 | Laptop useful lifetime (months) |
| Equipment C2 | G7–G11 | 1 | Laptop grant usage % (100%) |
| Equipment C2 | H7–H11 | 55 | Laptop grant usage months |
| Equipment C2 | D12–D14 | 60 | Audio recorder cost (€) each |
| Equipment C2 | F12–F14 | 60 | Audio recorder useful lifetime (months) |
| Equipment C2 | H12–H14 | 36 | Audio recorder grant usage months |

### C3 Other Goods Inputs

| Sheet | Cell | Value | Description |
|---|---|---|---|
| C3 Other Goods | B2 | 15,000 | Publications (open access) |
| C3 Other Goods | B3 | 3,000 | Translation services |
| C3 Other Goods | B4 | 12,000 | CFS (Certificate on Financial Statements) |
| C3 Other Goods | B5 | 20,000 | Fieldwork costs |
| C3 Other Goods | B6 | 9,870 | MAXQDA software |
| C3 Other Goods | B7 | 1,140 | Fireflies AI subscription |

### Travel Inputs

| Sheet | Cells | Description |
|---|---|---|
| Travel | B4:B7 | England trip: accommodation (209×1×5), flight (429×1), daily allowance (125×6), visa (150) |
| Travel | B11:B15 | France trip: accommodation (212×5), flight (429), daily allowance (127×6), visa (120), train (200) |
| Travel | B19:B23 | Spain trip: accommodation (154×5), flight (541), daily allowance (101×6), visa (120), train (200) |
| Travel | B28:B31 | Austria trip: accommodation (170×5 — see Issue E3), flight (365), daily allowance (131×6), visa (150) |
| Travel | B35:B38 | Turkey trip: accommodation (170×3×13), flights (75×3), daily allowance (55×14×3) |
| Travel | B42:B46 | India trip: accommodation (195×4×13), flights (857×2), domestic (340×4), daily allowance (50×14×4) |
| Travel | B49:B54 | Australia trip: accommodation (135×4×13), flight (1,595×1), domestic (365×4), daily allowance (75×14×4), visa (110) |
| Travel | B57:B61 | USA trip: accommodation (200×1×5), flight (1,250×1), daily allowance (80×1×6), visa (150) |
| Travel | G4:G15 | Trip multipliers (number of times each trip is taken) |
| Travel | F12–F15 | Conference flat costs (€2,000 or €4,000 per event) |

---

## 5. Outputs

| Sheet | Cell | Description |
|---|---|---|
| Overview | C17 | **Total requested EU contribution** (primary output) |
| Overview | C15 | Total direct costs |
| Overview | C16 | Indirect costs (25% of direct) |
| Budget (final) | G10 | Total personnel cost |
| Budget (final) | G20 | Total purchase costs (C1+C2+C3) |
| Budget (final) | G22 | Indirect costs |
| Budget (final) | G23 | Total eligible costs |
| Budget (final) | G24 | Requested EU contribution |
| Budget (final) | F29/G29 | Ozyegin University total cost / requested amount |
| Details | I36 | Total project costs (all periods) |
| Details | I37 | Requested grant (= I36) |
| Equipment C2 | I15 | Total eligible equipment cost |

---

## 6. Intermediate Calculations

### 6.1 Salary Inflation Chain

For each person, starting from a base monthly salary (`B`), the sheet computes year-by-year salary cost:

```
Year1 = B × inflation_multiplier_yr1
Year2 = Year1 × inflation_multiplier_yr2
...
Year5 = Year4 × inflation_multiplier_yr5
```

For PI: multiplier = 1.20 each year (chained).  
For all others: multiplier = 1.15 each year (some hardcoded, some chained).

A single period-specific value (`AVERAGE(CellYrN)` = just that year's value for single-period staff) is consumed by Personnel Costs and then by Details.

### 6.2 Personnel Cost Per Period

```
Cost(person, period) = salary_this_year × months_in_period × FTE_fraction
                     = Salary Estimation[year_col] × 12 × Details!K[row]
```

The total per person across the project (`Details!I`) sums columns D–H.

### 6.3 Equipment Depreciation

```
eligible = (purchase_cost / useful_lifetime_months) × usage_pct × grant_usage_months
```

Capped at `purchase_cost × usage_pct` if grant usage exceeds total lifetime or eligible amount would exceed cost.

### 6.4 Travel Annual Average

```
grand_total = SUM of (per_destination_cost × trip_multiplier)
annual_average = grand_total / 5
```

This annual average is then applied identically to all five project years (`Details!D28:H28` = same value × 5).

### 6.5 Indirect Cost

Applied at 25% of the sum of direct costs (A + C1 + C2 + C3), computed three times independently at different aggregation levels (see Issue D1).

---

## 7. Hardcoded Constants

Constants embedded inside formulas (not in named input cells):

| Location | Constant | Meaning |
|---|---|---|
| Overview!C16 | 0.25 | Indirect cost rate (25%) |
| Budget (final)!G22 | 0.25 | Indirect cost rate (25%) |
| Details!D34:H34 | 0.25 (×5) | Indirect cost rate (25%) — per period |
| Details!J7 | `=12*0.4` | Expert-1: 12 months × 0.4 FTE |
| Details!J12 | `=12*0.4` | Expert-Aus: 12 months × 0.4 FTE |
| Personnel Costs!D3 | `=60*0.7` | PI: 60 months × 0.7 FTE |
| Personnel Costs!D4 | `=12*0.4` | Expert-1: 12 months × 0.4 FTE |
| Travel!K6 | `/5` | Project duration in years (divisor) |
| Travel!B4 | 209 | England accommodation rate (€/night) |
| Travel!B5 | 429 | England flight cost |
| Travel!B6 | 125 | England daily allowance |
| Travel!B28 | 170 | Austria accommodation rate (≠ EU limit of 158) |
| Many travel cells | various | All per-diem/flight rates inline |

---

## 8. Formula Categories by Sheet

| Sheet | Arithmetic | Aggregation (SUM/AVG) | Conditional (IF/AND) | Total |
|---|---|---|---|---|
| Overview | 5 | 2 | 0 | 7 |
| Budget (final) | 15 | 3 | 0 | 18 |
| Details | 53 | 39 | 0 | 92 |
| Salary Estimation | 92 | 19 | 0 | 111 |
| Personnel Costs | 22 | 2 | 0 | 24 |
| Gantt Chart | 0 | 0 | 0 | 0 |
| Equipment C2 | 1 | 2 | 8 | 11 |
| C3 Other Goods | 0 | 1 | 0 | 1 |
| Travel and Subsist. | 70 | 14 | 0 | 84 |
| Other costs | 0 | 0 | 0 | 0 |
| **Total** | **258** | **82** | **8** | **348** |

---

## 9. Calculation Chains

**Chain A — Personnel to Total:**
```
Salary Estimation!B[base] 
  → ×inflation → C[yr1]…G[yr5]
  → AVERAGE or period-specific → H or I
  → Personnel Costs!C[row] 
  → Details!C[row] (label ref)
  
Salary Estimation!C[yr_col] 
  → × Details!D2 (=12) × Details!K[row] (FTE)
  → Details!D[row] (period cost)
  → SUM(D:H) → Details!I[row] (total per person)
  → SUM(I6:I15) → Details!I16
  → Budget (final)!G6/G7/G8/G9
  → SUM(G6:G9) → Budget (final)!G10
  → Overview!C11
```

**Chain B — Equipment to Total:**
```
Equipment C2!D (cost), F (lifetime), G (usage%), H (months)
  → IF formula → Equipment C2!I (eligible depreciation per item)
  → SUM(I7:I14) → Equipment C2!I15
  → Details!D19 (via SUM('Equipment C2'!I7:I14))
  → SUM(D19:H19) → Details!I19
  → Budget (final)!G13 (via SUM(Details!I19:I24))
  → Overview!C13
```

**Chain C — Travel to Total:**
```
Travel!B[destination_total] × Travel!G[multiplier]
  → Travel!H (trip total)
  → SUM(H4:H15) → Travel!G16
  → /5 → Travel!K6 (annual average)
  → Details!D28:H28 (same value in all 5 periods)
  → SUM(D28:H28) → Details!I28
  → Budget (final)!G12
  → Overview!C12
```

**Chain D — C3 to Total (fragmented):**
```
C3 Other Goods!B2 (publications) → Details!F29, G29, H29 (hardcoded 5000 each, no formula link)
C3 Other Goods!B3 (translation) → Details!G30
C3 Other Goods!B4 (CFS) → Details!H30
C3 Other Goods!B5:B7 (fieldwork+MAXQDA+Fireflies) → Details!D30
  → SUM(D29:H29) → Details!I29
  → SUM(D30:H30) → Details!I30
  → Budget (final)!G16 (publications)
  → Budget (final)!G18 (other direct costs)
  → SUM(G14:G18) → Budget (final)!G19
  → Overview!C14
```

---

## 10. Dead Logic

**DL-01 — Salary Estimation!I5 = 0.7**  
Orphaned value. The PI FTE fraction is stored in I5 of Salary Estimation but is not referenced by any formula. The actual PI FTE is separately hardcoded in `Personnel Costs!D3` as `=60*0.7` and in `Details!K6 = 0.7`. This cell serves no calculation purpose.

**DL-02 — Salary Estimation!J10 = 0.4 and J40 = 0.4**  
Same pattern for Expert-1 and Expert (Aus). These FTE headers in column J/I of Salary Estimation are annotation values never used in formulas.

**DL-03 — Details rows 20–24 (equipment sub-rows)**  
Five completely empty rows exist between the Equipment total (row 19) and the Consumables section (row 25). These appear to have been reserved for additional equipment sub-categories (the header in row 18 says "Equipment") but were never populated. Budget (final)!G13 references `Details!I19:I24`, meaning these empty rows are already summed — they are harmless but represent unfulfilled design intent.

**DL-04 — Details row 26 (Materials/Consumables)**  
Label exists ("Materials") with formula `I26 = =SUM(D26:H26)`, but D26:H26 are all empty. Budget (final)!G14 reads this zero. Reserved but unused.

**DL-05 — Details row 27 ("Other")**  
Label cell only. No values or formulas. May have been intended for additional C3 sub-lines.

**DL-06 — Sheet "Other costs"**  
Entirely empty. No labels, values, or formulas. Serves no function.

**DL-07 — Salary Estimation!B8, B13, B19, B25, B32, B37, B43, B50, B57, B64**  
Each block has a row that displays the TRY-equivalent salary (`=B[base] * A[row]` where A[row] = TRY/EUR rate). These are informational display cells only; they feed nothing downstream.

**DL-08 — Personnel Costs column E (Total Cost)**  
Column E in Personnel Costs computes total cost per person (`E3:E13`). These values are not referenced by Details or any other sheet. Details computes its own period-by-period costs independently. This column appears to be a sanity-check/reference only.

---

## 11. Duplicate Logic

**DUP-01 — Indirect cost rate (25%) hardcoded in three places**  
The 25% overhead rate appears as a literal constant in:
- `Overview!C16 = C15*0.25`
- `Budget (final)!G22 = 0.25*(G10+G20)`
- `Details!D34:H34 = 0.25*D33` (one per period column)

If the rate ever changes (e.g. if a different overhead rule applies), all seven cells must be manually updated. None are driven by a single shared parameter.

**DUP-02 — FTE fraction appears in three sheets**  
PI FTE (0.7) and Expert FTE (0.4) are each stored in three independent locations:
1. Salary Estimation (I5, J10, J40 — orphaned/unused)
2. Details (K6, K7, K12 — used in cost calculation)
3. Personnel Costs (formula constants in D3, D4 — used for PM count only)

A change in FTE must be made in two active places (Details + Personnel Costs) to keep PM totals and cost totals consistent.

**DUP-03 — Post-Doc base salary repeated via reference**  
`Salary Estimation!B23 = B17`, `B30 = B17`, `B35 = B17`, `B48 = B17`, `B55 = B17`. This chains all Post-Doc salaries to PD-1's value (B17 = 3,000). A change to B17 propagates to all. Correct design, but the dependency is invisible unless you trace the chain.

**DUP-04 — Publication costs duplicated without link**  
`C3 Other Goods!B2 = 15,000` (total publications budget). `Details!F29 = G29 = H29 = 5,000` (€5k/yr for years 3–5, totalling €15k). These arrive at the same total but there is no formula linking them. Changing one does not change the other.

---

## 12. Potential Errors

**E-01 — String dash '-' in period cells (HIGH RISK)**  
34 cells in `Details!D7:H14` contain the string value `'-'` to indicate "no activity in this period." Excel's `SUM()` silently ignores strings, so totals currently work. However:
- If these cells were ever referenced in multiplication (e.g., `=A1 * D7`), the formula would return `#VALUE!`
- A strict calculation engine treating these as text would also fail
- In the software product, all empty periods must be treated as numeric zero, not the string `'-'`

**E-02 — Austria accommodation rate mismatch (MEDIUM RISK)**  
`Travel!B28 = =170*1*5` but the label in A28 says `=158 Eur*1person*5nights` and the EU limit column C28 = 158. The formula uses 170 instead of 158. This overcharge vs. the EU-stated daily accommodation limit for Austria (€158) may create an eligibility problem. The extra €60 total may be queried by auditors.

**E-03 — Travel budget identical across all years (MEDIUM RISK)**  
`Details!D28:H28` all reference the same `Travel!K6` (annual average). This means the budget assumes identical travel spending in each of the five years. In reality, the travel plan shows fieldwork visits concentrated in certain years (India ×6, Turkey ×6). The per-period travel allocation is inaccurate — it should vary by year according to the trip plan.

**E-04 — Budget (final)!G13 references empty rows (LOW RISK — currently zero)**  
`=SUM(Details!I19:I24)` includes rows 20–24 which are empty. Numerically harmless today. If rows 20–24 are accidentally populated in future without updating the formula, double-counting would occur.

**E-05 — C3 Other Goods mapping gap: B3 and B4 skipped in Details!D30 (HIGH RISK)**  
`Details!D30 = =SUM('C3 Other Goods'!B5:B7)` pulls only rows 5–7 (fieldwork, MAXQDA, Fireflies) for period 1. Rows B3 (Translation €3k) and B4 (CFS €12k) are assigned to specific later years via individual references (`G30=B3`, `H30=B4`), but only for years 3 and 4 respectively. Row B2 (Publications €15k) is handled entirely separately in Details!F29:H29. A user editing the C3 sheet might expect all items to flow through — this is not the case, and the mapping is neither documented nor obvious.

**E-06 — Personnel naming inconsistency across sheets (MEDIUM RISK)**  
The same individual appears under different names across sheets:

| Details | Personnel Costs | Gantt Chart | Salary Estimation |
|---|---|---|---|
| A.1. Expert-1 | Samarjit Hoca | Samarjit Hoca | Expert-1 |
| A.2. Post Doc3 (y1) | Post Doc4 (y1) | Post Doc4 (y1) | Post-Doc-3 (IN) |
| A.2. Post Doc4 (y1) | Post Doc5 (y1) | Post Doc5 (y1) | Post-Doc-4 (IN) |
| A.3. Expert (y3) | Expert (Aus) | Expert (Aus) | Expert (Avustralya) |
| A.2. Post Doc5 (y3) | Post Doc6 (y3) | Post Doc6 (y3) | Post-Doc-5 (Avustralya) |
| A.2. Post Doc6 (y3) | Post Doc7 (y3) | Post Doc7 (y3) | Post-Doc-6 (Avustralya) |

There is a numbering offset of +1 between Details and all other sheets for Post-Docs 3–6. Additionally, Gantt row 10 labels "Post Doc3 (y4)" but places this person in months 37–48 (year 4), while Details calls a similarly positioned entry "Post Doc3 (y1)". This appears to be the same individual labelled differently.

**E-07 — Post-Doc-3/4 (India) inflation hardcoded not chained (LOW RISK)**  
Rows 31 and 36 in Salary Estimation have all five inflation multipliers (C31:G31 and C36:G36) set to literal 1.15 each, rather than chaining from a single cell as Expert-1 does (D12 = 1.15, then E12 = `=D12`, etc.). This means if the inflation assumption for Indian staff is changed, five cells per person must be updated individually rather than one.

**E-08 — Overview indirect cost base excludes D (internally invoiced)**  
`Overview!C16 = C15*0.25` where C15 = Personnel + Travel + Equipment + Other Goods. This excludes category D (internally invoiced goods) from the overhead base.  
`Budget (final)!G22 = 0.25*(G10+G20)` where G20 = C1+C2+C3 (also excludes D). These are consistent with each other and with ERC rules (D has no indirect costs).  
`Details!D34 = 0.25*D33` where D33 = D16+D31+**D32**. D32 is category D (internally invoiced). This means the Details overhead calculation **includes** category D in its base — inconsistent with Overview and Budget (final), and potentially non-compliant with ERC rules. However, since D32 is currently empty/zero, the discrepancy has no numerical effect today.

---

## 13. Hidden Assumptions

| ID | Assumption | Location | Risk if Wrong |
|---|---|---|---|
| HA-01 | All 5 reporting periods are exactly 12 months | Details!D2:H2 = 12 | ERC uses 2 reporting periods; this structure would need restructuring |
| HA-02 | Travel budget is distributed equally across all years | Details!D28:H28 = same K6 | Actual trips are concentrated in specific years; per-period breakdown is inaccurate |
| HA-03 | TRY/EUR exchange rate is fixed at 50.62 for 5 years | Salary Estimation!C2 | Currency risk not modelled; Turkish inflation makes this highly volatile |
| HA-04 | Indirect cost rate is exactly 25% | Hardcoded in 7 cells | ERC allows up to 25%; if a different rate is negotiated, all must be updated manually |
| HA-05 | All staff are 100% at Ozyegin University | Implicit in single beneficiary | If collaborators from India/Australia are at their home institutions, cost eligibility rules differ |
| HA-06 | Post-Doc base salary is the same for all 7 Post-Docs | Salary Estimation: all B[base] = =B17 | Actual market salaries may vary; currently all forced equal |
| HA-07 | Salary raise applies from the beginning of each year uniformly | Multiplication chain | ERC requires costs based on actual payroll; uniform inflation may not match real raises |
| HA-08 | Equipment useful life (48 months for laptops) means full cost is eligible at 55 months of use | Equipment C2!F7=48, H7=55 | Because H > F (55 > 48), the IF cap triggers and the full cost is charged — intentional but should be validated against ERC policy |

---

## 14. Summary Statistics

| Metric | Value |
|---|---|
| Total sheets | 10 |
| Active sheets (have data) | 9 |
| Empty sheets | 1 (Other costs) |
| Disconnected sheets (no refs in/out) | 2 (Gantt, Other costs) |
| Total formulas | ~348 |
| Total literal numeric inputs | ~223 |
| Named ranges | 0 |
| Excel tables | 0 |
| Data validations | 0 |
| Dead logic items identified | 8 |
| Duplicate logic items identified | 4 |
| Potential errors identified | 8 |
| Hidden assumptions identified | 8 |
| Formula depth (layers) | 5 |
| Staff members modelled | 10 |
| Cost categories (ERC A–E) | 5 |
| Project duration | 60 months (5 years) |
| Reporting periods in workbook | 5 (vs. ERC standard 2) |
| Beneficiaries | 1 (Ozyegin University) |
| Work Packages | 5 (names unknown) |

---

**Confidence Level: 90%**

High confidence on all formula structures, dependencies, and issues. Residual uncertainty: (1) computed totals not verified (no live Excel engine); (2) some Turkish-language cells may contain additional context not captured; (3) the intent behind some empty rows (DL-03 to DL-06) is inferred, not confirmed.

**Recommended Next Step:** Proceed to TASK-03 (Business Rules) after PI approval. Priority clarifications before TASK-03: resolve the personnel naming inconsistency (E-06), confirm the reporting period structure (HA-01), and confirm whether category D will be populated.
