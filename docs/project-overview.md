# Project Overview

**Document:** TASK-01 Deliverable  
**Date:** 2026-07-10  
**Status:** Draft — Awaiting Approval Before Proceeding to TASK-02

---

## 1. Workbook Purpose

The workbook is a custom-built budget preparation tool for an **ERC Consolidator Grant (ERC-CoG)** submission under Horizon Europe. It was authored for PI **Candan Türkkan Ghosh** at **Ozyegin University, Turkey**, with preparation dated **25 December 2025**.

Its purpose is to calculate, organise, and summarise the full five-year project budget according to ERC/Horizon Europe cost category rules — covering personnel, equipment, travel, other direct costs, and indirect costs — and to produce a final budget table ready for submission in the grant application form.

> **Critical Discrepancy Flagged:** The project brief is titled "Horizon Europe Lump Sum Budget Application," but the workbook itself states `Funding Type: Actual Costs`. ERC Consolidator Grants use **actual cost** reimbursement, not the lump sum model used in some other Horizon Europe actions. This distinction has major implications for the product design and calculation engine. **Clarification required from PI before proceeding.**

---

## 2. High-Level Structure

The workbook contains **10 worksheets** organised in a clear data-flow hierarchy:

| # | Sheet Name | Role |
|---|---|---|
| 1 | Overview | Final summary output (total budget by category + indirect costs) |
| 2 | Budget (final) | Consolidated budget table; main output layer |
| 3 | Details | Central calculation hub; aggregates all cost lines by reporting period |
| 4 | Salary Estimation | Lowest-level salary input & year-on-year inflation projection per person |
| 5 | Personnel Costs | Intermediate layer — extracts average monthly cost and total PMs per role |
| 6 | Gantt Chart | Staff allocation plan across 60 months and 5 Work Packages |
| 7 | Equipment C2 | Equipment depreciation calculations per item |
| 8 | C3 Other Goods | Other direct costs (publications, software, field work, services) |
| 9 | Travel and Subsist. | Per-destination travel cost build-up and annual average |
| 10 | Other costs | Appears empty (no data detected) |

**Data flow (simplified):**

```
Salary Estimation
       ↓
Personnel Costs
       ↓
Details ← Equipment C2
       ← Travel and Subsist.
       ← C3 Other Goods
       ↓
Budget (final)
       ↓
Overview
```

---

## 3. Number of Sheets

**10 sheets total.** One sheet (Other costs) contains no data and may be a placeholder or vestigial tab.

---

## 4. Major Budgeting Domains

The workbook covers the five standard ERC/Horizon Europe direct cost categories plus indirect costs:

**A. Personnel Costs**  
The largest and most complex domain. Covers 10 individuals across four staff categories: Principal Investigator (42 PM at 70% FTE), Expert collaborators (2 × 4.8 PM at 40% FTE), Post-Doctoral researchers (6 × 12 PM at 100% FTE, staggered across years 1–3), and Administrative Staff (60 PM at 100% FTE). Salary calculation involves Turkish Lira base salaries converted to EUR at a fixed exchange rate (50.62 TRY/EUR), with year-on-year inflation multipliers of 15–20% per individual. Collaborators based in India and Australia are also included.

**B. Subcontracting**  
Currently set to zero. A budget line exists in the structure but is not populated.

**C1. Travel and Subsistence**  
Multi-destination fieldwork and conference travel spanning England, France, Spain, Austria, Turkey, India, Australia, and the USA. Each destination has itemised accommodation, flights, daily allowances, visa fees, and domestic travel. Conference attendance (PI, expert collaborator, and Post-Docs separately) is also calculated. A total annual travel average feeds into the Details sheet.

**C2. Equipment**  
Depreciation-based eligibility calculation for 8 items: laptops (PI, Expert, Post-Docs × 2, Admin) and audio recording devices × 3. Each item has a purchase cost, useful lifetime (months), grant usage percentage, and expected usage duration — producing an eligible depreciation cost. Turkish classification codes and economic lifetimes are also tracked.

**C3. Other Goods, Works and Services**  
Six line items: open-access publications (€15,000), translation services (€3,000), Certificate on Financial Statements — CFS (€12,000), fieldwork costs (€20,000), MAXQDA software subscription (€9,870), and Fireflies AI Business Plan subscription (€1,140).

**D. Internally Invoiced Goods and Services**  
A single line referencing the "Industry 4.0 Center" at Ozyegin University. Not currently populated with a value.

**E. Indirect Costs (Overheads)**  
Applied at the ERC standard rate of **25%** of direct costs (Personnel + C1 + C2 + C3). Calculated both per reporting period and as a total. The Overview sheet recalculates this independently as a cross-check.

---

## 5. Complexity Assessment

**Overall Complexity: HIGH**

| Dimension | Assessment |
|---|---|
| Formula depth | 4–5 layers of cross-sheet dependencies |
| Personnel model | Highly individualised: per-person salary base, per-person inflation rate, per-person FTE fraction, per-person start/end month |
| Currency handling | TRY-to-EUR conversion baked into salary calculations with a single hardcoded exchange rate |
| Equipment depreciation | Non-trivial conditional formula: capped at purchase cost, sensitive to lifetime vs. usage duration |
| Travel | Itemised per destination, per person category, with multipliers and annual averaging |
| Language mix | Worksheets contain a mix of English and Turkish labels, notes, and formula comments |
| Reporting periods | Budget tracked across five 12-month periods (M1–12, M13–24, M25–36, M37–48, M49–60) |
| Work Packages | 5 WPs tracked via Gantt; not yet linked to cost lines |
| Auditability | The workbook was clearly evolved iteratively; several cells contain Turkish working notes (e.g., "Hocaya sorulacak kısım" — "to be asked to professor"), indicating some sections are unresolved |

---

## 6. Risks

**R-01 — Funding Type Mismatch (CRITICAL)**  
The workbook is structured for actual costs reimbursement. If the project transitions to a lump sum model, the entire calculation approach, eligibility rules, and reporting structure change fundamentally. This risk must be resolved before design begins.

**R-02 — Hardcoded Exchange Rate**  
The TRY/EUR rate (50.62) is hardcoded in a single cell (`Salary Estimation!C2`). All TRY-denominated salary costs depend on it. Any change in the rate during the project lifetime is not modelled; the rate will need to be a managed, dated input in the software product.

**R-03 — Inflation Assumptions Vary Per Person**  
Different salary raise multipliers are applied to different individuals (20% for PI, 15% for most others, hardcoded per-person). These are not governed by a single parameter, making them easy to overlook or diverge.

**R-04 — Empty and Partially Populated Sheets**  
The "Other costs" sheet is empty. Rows D20–D24 in the Details sheet (Equipment lines 2–5 beyond the first group) have no values. It is unclear whether these omissions are intentional or represent incomplete data entry.

**R-05 — Turkish-Language Content**  
Several labels, notes, and comments are in Turkish. A software product intended for international use will require all content to be in English (or properly internationalised).

**R-06 — WP–Cost Linkage Missing**  
The Gantt Chart maps staff to Work Packages across 60 months, but this mapping is not connected to the cost calculations. There is no per-WP budget breakdown in the current model, which is typically required for ERC reporting.

**R-07 — CFS Budget Line**  
A Certificate on Financial Statements (€12,000) is included in C3 Other Goods. Under ERC rules, CFS eligibility depends on grant size and audit thresholds. This line may need validation against grant amount once totals are finalised.

---

## 7. Unknowns

**U-01 — Total Grant Amount**  
The workbook does not show a completed total (data-only cell values were not computed during this analysis phase). The full requested EU contribution is unknown until formula values are resolved.

**U-02 — Project Title and Research Domain**  
The workbook contains no project title or scientific abstract. The Gantt chart labels WPs 1–5 by number only, with no descriptions. The research domain appears to involve fieldwork in Turkey and India (qualitative research tools like MAXQDA and audio recorders suggest social/linguistic sciences), but this is inferred, not stated.

**U-03 — Collaborator Status**  
"Samarjit Hoca" (Expert, India) and experts from Australia appear in the personnel plan, but their institutional affiliation and cost basis are unclear. Whether they are employed by Ozyegin University or are third-party collaborators (which would affect cost category eligibility) is not specified.

**U-04 — Reporting Period Alignment**  
The Details sheet divides costs across five 12-month periods, but ERC-CoG projects typically have two reporting periods. It is unclear whether the five-period structure is a planning convenience or is intended to match actual reporting milestones.

**U-05 — Industry 4.0 Center Usage**  
The "D. Internally invoiced goods and services" line references the Industry 4.0 center at Ozyegin University but carries no value. The intended cost and justification for this line are unknown.

**U-06 — Scope of "Other costs" Sheet**  
The sheet exists but is empty. It may be intended for future use or may be a residual from a template.

---

## 8. Deliverables Summary

| Item | Status |
|---|---|
| Workbook located and read | ✅ Complete |
| Sheet inventory | ✅ 10 sheets documented |
| Data-flow architecture understood | ✅ Complete |
| Major budgeting domains identified | ✅ 6 domains (A–E + overview) |
| Complexity assessed | ✅ HIGH |
| Risks identified | ✅ 7 risks logged |
| Unknowns flagged | ✅ 6 unknowns logged |
| Formulas documented | ❌ Deferred to TASK-02 |
| Code written | ❌ Not applicable at this phase |

---

## 9. Open Questions (Requiring PI Input Before TASK-02)

1. **Is the funding type Actual Costs or Lump Sum?** This determines the entire architecture of the product.
2. **What are the five Work Package names and descriptions?** Needed for traceability between cost lines and scientific deliverables.
3. **What are the intended reporting periods?** Two periods (standard ERC) or five?
4. **What is the expected total grant budget range?** Helps validate whether a CFS will be required.
5. **Should the product support multiple languages (English + Turkish)?** Several notes and labels are in Turkish.

---

## 10. Recommended Next Step

**AWAIT PI APPROVAL** on this overview and resolution of the Funding Type question (U-01 / R-01) before proceeding to TASK-02 (Excel Analysis).

Once approved: proceed to TASK-02 — document every worksheet, table, named range, cross-sheet dependency, formula, and hardcoded constant in detail.

---

**Confidence Level: 82%**

High confidence on structure, sheet contents, and data flow. Confidence limited by: uncomputed formula totals (no running Excel engine), the funding-type discrepancy, Turkish-language content requiring interpretation, and empty/incomplete sheet sections.
