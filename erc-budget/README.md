# M2-EU Budgeter

A cross-platform desktop application for preparing EU grant budgets (ERC Consolidator Grant and other Horizon Europe Actual Costs budgets) — personnel, equipment, travel, other direct costs, indirect costs, and the final submission table — without touching a spreadsheet.

It replaces a hand-built Excel workbook that was error-prone (hardcoded rates duplicated in seven places, string `-` placeholders silently ignored by `SUM()`, travel costs averaged equally across all years instead of the years they actually occur, an EU accommodation rate that quietly exceeded the official limit). Every one of those issues is fixed in the calculation engine.

---

## What it does

- Converts TRY-denominated salaries to EUR and projects them year-by-year with compounding inflation, per role.
- Calculates equipment depreciation with the EU eligibility cap (never claims more than the usage-weighted purchase cost).
- Looks up official EU Annex 2a/2b flight-distance-band, accommodation, and subsistence rates automatically — the user never has to know the rate tables.
- Computes Category E indirect costs (25% of eligible direct costs) and all project totals live, on every edit.
- Tracks the €430,000 Certificate on Financial Statements (CFS) threshold and prompts the user when it's crossed.
- Assigns each Work Package a start/end year and renders a Gantt-style timeline chart.
- Saves/loads projects as human-readable `.ercbudget` JSON files, with auto-save.
- Exports a formatted Excel workbook (Overview, Budget by Year, Detail, Other Direct Costs sheets), a one-page PDF summary, and a flat CSV.
- Runs fully offline — all EU rate tables are compiled into the binary.

See [`docs/business-rules.md`](docs/business-rules.md) for the 23 business rules and [`docs/calculation-engine.md`](docs/calculation-engine.md) for the exact formulas (CALC-01 through CALC-19).

---

## Tech stack

| Layer | Technology |
|---|---|
| Desktop framework | Tauri v2 |
| Backend | Rust — domain model, calculation engine, validation, persistence |
| Frontend | TypeScript 5, React 18, Zustand, React Hook Form + Zod, Recharts, Radix UI |
| Decimal arithmetic | `rust_decimal` — exact decimal math, no floating-point rounding on money |
| Export | ExcelJS, `@react-pdf/renderer` |
| Persistence | JSON (`.ercbudget` files), rate tables embedded via `include_str!` |

Full architecture write-up: [`docs/architecture-final.md`](docs/architecture-final.md).

---

## Installing (end users)

Download the installer for your platform from the project's GitHub Releases page:

- **macOS:** `.dmg` — drag the app into Applications, then right-click → Open on first launch (Gatekeeper requires this once for unsigned/unnotarized apps). If it doesn't work, try opening terminal and write the following code and execute it and then try to open the app. It will work. Code: `xattr -cr /Applications/M2-EU Budgeter.app`
- **Windows:** NSIS `.exe` or `.msi` — run the installer. If SmartScreen warns "unrecognized publisher," choose *More info → Run anyway* (the build isn't currently code-signed).

Full walkthrough of every screen: [`docs/user-manual.md`](docs/user-manual.md).

---

## Development

### Prerequisites

| Tool | Version |
|---|---|
| Rust (stable) | ≥ 1.78 |
| Node.js | ≥ 20 LTS (pnpm requires ≥ 22.13 — see note below) |
| pnpm | ≥ 10 |

macOS also needs Xcode Command Line Tools (`xcode-select --install`); Windows needs the Visual Studio Build Tools ("Desktop development with C++").

### Run in development

```bash
pnpm install
pnpm tauri dev
```

Opens a live-reloading Tauri window. Frontend changes hot-reload; Rust changes trigger a recompile.

### Run the tests

```bash
cd src-tauri && cargo test      # Rust unit + integration tests
cd .. && pnpm test               # TypeScript/Vitest tests
```

### Build an installer

```bash
pnpm tauri build
```

Produces platform-native installers under `target/release/bundle/` (note: at the workspace root, not inside `src-tauri/`, since this is a Cargo workspace). See [`docs/deployment-guide.md`](docs/deployment-guide.md) for code signing, notarization, and the CI pipeline.

---

## Project structure

```
erc-budget/
├── src/                  # TypeScript / React frontend
│   ├── screens/          # One file per wizard step
│   ├── components/       # Shared UI (cards, charts, dashboard panels)
│   ├── store/            # Zustand store (UI state only — backend owns domain data)
│   ├── validators/       # Zod schemas
│   ├── ipc/               # Typed wrappers around Tauri invoke()
│   └── export/            # Excel / PDF / CSV exporters
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri IPC command handlers
│   │   ├── domain/         # Entities + DTOs
│   │   ├── calculation/    # Pure calculation functions (CALC-01..19)
│   │   ├── validation/     # Business-rule validators
│   │   └── persistence/    # File I/O + bundled EU rate data
│   ├── resources/eu_travel_rates/   # Bundled Annex 2a/2b rate tables (JSON)
│   └── tests/integration_test.rs
├── .github/workflows/     # CI (Windows installer build)
└── docs/                  # Full spec set (see below)
```

Full module map and conventions: [`docs/developer-guide.md`](docs/developer-guide.md). How to extend the app (new cost category, new rate version, new export format, multi-partner support, i18n): [`docs/future-extensions.md`](docs/future-extensions.md).

---

## Documentation

| Document | Covers |
|---|---|
| [`docs/project-overview.md`](docs/project-overview.md) | Origin, scope, and the source Excel workbook this replaces |
| [`docs/excel-analysis.md`](docs/excel-analysis.md) | Line-by-line analysis of the original workbook, including every error it corrected |
| [`docs/business-rules.md`](docs/business-rules.md) | All 23 business rules (PS/PE/EQ/TR/OC/SC/IC/PT) |
| [`docs/domain-model.md`](docs/domain-model.md) | Every entity, attribute, and constraint |
| [`docs/input-catalog.md`](docs/input-catalog.md) | Every user-facing input field, with validation rules |
| [`docs/calculation-engine.md`](docs/calculation-engine.md) | Exact formulas for CALC-01 through CALC-19 |
| [`docs/ux-design.md`](docs/ux-design.md) | Screen-by-screen UX spec |
| [`docs/architecture-final.md`](docs/architecture-final.md) | As-built architecture, IPC contract, test architecture |
| [`docs/development-plan.md`](docs/development-plan.md) | Original sprint plan and risk register |
| [`docs/user-manual.md`](docs/user-manual.md) | End-user guide |
| [`docs/developer-guide.md`](docs/developer-guide.md) | Codebase map, how to add a feature |
| [`docs/deployment-guide.md`](docs/deployment-guide.md) | Build, sign, and release installers |
| [`docs/future-extensions.md`](docs/future-extensions.md) | Extension checklists (new cost category, i18n, multi-partner, etc.) |

---

## Status

v1.0 — Windows installer is built via GitHub Actions CI (`.github/workflows/windows-build.yml`); macOS is built locally.
