# ERC Budget Tool — Future Extension Guide

**Version:** 1.0  
**Date:** 2026-07-10  
**Audience:** Engineers adding new features to ERC Budget Tool v1.x or beyond

This guide describes how to extend the application in the most likely directions. Each extension area includes a checklist of files to touch, the design contract to follow, and what tests to add.

---

> ## ⚠ Current Implementation Notes (as of v1.6.0, 2026-07-17)
>
> - **§5 (Adding a New Personnel Role Type) is done** — an "MSc Student" role type was added following exactly this checklist's pattern. Still a good reference for adding another one.
> - **§1 (Adding a New Cost Category)** has two stale file paths: `src-tauri/src/dto/mod.rs` should read `src-tauri/src/domain/dto.rs`, and `src-tauri/src/persistence/project_file.rs` should read `src-tauri/src/persistence/mod.rs` (there's no submodule split). More substantively, its design contract predates Work-Package-based budgeting — a new category today should be **Work-Package-tagged** (`work_package_ids`, split evenly across multiple selections, following the `OtherDirectCostItem` pattern), not year-tagged, and needs a corresponding entry in `calculation/wp_budget.rs`'s per-WP aggregation. `docs/developer-guide.md` §9 has an updated version of this exact checklist reflecting that.
> - **§2 (Adding a New Rate Table Version)** — still directionally correct; `docs/developer-guide.md` §10 and `docs/deployment-guide.md` have the current, more detailed version (including the lesson learned when the originally-shipped rate tables turned out to be fabricated placeholder data rather than the real EU figures — transcribe carefully, script it, and add a spot-check test).
> - **New extension area not covered here at all**: the in-app auto-updater (`docs/developer-guide.md` §14, `docs/deployment-guide.md` in full) — releasing a new version now requires a signed build and an updated `latest.json` manifest, not just building installers.
> - The other sections (export formats, validation rules, multi-partner, i18n, multi-project, multi-currency, schema migration) remain forward-looking and haven't been touched — no divergence to flag, since none of them have been built yet.

---

## Contents

1. Adding a New Cost Category
2. Adding a New Rate Table Version
3. Adding a New Export Format
4. Adding a New Validation Rule
5. Adding a New Personnel Role Type
6. Supporting Multiple Partners (Multi-Institution Projects)
7. Adding Internationalisation (i18n)
8. Multi-Project Dashboard
9. Multi-Currency Support (Beyond TRY)
10. Schema Migration Between Versions

---

## 1. Adding a New Cost Category

**When:** The European Commission introduces a new eligible cost category (e.g., "Category D: Exceptional Costs" under a future ERC programme).

### Design contract

- Every cost category must live in the Rust domain model, not in the frontend.
- Every mutation must return a fully recalculated `BudgetSummaryDto`.
- The new category total must appear in the live dashboard.
- The indirect cost base (`CALC-19`) must be updated to include or exclude the new category according to EU rules.

### Checklist

**Rust backend:**

1. `src-tauri/src/domain/entities.rs` — add the entity struct and add a `Vec<NewItem>` field to `Project`.
2. `src-tauri/src/dto/mod.rs` — add `NewItemInput`, `NewItemDetailDto`. Add `category_d_total: String` to `BudgetSummaryDto`. Add `new_item_detail: Vec<NewItemDetailDto>` to `BudgetSummaryDto`.
3. `src-tauri/src/validation/mod.rs` — add `validate_new_item(dto, project)`. Add `#[cfg(test)]` tests.
4. `src-tauri/src/calculation/budget_aggregator.rs` — add category D summation. Update `CALC-19` indirect base if the category is eligible. Add `#[cfg(test)]` tests.
5. `src-tauri/src/commands/new_category.rs` — implement `add_new_item`, `update_new_item`, `delete_new_item` following the same pattern as `other_costs.rs`.
6. `src-tauri/src/lib.rs` — register new commands in `tauri::generate_handler![]`.
7. `src-tauri/src/persistence/project_file.rs` — serde naturally handles new fields; verify round-trip in tests.
8. `src-tauri/tests/integration_test.rs` — add at least one integration test scenario including a Category D item.

**TypeScript frontend:**

9. `src/types/index.ts` — add `NewItemInput`, `NewItemDetailDto`. Update `BudgetSummaryDto`.
10. `src/validators/schemas.ts` — add `newItemSchema`. Add tests to `src/__tests__/validators.test.ts`.
11. `src/ipc/commands.ts` — add `addNewItem`, `updateNewItem`, `deleteNewItem`.
12. `src/screens/NewCategory.tsx` — new screen with item list and item form.
13. `src/components/NewItemCard.tsx` — card component for the item list.
14. `src/App.tsx` — add the new screen to the wizard sequence.
15. `src/components/CategoryTotalsPanel.tsx` — add Category D row.
16. `src/components/BudgetRingChart.tsx` — add Category D segment.
17. `src/export/excelExporter.ts` — add Category D rows to Overview and Detail sheets.
18. `src/export/pdfExporter.ts` — add Category D to the PDF summary table.

---

## 2. Adding a New Rate Table Version

**When:** The European Commission publishes a new Annex 2a/2b update.

This is the lowest-effort extension. No code changes are required — only a new JSON file.

### Checklist

1. Create `src-tauri/resources/eu_travel_rates/v_from_YYYY_MM_DD.json` following the existing schema (see Architecture Guide §8).
2. Add the file to the `include_str!` array in `src-tauri/src/persistence/rate_data.rs`.
3. Add at least one test in `src-tauri/tests/integration_test.rs` that uses a known country's rates from the new version and asserts the exact expected trip cost.
4. Run `cargo test` — the `RateData` parser will catch any JSON schema errors immediately.

The `get_rate_versions` IPC command automatically returns all loaded versions. The frontend dropdown and auto-selection logic require no changes.

---

## 3. Adding a New Export Format

**When:** You need to support a new output file type (e.g., XML for institutional finance systems, DOCX for a word processor report, or JSON for API upload).

### Design contract

- All export functions receive `BudgetSummaryDto` (and optionally the full `ProjectDto`) as their only input.
- Export functions write to a user-chosen file path via Tauri's `tauri-plugin-fs`.
- Export functions are pure TypeScript — no Rust changes needed.
- Export functions must be tested in isolation by providing a mock `BudgetSummaryDto`.

### Checklist

1. `src/export/newFormatExporter.ts` — implement the export function:
   ```typescript
   export async function exportToNewFormat(
     summary: BudgetSummaryDto,
     project: ProjectDto,
     path: string,
   ): Promise<void> { ... }
   ```
2. `src/screens/ReviewExport.tsx` — add an export button that:
   - Opens a save dialog via `save()` from `@tauri-apps/plugin-dialog` with the appropriate file filter.
   - Calls the export function.
   - Shows a success toast or error banner.
3. Add unit tests for the new exporter by mocking `BudgetSummaryDto` and asserting the generated string/buffer content.

---

## 4. Adding a New Validation Rule

**When:** A new EU programme requirement introduces a new constraint (e.g., the PI's FTE must be at least 30% across the project, or subcontracting cannot exceed 30% of total eligible costs).

### Design contract

- All business-rule validation lives in Rust (`src-tauri/src/validation/mod.rs`).
- Field-level format validation lives in TypeScript Zod schemas (`src/validators/schemas.ts`).
- Validation error codes are machine-readable strings in SCREAMING_SNAKE_CASE.
- Every new validation rule has at least two `#[cfg(test)]` test cases: one where it passes, one where it fails with the specific error code.

### Checklist

1. `src-tauri/src/validation/mod.rs` — add the rule to the appropriate validator function:
   ```rust
   if project.subcontracting.amount_eur > total_eligible * dec!(0.30) {
       errors.push(FieldError::entity("SC_EXCEEDS_30_PCT",
           "Subcontracting cannot exceed 30% of total eligible costs."));
   }
   ```
2. Add `#[cfg(test)]` tests using `has_field_error()` or `has_entity_error()`.
3. If the rule involves a cross-entity check (like subcontracting vs. total), the validator will need to accept additional inputs (`&BudgetSummary` or computed totals). Pass these from the command handler.
4. `src/types/index.ts` — add the new error code as a string literal union type if the frontend needs to handle it specially (e.g., to highlight a specific field).
5. `src/screens/*.tsx` — map the new error code to a field-level error message in the relevant form.
6. `src/__tests__/validators.test.ts` — if the new rule also appears in the Zod schema, add a test for the failing case.

---

## 5. Adding a New Personnel Role Type

**When:** The grant programme introduces a new staffing category (e.g., "Technician" or "Visiting Professor").

### Checklist

1. `src-tauri/src/domain/entities.rs` — add the new variant to `RoleType`:
   ```rust
   pub enum RoleType {
       Pi,
       PostDoc,
       Expert,
       Admin,
       Technician,   // NEW
   }
   ```
2. Update any `match` statement that exhaustively covers `RoleType` — the Rust compiler will produce a compile error for every missing arm.
3. `src-tauri/src/validation/mod.rs` — update the PI uniqueness check if the new type should also be unique.
4. `src/validators/schemas.ts` — add `'Technician'` to the `roleType` Zod enum.
5. `src/types/index.ts` — add `'Technician'` to the `RoleType` union type.
6. `src/screens/Personnel.tsx` — add `Technician` to the role type dropdown options.
7. Add tests: one integration test that includes a Technician role and asserts the salary is included in Category A.

---

## 6. Supporting Multiple Partners (Multi-Institution Projects)

**When:** The tool needs to support consortia where multiple universities each have their own staff and budgets (relevant for ERC Synergy Grants or Proof of Concept grants with multiple beneficiaries).

This is a larger structural extension.

### Design considerations

- Each partner is an institution with its own currency, exchange rate, and inflation assumptions.
- Personnel roles, equipment, trips, and C3 costs belong to a partner.
- The budget summary aggregates at both the partner level and the project level.
- The export must produce per-partner budget tables plus a consolidated project table.

### Domain model extension

```rust
pub struct Partner {
    pub id: Uuid,
    pub name: String,
    pub country_code: String,
    pub currency_code: String,    // e.g. "PLN", "SEK", "TRY"
    pub currency_eur_rate: Decimal,
    pub default_inflation_rate_pct: Decimal,
    pub personnel_roles: Vec<PersonnelRole>,
    pub equipment_items: Vec<EquipmentItem>,
    pub trips: Vec<Trip>,
    pub other_cost_items: Vec<OtherDirectCostItem>,
    pub subcontracting: Subcontracting,
}

pub struct Project {
    pub id: Uuid,
    pub config: ProjectConfig,
    pub partners: Vec<Partner>,    // replaces direct cost lists
}
```

### Checklist (high level)

1. Extend the domain entities as above.
2. Move `PersonnelRole`, `EquipmentItem`, etc. inside `Partner` (the `Project` no longer owns them directly).
3. Update all command handlers to accept a `partner_id` parameter.
4. Update `budget_aggregator.rs` to aggregate per-partner first, then project-wide.
5. Update `BudgetSummaryDto` to include per-partner breakdowns.
6. Update the frontend to show a partner selector and per-partner summaries.
7. Update the export to produce per-partner sheets in the Excel workbook.
8. Extend the `.ercbudget` format version to `"2.0"` and write a migration function for v1.0 files (single-partner projects).

---

## 7. Adding Internationalisation (i18n)

**When:** The application needs to support UI languages other than English (e.g., Turkish, French, German).

### Recommended approach

Use `react-i18next` for UI string translation. All strings displayed to the user should be looked up via the `t()` function. Numbers and currencies should use the browser's `Intl.NumberFormat` API.

### Checklist

1. Install: `npm install react-i18next i18next`.
2. Create `src/locales/en.json` with all UI strings as key-value pairs.
3. Create `src/locales/tr.json` with Turkish translations.
4. Wrap the app in an `I18nextProvider` in `src/main.tsx`.
5. Replace all hardcoded strings in components with `t('key')` calls.
6. Add a language selector in the application settings (a new Settings screen, or a menu item).
7. Persist the selected language in the `.ercbudget` file or in Tauri's app data storage.

**Note:** validation error messages returned from Rust are English strings. If you want translated validation messages, either: (a) translate them by error code on the TypeScript side, or (b) pass a `locale` parameter to the Rust commands and return localised strings from the Rust validators.

---

## 8. Multi-Project Dashboard

**When:** Users want an overview screen that shows all their `.ercbudget` files in one place — a project gallery with summary cards, search, and sorting.

### Design considerations

- The dashboard is a screen shown before a project is opened (from the Welcome screen, or a dedicated "Projects" tab).
- Each card shows: project title, PI name, duration, total eligible costs, last modified date.
- The list is populated by scanning a user-configured folder, or by reading a recents list.
- Opening a project card calls the existing `load_project` command.

### Checklist

1. `src-tauri/src/commands/dashboard.rs` — add:
   ```rust
   #[tauri::command]
   pub fn list_recent_projects() -> Vec<ProjectSummaryDto>
   
   #[tauri::command]
   pub fn scan_folder_for_projects(path: String) -> Vec<ProjectSummaryDto>
   ```
   `ProjectSummaryDto` contains: `path`, `title`, `pi_name`, `total_eligible_costs`, `updated_at`.
   Each command opens each `.ercbudget` file, deserialises only the top-level fields, and returns without loading the full project into memory.
2. Persist the recents list in Tauri's app data directory (a small JSON file, not the `.ercbudget` file).
3. `src/screens/Dashboard.tsx` — new screen showing project cards.
4. `src/App.tsx` — add the Dashboard screen to the navigation flow.
5. Update the Welcome screen to show "Recent Projects" instead of just "Open Project".

---

## 9. Multi-Currency Support (Beyond TRY)

**When:** The institution is in a non-TRY country and salaries are denominated in a different currency (PLN, SEK, CZK, etc.).

### Design considerations

The current model has a single `try_eur_rate` field. A multi-currency extension replaces this with a per-role currency setting.

### Checklist

1. `src-tauri/src/domain/entities.rs` — add `currency_code: String` and `currency_eur_rate: Decimal` to `PersonnelRole` (or to `ProjectConfig` if all roles share the same currency).
2. `src-tauri/src/calculation/salary_projection.rs` — replace the hardcoded TRY field reference with the generic `currency_eur_rate` from the role.
3. `src-tauri/src/validation/mod.rs` — add validation: `currency_code` must be a valid ISO 4217 code; `currency_eur_rate` must be > 0.
4. `src/validators/schemas.ts` — add `currencyCode` and `currencyEurRate` fields to `personnelRoleSchema`.
5. `src/screens/Personnel.tsx` — replace the global TRY rate display with a per-role currency picker and rate field.
6. Update all tests to use the generic rate field instead of the TRY-specific one.
7. Update `BudgetSettings.tsx` — the global `try_eur_rate` field becomes a default rate (still useful when all roles are TRY-denominated) with a note that it can be overridden per role.

---

## 10. Schema Migration Between Versions

**When:** A new application version changes the `.ercbudget` JSON format in a way that makes old files incompatible.

### Design contract

- The `.ercbudget` file always contains `"format_version": "X.Y"`.
- The Persistence Layer reads the version on load and applies a migration chain to bring the data up to the current schema.
- Migration functions are pure: they take a `serde_json::Value` and return a transformed `serde_json::Value`. No domain entities are involved.
- Each migration is tested with a minimal fixture file representing the old format.

### Implementation pattern

```rust
// src-tauri/src/persistence/project_file.rs

const CURRENT_VERSION: &str = "2.0";

pub fn load_project(path: &str) -> Result<Project, AppError> {
    let raw = std::fs::read_to_string(path)?;
    let mut json: serde_json::Value = serde_json::from_str(&raw)?;

    let version = json["format_version"].as_str().unwrap_or("1.0");
    json = apply_migrations(json, version)?;

    let project: Project = serde_json::from_value(json["project"].clone())?;
    Ok(project)
}

fn apply_migrations(
    mut json: serde_json::Value,
    from_version: &str,
) -> Result<serde_json::Value, AppError> {
    if from_version == "1.0" {
        json = migrate_1_0_to_2_0(json)?;
    }
    // Add future migrations here:
    // if current_version == "2.0" { json = migrate_2_0_to_3_0(json)?; }
    Ok(json)
}

fn migrate_1_0_to_2_0(mut json: serde_json::Value) -> Result<serde_json::Value, AppError> {
    // Example: v1.0 had a single "try_eur_rate" in config;
    // v2.0 moves it into each personnel role as "currency_eur_rate".
    let try_eur_rate = json["project"]["config"]["try_eur_rate"].clone();
    if let Some(roles) = json["project"]["personnel_roles"].as_array_mut() {
        for role in roles.iter_mut() {
            role["currency_code"] = serde_json::json!("TRY");
            role["currency_eur_rate"] = try_eur_rate.clone();
        }
    }
    json["format_version"] = serde_json::json!("2.0");
    Ok(json)
}
```

### Checklist for a new migration

1. Increment the `CURRENT_VERSION` constant.
2. Write `migrate_X_Y_to_A_B(json)` — pure JSON transformation, no domain types.
3. Add the migration call to `apply_migrations()`.
4. Write a test that loads a minimal v1.x fixture JSON string and asserts the migrated output has the expected v2.x structure.
5. Bump `"format_version"` in all test fixture files used in `integration_test.rs`.
6. Update the User Manual to note that old files are automatically migrated on first open.
