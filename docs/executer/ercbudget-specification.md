# .ercbudget File Specification

**Phase 03 — .ercbudget File Specification**
**Project:** Horizon Europe Project Management Platform
**Date:** 2026-08-03

---

## 1. Purpose

This document provides the complete technical specification of the `.ercbudget` file format — the on-disk representation of an ERC budget project. It covers the current format (v1.0), its internal structure, versioning strategy, serialization rules, validation, and the forward-compatible extension design that allows the Project Execution Application to enrich these files without breaking the Budget Application.

---

## 2. Current Format — Version 1.0

### 2.1 Overview

`.ercbudget` files are **UTF-8 encoded JSON** with a `.ercbudget` extension. They are human-readable and can be opened in any text editor. They contain no binary data.

The file is structured as a thin **envelope wrapper** around the serialized `Project` entity.

### 2.2 Top-Level Envelope

```json
{
  "format_version": "1.0",
  "created_at": "2025-06-15T09:22:41.000Z",
  "updated_at": "2025-11-03T14:07:55.000Z",
  "project": { ... }
}
```

| Field | Type | Description |
|---|---|---|
| `format_version` | string | Semver-like version string. Current value: `"1.0"` |
| `created_at` | string | ISO 8601 UTC timestamp. Set once on first save; never updated. |
| `updated_at` | string | ISO 8601 UTC timestamp. Updated on every save. |
| `project` | object | Full serialized `Project` entity (see §2.3) |

### 2.3 Project Object

```json
{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "config": { ... },
  "personnel_roles": [ ... ],
  "equipment_items": [ ... ],
  "trips": [ ... ],
  "other_cost_items": [ ... ],
  "subcontracting": { ... },
  "cfs_warning_dismissed": false
}
```

| Field | Type | Description |
|---|---|---|
| `id` | string (UUID v4) | Unique project identifier |
| `config` | object | `ProjectConfig` (see §2.4) |
| `personnel_roles` | array | Zero or more `PersonnelRole` objects (see §2.5) |
| `equipment_items` | array | Zero or more `EquipmentItem` objects (see §2.6) |
| `trips` | array | Zero or more `Trip` objects (see §2.7) |
| `other_cost_items` | array | Zero or more `OtherDirectCostItem` objects (see §2.8) |
| `subcontracting` | object | `Subcontracting` (see §2.9) |
| `cfs_warning_dismissed` | boolean | True if user clicked "Remind Me Later" on CFS modal |

### 2.4 ProjectConfig Object

```json
{
  "project_title": "Neural Plasticity in Aging Brains",
  "pi_name": "Prof. Ayşe Kaya",
  "call_reference": "ERC-2025-CoG",
  "duration_years": 5,
  "work_package_count": 3,
  "work_package_names": ["Data Collection", "Analysis", "Dissemination"],
  "work_package_start_months": [1, 7, 49],
  "work_package_end_months": [60, 60, 60],
  "default_inflation_rate_pct": "20.00",
  "try_eur_rate": "50.62",
  "indirect_cost_rate_pct": "25.00",
  "rate_version_id": "v_from_2025_05_13",
  "call_opening_date": "2025-10-15"
}
```

| Field | Type | Constraints | Notes |
|---|---|---|---|
| `project_title` | string | Non-empty | Display only |
| `pi_name` | string | Non-empty | Display only |
| `call_reference` | string | Non-empty | e.g., `"ERC-2025-CoG"` |
| `duration_years` | integer | 1–7 | |
| `work_package_count` | integer | 1–10 | Must equal length of all WP arrays |
| `work_package_names` | array of string\|null | Length = `work_package_count` | |
| `work_package_start_months` | array of integer | 1-indexed, ≥ 1; `#[serde(default)]` | Empty = backward-compatible |
| `work_package_end_months` | array of integer | 1-indexed, inclusive; `#[serde(default)]` | Empty = backward-compatible |
| `default_inflation_rate_pct` | string (Decimal) | 0–100 | Stored as decimal string |
| `try_eur_rate` | string (Decimal) | > 0 | TRY per 1 EUR |
| `indirect_cost_rate_pct` | string (Decimal) | 0–50 | ERC rule: max 50% |
| `rate_version_id` | string | Must match embedded rate table | |
| `call_opening_date` | string\|null | ISO 8601 date (YYYY-MM-DD) | Determines rate version |

### 2.5 PersonnelRole Object

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "role_label": "PI",
  "role_type": "Pi",
  "current_monthly_salary_try": "227900.00",
  "fte_fraction": "0.70",
  "inflation_rate_pct": "20.00",
  "start_month": 1,
  "end_month": 60
}
```

| Field | Type | Constraints |
|---|---|---|
| `id` | string (UUID v4) | |
| `role_label` | string | Unique within project (case-insensitive) |
| `role_type` | enum | `"Pi" \| "Expert" \| "PostDoc" \| "PhdStudent" \| "MscStudent" \| "Admin"` |
| `current_monthly_salary_try` | string (Decimal) | > 0 |
| `fte_fraction` | string (Decimal) | (0, 1] |
| `inflation_rate_pct` | string (Decimal) | [0, 100] |
| `start_month` | integer | 1-indexed; ≥ 1 |
| `end_month` | integer | ≥ `start_month`; ≤ `duration_years × 12` |

### 2.6 EquipmentItem Object

```json
{
  "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "name": "High-performance Laptop",
  "purchase_cost_eur": "2500.00",
  "useful_lifetime_months": 48,
  "grant_usage_pct": "100.00",
  "grant_usage_months": 55,
  "work_package_id": 1
}
```

| Field | Type | Constraints |
|---|---|---|
| `id` | string (UUID v4) | |
| `name` | string | Non-empty |
| `purchase_cost_eur` | string (Decimal) | > 0 |
| `useful_lifetime_months` | integer | ≥ 1 |
| `grant_usage_pct` | string (Decimal) | (0, 100] |
| `grant_usage_months` | integer | ≥ 1 |
| `work_package_id` | integer | 1 ≤ x ≤ `work_package_count` |

### 2.7 Trip Object

```json
{
  "id": "6ba7b811-9dad-11d1-80b4-00c04fd430c8",
  "name": "ERC Project Meeting — Brussels",
  "trip_type": {
    "Itemized": {
      "destination_country_code": "BE",
      "one_way_distance_km": 2800,
      "number_of_nights": 2,
      "number_of_days": 3,
      "domestic_transport_per_instance_eur": "0"
    }
  },
  "number_of_instances": 4,
  "work_package_ids": [1, 2]
}
```

**Flat Amount variant:**
```json
{
  "trip_type": {
    "FlatAmount": {
      "flat_amount_per_instance_eur": "850.00"
    }
  }
}
```

| Field | Type | Notes |
|---|---|---|
| `trip_type` | tagged enum | `"Itemized"` or `"FlatAmount"` as the key |
| `number_of_instances` | integer | ≥ 1 |
| `work_package_ids` | array of integer | Non-empty; each ≥ 1 and ≤ `work_package_count` |

### 2.8 OtherDirectCostItem Object

```json
{
  "id": "6ba7b812-9dad-11d1-80b4-00c04fd430c8",
  "name": "MAXQDA Software License",
  "amount_eur": "9870.00",
  "is_cfs_item": false,
  "notes": "Annual renewal for qualitative analysis",
  "work_package_ids": [2]
}
```

| Field | Type | Notes |
|---|---|---|
| `is_cfs_item` | boolean | True = CFS auto-created item; only one allowed |
| `notes` | string\|null | |
| `work_package_ids` | array of integer | May be empty only for `is_cfs_item = true` |

### 2.9 Subcontracting Object

```json
{
  "amount_eur": "0",
  "work_package_id": 1
}
```

| Field | Type | Notes |
|---|---|---|
| `amount_eur` | string (Decimal) | Default `"0"` |
| `work_package_id` | integer | Single WP assignment |

---

## 3. Serialization Rules

### 3.1 Decimal Values

All monetary and percentage values are stored as **JSON strings** representing exact decimal numbers. This prevents floating-point precision loss during JSON serialization/deserialization.

- Correct: `"227900.00"`, `"0.70"`, `"25"`, `"0"`
- Incorrect: `227900.00` (JSON number — lossy for large values)

Implementation: `rust_decimal::serde::str` attribute on all `Decimal` fields.

### 3.2 UUID Values

All `id` fields are stored as **standard lowercase hyphenated UUID v4 strings**.

- Correct: `"f47ac10b-58cc-4372-a567-0e02b2c3d479"`
- Incorrect: `"F47AC10B58CC4372A5670E02B2C3D479"` (no hyphens)

### 3.3 Timestamps

All timestamp fields (`created_at`, `updated_at`) are stored as **RFC 3339 UTC strings** with `Z` suffix.

- Correct: `"2025-06-15T09:22:41Z"`

### 3.4 Tagged Enums

Rust enums with data (e.g., `TripType`) are serialized as **JSON objects with a single key** matching the variant name. This is serde's default adjacently tagged format.

```json
{ "Itemized": { ... } }
{ "FlatAmount": { ... } }
```

### 3.5 Optional Fields

Fields typed `Option<T>` serialize as `null` when absent or as the value when present. Fields with `#[serde(default)]` serialize as the type's default value when the field is absent from the input JSON (for backward compatibility).

### 3.6 Pretty Printing

Files are serialized with `serde_json::to_string_pretty()` — 2-space indented, human-readable JSON. This facilitates debugging, auditing, and version control diffing.

---

## 4. Versioning Strategy

### 4.1 Current State

`format_version: "1.0"` — the initial production version.

### 4.2 Versioning Scheme

The format version follows `MAJOR.MINOR`:
- **MINOR** increment: backward-compatible additions (new optional fields with `serde(default)`)
- **MAJOR** increment: breaking structural changes (field renames, type changes, required field additions)

### 4.3 Migration Protocol

The persistence layer's `load_project()` function is the migration boundary:

```rust
pub fn load_project(path: &Path) -> Result<Project, AppError> {
    let file: ProjectFile = serde_json::from_str(&json)?;

    match file.format_version.as_str() {
        "1.0" => Ok(file.project),
        "1.1" => migrate_1_0_to_1_1(file.project),
        "2.0" => migrate_1_x_to_2_0(file.project),
        v => Err(AppError::Persistence(format!(
            "Unsupported file format version: {v}. Please upgrade the application."
        )))
    }
}
```

Each migration function transforms the older domain representation to the current one. Migrations are cumulative (1.0 → 1.1 → 2.0, never 1.0 → 2.0 directly).

### 4.4 Backward Compatibility Guarantee

**The Budget Application must be able to open any `.ercbudget` file it previously produced.** Fields introduced in newer versions that are unknown to an older app version are silently ignored by serde. This is safe as long as unknown fields do not affect existing calculations.

The `serde(default)` attribute on `work_package_start_months` and `work_package_end_months` is the existing example of this pattern.

---

## 5. Format Extension for Execution Data — Version 1.1 Design

### 5.1 Extension Strategy

The execution data is stored as an **optional top-level block** in the file envelope alongside the existing `project` block. The Budget Application does not know about `execution_data` and ignores it via `serde`'s unknown field handling.

```json
{
  "format_version": "1.1",
  "created_at": "2025-06-15T09:22:41Z",
  "updated_at": "2026-03-01T11:45:00Z",
  "project": { ... },
  "execution_data": { ... }
}
```

The `execution_data` field is typed as `Option<ExecutionData>` in the Rust struct:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    pub format_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub project: Project,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_data: Option<ExecutionData>,
}
```

The `skip_serializing_if = "Option::is_none"` ensures the Budget Application never writes this field and existing files without it continue to load cleanly.

### 5.2 Format Version Increment

When the Execution Application first writes execution data to a file, `format_version` is incremented to `"1.1"`. The Budget Application (which only knows `"1.0"`) must be updated to tolerate `"1.1"` files — it should accept any `1.x` version and log a warning if it encounters an unknown MINOR version, rather than refusing to open the file.

### 5.3 ExecutionData Block Structure

```json
{
  "execution_data": {
    "schema_version": "1.0",
    "last_modified_by": "execution_app",
    "reporting_periods": [ ... ],
    "persons": [ ... ],
    "milestones": [ ... ],
    "deliverables": [ ... ],
    "tasks": [ ... ],
    "person_month_records": [ ... ],
    "actual_cost_entries": [ ... ],
    "travel_executions": [ ... ],
    "equipment_procurements": [ ... ],
    "risk_register": [ ... ],
    "issue_log": [ ... ],
    "meetings": [ ... ],
    "action_items": [ ... ],
    "documents": [ ... ],
    "subcontracting_lines": [ ... ],
    "project_notes": null
  }
}
```

All arrays default to `[]` and all optional strings default to `null`. The execution block is fully self-contained and cross-references Budget Application entities by their `id` (UUID).

### 5.4 Key ExecutionData Sub-Objects

**ReportingPeriod**
```json
{
  "id": "<uuid>",
  "period_number": 1,
  "start_month": 1,
  "end_month": 18,
  "submission_deadline": "2027-03-31",
  "status": "InProgress",
  "technical_report_submitted": false,
  "financial_report_submitted": false
}
```

**Person**
```json
{
  "id": "<uuid>",
  "full_name": "Dr. Mehmet Demir",
  "email": "m.demir@university.edu.tr",
  "institution": "İTÜ",
  "linked_role_id": "<PersonnelRole UUID>",
  "actual_start_date": "2026-01-15",
  "actual_end_date": null,
  "orcid": null
}
```

**Milestone**
```json
{
  "id": "<uuid>",
  "work_package_id": 1,
  "title": "Dataset Collection Complete",
  "planned_month": 18,
  "actual_completion_date": null,
  "status": "OnTrack",
  "description": null,
  "linked_deliverable_ids": []
}
```

**Deliverable**
```json
{
  "id": "<uuid>",
  "work_package_id": 2,
  "deliverable_number": "D2.1",
  "title": "Interim Analysis Report",
  "deliverable_type": "Report",
  "planned_month": 24,
  "actual_submission_date": null,
  "responsible_role_id": "<PersonnelRole UUID>",
  "status": "NotStarted",
  "dissemination_level": "Public",
  "document_ids": []
}
```

**PersonMonthRecord**
```json
{
  "id": "<uuid>",
  "person_id": "<Person UUID>",
  "role_id": "<PersonnelRole UUID>",
  "reporting_period_id": "<ReportingPeriod UUID>",
  "planned_months": "6.00",
  "reported_months": null,
  "approved_months": null,
  "status": "NotReported"
}
```

**ActualCostEntry**
```json
{
  "id": "<uuid>",
  "category": "Personnel",
  "description": "Dr. Demir — Period 1 salary",
  "planned_amount_eur": "47610.00",
  "actual_amount_eur": null,
  "currency": "EUR",
  "reporting_period_id": "<uuid>",
  "work_package_id": 1,
  "linked_entity_id": "<PersonnelRole UUID>",
  "invoice_reference": null,
  "status": "Draft",
  "auditor_notes": null
}
```

**RiskEntry**
```json
{
  "id": "<uuid>",
  "title": "Key researcher departure",
  "description": "PostDoc may leave before project completion",
  "work_package_id": 2,
  "probability": "Medium",
  "impact": "High",
  "risk_score": 6,
  "mitigation": "Cross-train second researcher on methodology",
  "contingency": "Recruit replacement within 3 months",
  "status": "Open",
  "owner_role_id": "<uuid>",
  "raised_date": "2026-02-01",
  "review_date": "2026-05-01"
}
```

**Meeting**
```json
{
  "id": "<uuid>",
  "title": "Monthly Project Meeting",
  "meeting_type": "Internal",
  "date": "2026-03-15",
  "attendee_role_ids": ["<uuid>", "<uuid>"],
  "agenda": "Budget review, deliverable status, risks",
  "minutes_document_id": null,
  "action_item_ids": []
}
```

### 5.5 Status Enumerations

```
MilestoneStatus:  NotStarted | OnTrack | Delayed | AtRisk | Completed | Cancelled
DeliverableStatus: NotStarted | InProgress | Submitted | Accepted | Rejected | Revised
DeliverableType:   Report | Dataset | Software | Prototype | Other
DisseminationLevel: Public | Restricted | Confidential
CostEntryStatus:   Draft | Submitted | Approved | Rejected
RiskProbability:   Low | Medium | High
RiskImpact:        Low | Medium | High
MeetingType:       Internal | ReviewMeeting | AuditMeeting | SteeringCommittee | Other
ActionItemStatus:  Open | InProgress | Closed
PersonMonthStatus: NotReported | Reported | Approved | Queried
ReportingPeriodStatus: NotStarted | InProgress | Submitted | Accepted
```

---

## 6. Autosave File

The `.ercbudget.autosave` sibling file uses the identical format. It is a safety copy written after every mutation. It is not the canonical file — users must explicitly save to the named `.ercbudget` file.

The autosave file should be deleted when the project is explicitly saved. If an autosave exists but no corresponding `.ercbudget` file is found, the application should offer to recover from the autosave.

---

## 7. File Validation Rules

When loading a `.ercbudget` file, the following must be checked:

| Check | Error Code | Recovery |
|---|---|---|
| File is valid UTF-8 JSON | `INVALID_JSON` | Show parse error to user |
| `format_version` field exists | `MISSING_FORMAT_VERSION` | Attempt to load as 1.0 |
| `format_version` is supported | `UNSUPPORTED_VERSION` | Ask user to upgrade app |
| `project.id` is a valid UUID | `INVALID_PROJECT_ID` | Generate new UUID and warn |
| `project.config.rate_version_id` exists in embedded data | `UNKNOWN_RATE_VERSION` | Prompt user to select rate version |
| Arrays lengths match `work_package_count` | `ARRAY_LENGTH_MISMATCH` | Attempt truncation/padding with warning |

Structural validation (field types, required fields) is handled by serde's deserialization — any type mismatch produces a `Persistence` error with the serde error message.

---

## 8. File Size and Performance Characteristics

Based on analysis of realistic projects:

| Scenario | Estimated file size |
|---|---|
| Empty project (config only) | ~1 KB |
| Typical project (10 roles, 20 trips, 10 equipment) | 15–30 KB |
| Large project (30 roles, 60 trips, 30 equipment) | 60–100 KB |
| With full execution data | Add 50–200 KB |

All files are small enough for instant serialization and deserialization. No streaming or chunked I/O is needed. The pretty-printed JSON format is appropriate for these sizes.

---

## 9. Future Format Versions (Planned)

| Version | Changes |
|---|---|
| 1.1 | Add optional `execution_data` block (Execution App release) |
| 1.2 | Add optional `partner_budgets` array (multi-partner support) |
| 2.0 | Reserved for major structural restructuring (if ever needed) |

---

## 10. Open Questions

1. Should the Budget Application be updated to explicitly tolerate `format_version: "1.1"` before the Execution App ships, or should they ship simultaneously?
2. Should execution data have its own `schema_version` independent of the file `format_version` to allow the execution schema to evolve independently?
3. Should the autosave file use a different extension (e.g., `.ercbudget~`) to distinguish it from canonical files more clearly?

---

## 11. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Budget App opens a 1.1 file and silently drops execution data | High | Test Budget App with 1.1 files; ensure unknown fields are not removed on save |
| ExecutionData block grows very large, slowing load times | Low | File sizes remain small (< 1 MB) for realistic projects |
| Concurrent edits by Budget App and Execution App corrupt the file | Medium | File locking at OS level; warn users; never allow both apps to have the same file open simultaneously |

---

## 12. Confidence Level

**88%** — The extension strategy is sound and follows established patterns. The specific sub-object schemas for execution data are preliminary and will be refined during Phase 04 (requirements). The backward-compatibility guarantee is high-confidence.

---

## 13. Recommended Next Step

**Proceed to Phase 04 — Project Execution Requirements.**

The file format is defined. The execution data schema will be refined in Phase 04 as the full requirements for each execution module are elaborated.
