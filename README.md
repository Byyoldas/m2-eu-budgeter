# M2-EU Budgeter

A cross-platform desktop application for preparing EU grant budgets (ERC Consolidator Grant and other Horizon Europe Actual Costs budgets) — personnel, equipment, travel, other direct costs, indirect costs, and the final submission table — without touching a spreadsheet.

It replaces a hand-built Excel workbook that was error-prone (hardcoded rates duplicated in seven places, string `-` placeholders silently ignored by `SUM()`, travel costs averaged equally across all years instead of the years they actually occur, an EU accommodation rate that quietly exceeded the official limit). Every one of those issues is fixed in the calculation engine.

This repo is a small workspace: the app itself ([`erc-budget/`](erc-budget)) plus a shared domain/calculation/validation/persistence library ([`erc-core/`](erc-core)) that a sibling project, [ERC Execution](https://github.com/Byyoldas/erc-execution), also depends on from its own repo. See [Related projects](#related-projects) below.

---

## What it does

- Converts TRY-denominated salaries to EUR and projects them year-by-year with compounding inflation, per role.
- Organizes every cost category by **Work Package** rather than project year: Personnel cost is auto-allocated across Work Packages by month-overlap (a role's Start/End Month range is prorated against each WP's own Start/End Month range); Equipment is tagged to a single WP; Travel, Other Direct Costs, and Subcontracting can be tagged to one or more WPs, split evenly when more than one applies.
- Calculates equipment depreciation with the EU eligibility cap (never claims more than the usage-weighted purchase cost).
- Looks up official EU Annex 2a/2b flight-distance-band, accommodation, and subsistence rates automatically — the user never has to know the rate tables.
- Computes Category E indirect costs (25% of eligible direct costs) and all project totals live, on every edit.
- Tracks the €430,000 Certificate on Financial Statements (CFS) threshold and prompts the user when it's crossed.
- Renders a Gantt-style Work Package timeline chart, embedded in the exported Excel workbook.
- Saves/loads projects as human-readable `.ercbudget` JSON files, with auto-save.
- Exports a formatted Excel workbook (formula-linked, not just static values), a print-ready PDF summary, and a flat CSV.
- Checks for updates automatically on launch (and on demand from the Welcome screen), installing signed releases in place via Tauri's updater.
- Runs fully offline — all EU rate tables are compiled into the binary.

See [`docs/business-rules.md`](docs/business-rules.md) for the business rules and [`docs/calculation-engine.md`](docs/calculation-engine.md) for the exact formulas. Both carry "Current Implementation Notes" callouts where the as-built app has moved past the original spec (WP-based budgeting being the main one).

---

## Tech stack

| Layer | Technology |
|---|---|
| Desktop framework | Tauri v2 |
| Backend | Rust — domain model, calculation engine, validation, persistence (shared with ERC Execution via `erc-core`) |
| Frontend | TypeScript 5, React 18, Zustand, React Hook Form + Zod, Recharts, Radix UI |
| Decimal arithmetic | `rust_decimal` — exact decimal math, no floating-point rounding on money |
| Export | ExcelJS (formula-linked `.xlsx`), `window.print()` with a print stylesheet (PDF), flat CSV |
| Persistence | JSON (`.ercbudget` files), rate tables embedded via `include_str!` |
| Auto-update | `tauri-plugin-updater`, minisign-signed releases published to GitHub Releases |

Full architecture write-up: [`docs/architecture-final.md`](docs/architecture-final.md).

---

## Installing (end users)

Download the installer for your platform from the [GitHub Releases page](https://github.com/Byyoldas/m2-eu-budgeter/releases/latest):

- **macOS:** `.dmg` — drag the app into Applications, then right-click → Open on first launch (Gatekeeper requires this once for unsigned/unnotarized apps). If it doesn't work, open Terminal and run: `xattr -cr "/Applications/M2-EU Budgeter.app"`
- **Windows:** NSIS `.exe` or `.msi` — run the installer. If SmartScreen warns "unrecognized publisher," choose *More info → Run anyway* (the build isn't currently code-signed).

Once installed, the app checks for updates automatically and offers to install them in place. Full walkthrough of every screen: [`docs/user-manual.md`](docs/user-manual.md).

---

## Development

### Prerequisites

| Tool | Version |
|---|---|
| Rust (stable) | ≥ 1.78 |
| Node.js | ≥ 20 LTS |
| pnpm | ≥ 10 |

macOS also needs Xcode Command Line Tools (`xcode-select --install`); Windows needs the Visual Studio Build Tools ("Desktop development with C++").

### Run in development

```bash
pnpm install
cd erc-budget && pnpm tauri dev
```

Opens a live-reloading Tauri window. Frontend changes hot-reload; Rust changes (in either `erc-budget/src-tauri` or `erc-core`) trigger a recompile.

### Run the tests

```bash
# from the repo root — covers both erc-core and erc-budget/src-tauri
cargo test --workspace

# TypeScript/Vitest tests
pnpm --filter m2-eu-budgeter test
```

201 Rust tests, 116 TypeScript tests, as of this writing.

`erc-core`'s Rust structs also generate TypeScript bindings via `ts-rs` (checked in CI so they can't drift):

```bash
cargo test -p erc-core --features ts-rs
```

### Build an installer

```bash
cd erc-budget && pnpm tauri build
```

Produces platform-native installers under `target/release/bundle/` at the **repo root** (not inside `erc-budget/`), since this is a Cargo workspace. See [`docs/deployment-guide.md`](docs/deployment-guide.md) for signing and the release process.

---

## Project structure

```
.
├── erc-budget/             # The Tauri app itself
│   ├── src/                # TypeScript / React frontend
│   │   ├── screens/        # One file per wizard step
│   │   ├── components/     # Shared UI (cards, charts, dashboard panels)
│   │   ├── store/          # Zustand store (UI state only — backend owns domain data)
│   │   ├── validators/     # Zod schemas
│   │   ├── ipc/            # Typed wrappers around Tauri invoke()
│   │   └── export/         # Excel / PDF / CSV exporters
│   └── src-tauri/          # Rust backend — Tauri IPC commands + thin
│       └── src/            # re-export shims over erc-core's logic
│           ├── commands/   # Tauri IPC command handlers (erc-budget-specific)
│           └── ...         # domain/calculation/validation/persistence shims
├── erc-core/                # Shared domain, calculation, validation, and
│   ├── src/                 # persistence logic — also consumed by the
│   ├── bindings/             # separate erc-execution repo. Never depends
│   └── ts/                   # on erc-budget or erc-execution.
├── docs/                    # Full spec set (see below)
├── test-fixtures/           # Sample .ercbudget files used by integration tests
└── .github/workflows/       # CI (test/lint gate + Windows installer build)
```

Full module map and conventions: [`docs/developer-guide.md`](docs/developer-guide.md). How to extend the app (new cost category, new rate version, new export format, multi-partner support, i18n): [`docs/future-extensions.md`](docs/future-extensions.md).

---

## Documentation

| Document | Covers |
|---|---|
| [`docs/project-overview.md`](docs/project-overview.md) | Origin, scope, and the source Excel workbook this replaces |
| [`docs/excel-analysis.md`](docs/excel-analysis.md) | Line-by-line analysis of the original workbook, including every error it corrected |
| [`docs/business-rules.md`](docs/business-rules.md) | Business rules (PS/PE/EQ/TR/OC/SC/IC/PT), with current-implementation notes |
| [`docs/domain-model.md`](docs/domain-model.md) | Every entity, attribute, and constraint |
| [`docs/input-catalog.md`](docs/input-catalog.md) | Every user-facing input field, with validation rules |
| [`docs/calculation-engine.md`](docs/calculation-engine.md) | Exact calculation formulas, with current-implementation notes |
| [`docs/ux-design.md`](docs/ux-design.md) | Screen-by-screen UX spec |
| [`docs/architecture-final.md`](docs/architecture-final.md) | As-built architecture, IPC contract, test architecture |
| [`docs/development-plan.md`](docs/development-plan.md) | Original sprint plan and risk register |
| [`docs/user-manual.md`](docs/user-manual.md) | End-user guide |
| [`docs/developer-guide.md`](docs/developer-guide.md) | Codebase map, how to add a feature |
| [`docs/deployment-guide.md`](docs/deployment-guide.md) | Build, sign, and release installers |
| [`docs/future-extensions.md`](docs/future-extensions.md) | Extension checklists (new cost category, i18n, multi-partner, etc.) |

---

## Related projects

- **[ERC Execution](https://github.com/Byyoldas/erc-execution)** — a companion desktop app that reads a `.ercbudget` file produced here and adds day-to-day project-execution tracking on top of it (deliverables, milestones, risk register, actuals vs. planned). Shares its domain model and calculation logic with this app via `erc-core`, but never modifies a planned budget.
- **[erc-core](https://github.com/Byyoldas/erc-core)** — the standalone mirror of this repo's `erc-core/`, published so `erc-execution` can depend on it without needing this monorepo to exist. This repo keeps its own copy of `erc-core/` and doesn't depend on the standalone one.

---

## Status

v1.7.0 — auto-updating, signed macOS builds (local) and Windows builds (GitHub Actions CI, [`.github/workflows/windows-build.yml`](.github/workflows/windows-build.yml)); a separate [`ci.yml`](.github/workflows/ci.yml) gates every push/PR on `cargo fmt`/`clippy`/`test --workspace` plus the frontend test suite.
