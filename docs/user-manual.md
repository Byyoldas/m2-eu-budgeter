# ERC Budget Tool — User Manual

**Version:** 1.0  
**Applies to:** ERC Budget Tool v1.0 (Tauri desktop application)  
**Audience:** Proposal writers, PI assistants, research grant managers  
**Date:** 2026-07-10

---

> ## ⚠ Current Implementation Notes (as of v1.6.0, 2026-07-17)
>
> The app is now called **M2-EU Budgeter** (renamed from "ERC Budget Tool"). The step order, dashboard, and CFS/save/export sections of this manual are still accurate. The following field-level details have changed:
>
> - **Step 4 (Personnel)**: "Active Years" is now a **Start Month / End Month** range instead of a year checklist. "Work Packages" is no longer a field on this form at all — the person's cost is now split across Work Packages **automatically**, based on which WPs' timelines overlap their Start/End Month; the resulting per-WP breakdown is shown read-only. Role Type also gained an **MSc Student** option alongside PI/PostDoc/Expert/Admin.
> - **Step 5 (Equipment)**: "Year of Purchase" no longer exists. Work Package is now **required** (not optional) and single-select — pick the one WP the purchase is charged to.
> - **Step 6 (Travel)**: "Project Year" no longer exists. Work Package is now **required** (at least one; cost splits evenly if you tag more than one). The Distance Calculator link now opens in your regular web browser rather than inside the app. The specific flight-band EUR figures in this section's example are outdated — the real EU Annex 2a/2b rates (flight, accommodation, subsistence) were fixed in v1.4.0; the app now shows the correct current figures, this document just hasn't been re-derived from them.
> - **Step 7 (Other Direct Costs)**: "Project Year" no longer exists. Work Package is now **required** (at least one). Items you've added can now be properly **edited** — click Edit on any item to change its name, amount, notes, or Work Package(s).
> - **Not covered here**: an in-app updater. The Welcome screen has a "Check for Updates" button, and the app also checks silently in the background on launch — if a newer version is available, a popup offers to download and install it automatically.
>
> This document otherwise reflects the app reasonably well for a proposal writer's day-to-day use — the changes above are the ones that would actually confuse a first-time user following it step by step.

---

## Contents

1. Introduction
2. Installation
3. First Launch — The Welcome Screen
4. Step-by-step Wizard
   - Step 1: Project Setup
   - Step 2: Budget Settings
   - Step 3: Work Packages
   - Step 4: Personnel
   - Step 5: Equipment
   - Step 6: Travel
   - Step 7: Other Direct Costs
   - Step 8: Review & Export
5. The Live Budget Dashboard
6. Certificate on Financial Statements (CFS)
7. Saving and Opening Projects
8. Exporting the Budget
9. Common Warnings and What They Mean
10. Tips and Best Practices

---

## 1. Introduction

The ERC Budget Tool is a desktop application for building lump-sum budget proposals for European Research Council (ERC) grants, in particular Consolidator Grants (CoG) using the **Actual Costs** funding model. It replaces the complex Excel workbook that researchers traditionally use, hiding all spreadsheet complexity behind a guided wizard.

**What the tool does for you:**

- Converts Turkish Lira salaries to EUR and applies year-by-year inflation automatically.
- Calculates equipment depreciation (with the EU cap rule applied).
- Looks up official EU Annex 2a/2b accommodation and subsistence rates for your destination countries.
- Computes Category E indirect costs (25% of eligible direct costs) automatically.
- Tracks the €430,000 CFS threshold and prompts you when a Certificate on Financial Statements is required.
- Exports a formatted Excel workbook, PDF summary, and CSV file ready for your budget justification.

**What you do NOT need to do:**

- Enter formulas.
- Calculate totals by hand.
- Know which Annex applies to your call date.
- Know the depreciation cap rule.

---

## 2. Installation

**macOS:**

1. Open the `.dmg` file you downloaded.
2. Drag the ERC Budget Tool icon into your Applications folder.
3. On first launch, right-click the app and choose **Open** (macOS Gatekeeper requires this once for apps from outside the App Store).

**Windows:**

1. Run the `.msi` installer.
2. Follow the installer prompts (no admin rights required for per-user install).
3. The app appears in your Start menu as **ERC Budget Tool**.

**System requirements:**

| Platform | Minimum |
|---|---|
| macOS | 12 Monterey or later |
| Windows | Windows 10 (64-bit) or later |
| RAM | 512 MB (4 GB recommended) |
| Disk | 50 MB |
| Network | Not required — the app works fully offline |

---

## 3. First Launch — The Welcome Screen

When you first open the app, you see the Welcome screen with two options:

**New Project** — starts a fresh budget from scratch. The wizard opens at Step 1 (Project Setup).

**Open Project** — loads an existing `.ercbudget` file from your computer. The wizard reopens at the Review & Export screen with all your previously entered data.

The left panel always shows the wizard steps; the right panel shows the live budget dashboard. Both panels update as you enter data.

---

## 4. Step-by-step Wizard

Work through the wizard in order. You can return to any previous step at any time using the left-panel navigation — your data is preserved.

---

### Step 1 — Project Setup

This step establishes the skeleton of your budget. All other steps depend on it.

**Project Title** — used in the export header. Does not affect calculations.

**Principal Investigator Name** — used in the export header. Does not affect calculations.

**Grant Call Reference** — e.g., `ERC-2025-CoG-123456`. Used in the export header. Does not affect calculations.

**Project Duration (years)** — how many full years your grant covers. Must be between 1 and 7. Default: 5.

> If you change the duration after adding costs, entries in years outside the new range are excluded from totals but not deleted.

**Number of Work Packages** — how many Work Packages (WPs) your project has. Between 1 and 10. Default: 5. WP names are optional and can be entered in Step 3.

**Grant Call Opening Date** — the date when your ERC call officially opened. The app uses this to automatically select the correct EU travel rate version (Annex 2a/2b). If you are unsure, leave it blank and select the rate version manually in Step 2.

Click **Next** once all required fields are complete.

---

### Step 2 — Budget Settings

These three financial parameters apply across your entire budget.

**Default Annual Salary Inflation Rate (%)** — the expected year-on-year salary increase for personnel in Turkey. This pre-fills the inflation rate for every new personnel role you add; you can override it per role. Typical values: 10–40%.

> This does not apply to flat-amount trips or equipment.

**TRY / EUR Exchange Rate** — how many Turkish Lira equal one Euro at the time of budget preparation (e.g., `38.50`). Applied uniformly to all TRY-denominated salaries for the entire project duration. Check the European Central Bank reference rate for the applicable date.

**Indirect Cost Rate (%)** — the ERC standard overhead rate. **Leave this at 25%** unless your institution has a different agreed rate. The application shows a warning if you change it, because deviating from 25% must be justified in your proposal.

**EU Travel Rate Version** — automatically selected if you entered a Call Opening Date in Step 1. You can also select it manually:

| Version label | Use when call opened |
|---|---|
| Before 31 July 2024 | Call opened before 2024-07-31 |
| 31 July 2024 – 12 May 2025 | Call opened between those dates |
| From 13 May 2025 | Call opened on or after 2025-05-13 |

If in doubt, select "From 13 May 2025" (the current version).

---

### Step 3 — Work Packages

Each Work Package slot created in Step 1 is listed here. Entering names is optional but recommended — names appear in the export and help you track cost allocation by WP later.

**WP Name** — a short label, e.g., `WP1: Literature Review`. Leave blank to use the default `WP-1` label.

---

### Step 4 — Personnel

Add each person (or role) whose salary is charged to the grant. Click **Add Role** to open the role form.

**Role Type** — choose one:

| Type | Notes |
|---|---|
| PI (Principal Investigator) | Only one PI is allowed per project |
| PostDoc | Postdoctoral researcher |
| Expert | Technical expert or research assistant |
| Admin | Administrative support |

**Role Label** — a unique name for this person within the project, e.g., `PostDoc-1` or `Research Assistant – Chemistry`. Must be unique across all roles.

**Current Monthly Salary (TRY)** — the gross monthly salary in Turkish Lira at the start of the project. The app converts this to EUR and applies inflation year by year automatically.

**FTE (Full-Time Equivalent)** — the fraction of this person's time charged to the grant. Enter as a decimal between 0 and 1. For example, 0.5 means 50% of their time. Full-time = 1.0.

**Annual Inflation Rate (%)** — pre-filled from Step 2; override here if this person's salary grows at a different rate. Enter 0 for no inflation.

**Active Years** — tick the years in which this person works on the grant. A person active in Year 1 only but not Year 2 will have their cost charged only in Year 1. If active, the application always uses 12 months per year.

**Work Packages** — optionally tick the WPs this person contributes to (for cost allocation visibility only; does not affect totals).

As you fill in the form, the **Live Preview** box at the bottom of the form shows the projected cost per year in real time.

After saving, the role appears as a card in the personnel list. Click **Edit** on any card to modify it; click **Delete** to remove it.

> All personnel costs roll up to **Category A** in the budget dashboard.

---

### Step 5 — Equipment

Add equipment items costing more than €1,000. Items below that threshold are generally classified as consumables (Other Direct Costs, Step 7). Click **Add Equipment** to open the form.

**Name** — a descriptive label, e.g., `High-Performance Laptop`.

**Purchase Cost (EUR)** — the full purchase price in euros.

**Useful Lifetime (months)** — the expected total economic life of the item. For example, a laptop with a 3-year life = 36 months. Common values:

| Item type | Typical lifetime |
|---|---|
| Laptop / computer | 36 months |
| Lab instrument | 60–84 months |
| Server | 48–60 months |
| Audio recorder | 36 months |

**Usage Percentage (%)** — what percentage of the item's use is dedicated to the grant. Enter 100 if it is used exclusively for the project.

**Usage Duration (months)** — how many months of the item's life fall within the grant period.

**Year of Purchase** — optional. The project year in which the item is purchased. Used for allocation tracking only; does not affect the depreciation calculation.

**The depreciation cap rule** — the application automatically applies the EU rule:

> Eligible depreciation = min( (cost ÷ lifetime) × usage% × usage months, cost × usage% )

In plain terms: you cannot claim more than the usage-weighted cost of the item, even if the depreciation formula would produce a higher number. The Live Preview shows whether the cap is being applied (a "capped" badge appears on the result).

> All equipment costs roll up to **Category C2**.

---

### Step 6 — Travel

Add each type of trip. If many trips share the same itinerary, enter one trip with the number of instances. Click **Add Trip** to open the form.

**Trip Name** — a short label, e.g., `Annual Consortium Meeting – Vienna` or `India Fieldwork`.

**Trip Type** — choose one:

**Itemized** — the app calculates the full cost from EU Annex 2a/2b unit costs. Use this for most international trips. Requires:

- **Destination Country** — select from the dropdown; the app loads the official accommodation and subsistence daily rates for that country.
- **One-way Distance (km)** — the one-way distance from your departure city to the destination. The app maps this to the correct EU flight band automatically.
  - Under 400 km → no flight cost (assumed surface transport).
  - 400–999 km → Band F-01, and so on up to Band F-09 (>= 10,000 km).
- **Number of Nights** — how many nights accommodation per trip instance.
- **Number of Days** — how many days of subsistence (meals, local transport) per trip instance.
- **Domestic/Local Transport (EUR)** — any additional local transport cost per trip instance not covered by the flat rates (e.g., airport taxi, train within the country). Enter 0 if none.

**Flat Amount** — you already know the total cost per trip. Enter:

- **Amount per Instance (EUR)** — the all-in cost per single trip.

Both types then require:

- **Project Year** — which year this trip takes place.
- **Number of Instances** — how many times this trip occurs (e.g., 4 for a quarterly fieldwork trip).
- **Work Package** — optional tagging for WP allocation.

The Live Preview shows the computed cost per instance and total cost.

> All travel costs roll up to **Category C1**.

---

### Step 7 — Other Direct Costs

Category C3 covers direct costs that do not fit in Personnel, Equipment, or Travel. Typical examples: publication fees, research consumables, workshop registration fees, software licences, data management costs.

Click **Add Cost Item** to open the form.

**Name** — a descriptive label, e.g., `Open Access Publication Fees – Y1`.

**Amount (EUR)** — the total cost of this item.

**Project Year** — the year in which this cost is incurred.

**CFS Item** — tick this box if this item would be verified in a Certificate on Financial Statements audit. Typically ticked for large third-party costs where you have a formal invoice trail. See Section 6 for details on CFS.

**Notes** — optional free-text justification that appears in the export.

**Work Package** — optional.

> All C3 items roll up to **Category C3**.

---

### Step 8 — Review & Export

The Review screen shows a full read-only summary of every cost line in your budget. Use it to verify totals before generating the export files.

**Sections shown:**

- Category A: Personnel (per-role breakdown by year)
- Category B: Subcontracting (if entered)
- Category C1: Travel (per-trip total)
- Category C2: Equipment (per-item total, with cap indicator)
- Category C3: Other Direct Costs (per-item total)
- Category E: Indirect Costs (= 25% × (A + C1 + C2 + C3))
- Total Direct Costs (A + B + C1 + C2 + C3)
- Total Eligible Costs (Direct + E)
- Requested EU Contribution (= Total Eligible Costs, because ERC funds 100%)

**Subcontracting (Category B)** — if your project has subcontracted work, enter the total subcontracting amount here. Note that Category B is excluded from the indirect cost base (subcontracting costs do not attract overhead).

**CFS Status banner** — if your Requested EU Contribution exceeds €430,000, a red banner appears. See Section 6.

**Export buttons** — generate Excel, PDF, or CSV files. See Section 8.

---

## 5. The Live Budget Dashboard

The right panel updates every time you save a cost item. It shows:

**Ring chart** — proportional breakdown of your total budget by category (A, B, C1, C2, C3, E). Hover over a segment to see the category name and EUR amount.

**Year bar chart** — total eligible costs broken down by project year. Useful for spotting uneven budget distributions.

**Category Totals panel** — each category total in EUR, plus the running grand total and requested EU contribution.

The dashboard updates within a fraction of a second after you save any change. You do not need to navigate to the Review screen to see updated totals.

---

## 6. Certificate on Financial Statements (CFS)

If your **Requested EU Contribution exceeds €430,000**, EU grant rules require that your institution produces a Certificate on Financial Statements (CFS) at project close. The CFS is an independent audit of the costs you claim.

**What the application does:**

- Monitors your total as you enter costs.
- When the total exceeds €430,000, a red warning banner appears at the top of every screen.
- On the Review & Export screen, a CFS checklist prompt asks you to confirm awareness of the requirement.
- Once you tick the confirmation checkbox, the banner changes from red to amber (acknowledged).
- The CFS status is saved with your project file.

**CFS does not affect the budget calculation.** It is purely an administrative reminder. Your totals and export files are not changed by CFS status.

**The three states:**

| Status | Meaning |
|---|---|
| Not Required | Your total is ≤ €430,000 |
| Required — Unacknowledged | Total > €430,000; you have not yet confirmed awareness |
| Required — Acknowledged | Total > €430,000; you have confirmed in the checklist |

---

## 7. Saving and Opening Projects

**Auto-save** — the application auto-saves your project to a temporary file after every change. If the app closes unexpectedly, your data is not lost.

**Save As** — use **File → Save As** (or the **Save** button on the Review screen) to save a named `.ercbudget` file to your computer. Choose a descriptive name such as `ERC-CoG-2025-Budget-v3.ercbudget`.

**Open** — use **File → Open** to load any `.ercbudget` file. The file format is human-readable JSON, so it can also be opened in any text editor if needed.

**Where to keep your file** — treat the `.ercbudget` file like any other document. Back it up using your institution's file storage or cloud drive. The file is self-contained: it includes all cost entries and budget settings.

---

## 8. Exporting the Budget

From the Review & Export screen, click one of the three export buttons:

**Export to Excel (.xlsx)** — generates a formatted Excel workbook with three sheets:

- *Overview* — one-page budget table in ERC submission format (Categories A–E, totals).
- *Budget by Year* — per-year breakdown (years as columns, categories as rows).
- *Detail* — itemised list of every cost line.

**Export to PDF** — generates a formatted one-page PDF summary. Suitable for attaching to a printed proposal or sharing with a co-investigator.

**Export to CSV** — generates a flat comma-separated file with all budget lines. Useful for importing into an institutional finance system or a different spreadsheet.

All three exports use the same calculated values shown on the Review screen.

---

## 9. Common Warnings and What They Mean

**"Only one PI is allowed per project"** — you have added a second role with type PI. Change the role type of the second person to Expert or PostDoc.

**"Inflation rate must be between 0% and 100%"** — the value you entered is outside the accepted range. A value of 0 means no inflation; 100 means salaries double each year.

**"FTE must be between 0 and 1"** — the fraction of time must be expressed as a decimal. Enter 0.75 for 75%, not 75.

**"Active years include a year outside the project duration"** — you ticked a year (e.g., Year 6) that does not exist in a 5-year project. Adjust the active years selection.

**"No flight cost for distances under 400 km"** — this is not an error; it is a notice. Trips under 400 km do not attract a flight unit cost under EU rules. Only accommodation and subsistence apply.

**"Depreciation is capped"** — the EU rule limits eligible equipment depreciation to the usage-weighted cost. The Live Preview shows both the theoretical and capped values. The capped amount is used in your totals.

**"Indirect rate deviates from the ERC standard (25%)"** — you changed the Indirect Cost Rate away from 25%. This is permitted but requires justification. The warning is a reminder to document your reason in the budget justification narrative.

**"CFS Required"** (red banner) — your Requested EU Contribution has exceeded €430,000. See Section 6.

---

## 10. Tips and Best Practices

**Start with duration and work packages.** All other inputs depend on project duration. Set it correctly before adding any costs; changing it later can shift year allocations.

**Use the Live Preview before saving.** Every form shows a real-time cost preview as you type. Check it before clicking Save — it confirms the formula is working as expected with your inputs.

**Enter one role per real person.** If the same person will work on the project at different FTE fractions in different phases, create two roles for them (e.g., `PostDoc-1 Phase A` and `PostDoc-1 Phase B`) with different active year selections.

**For travel, use the Itemized type wherever possible.** The Flat Amount type is convenient but loses the itemised rate breakdown in the export. Reviewers often expect to see the Annex 2a/2b rates applied explicitly.

**Check the ring chart after each step.** An unexpectedly large category slice is often a sign of a data entry error (e.g., monthly salary entered as annual, or usage months entered as full project duration).

**Export early and often.** Generate a draft Excel file after completing Personnel and again after completing Travel. Share with your co-PI or grants officer for a review before the budget is final.

**Back up your `.ercbudget` file.** The auto-save is a safety net, not a backup. Copy your file to a shared drive after each session.
