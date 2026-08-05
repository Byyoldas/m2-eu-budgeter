# ERC Execution v1.1.2 — Release Notes

*Tagged and released as `erc-execution/v1.1.2`. This is a small follow-up to
v1.1.1 — one visual fix, no functional changes.*

## What's New

### App icon

ERC Execution had been shipping with the exact same icon file as ERC
Budget, making the two apps hard to tell apart in the Dock/Finder. It now
has its own icon: same EU-star-ring family look and rounded-square shape,
but on a dark green field with a checkmark in place of ERC Budget's euro
sign — green + checkmark reads as "execution/tracking" versus navy +
euro for "budget/planning."

No other changes since v1.1.1.

## What's Included (unchanged from v1.1.1)

**ERC Execution** is a companion desktop app to the **ERC Budget**
application (M2-EU Budgeter). It reads a `.ercbudget` file already created
in the Budget App and adds full project-execution tracking on top of it —
nothing in the planned budget is ever modified from this app.

All business logic is shared with erc-budget via the `erc-core` crate
rather than duplicated:

- **Project Dashboard** — planned-vs-actual per ERC budget category, CFS
  status tracking against actuals.
- **Work Package Management** — automatically derived status (Not
  Started/On Track/At Risk/Completed), leader assignment, notes.
- **Deliverable Tracking** — full lifecycle from planning through
  acceptance, automatic overdue detection, CORDIS registration reminders.
- **Personnel & Person-Month Tracking** — roster management and monthly
  FTE declarations with automatic salary cost estimation, plus a
  **Time Declaration export**.
- **Milestone Tracking** — automatic at-risk detection, completion gated
  on linked deliverables being accepted.
- **Amendment Management** — a log of formal grant amendments.
- **Travel, Equipment, Other Costs, and Subcontracting Tracking** — actual
  expenditure recording against each planned budget line, each with its
  own overspend warning threshold.
- **Financial Reporting** — planned-vs-actual reconciliation across all
  five ERC cost categories, reusing the Budget App's own calculation
  engine against actual instead of planned figures.
- **Reporting Period Management** — periods pre-populate automatically on
  first open (following the standard ERC CoG P1/P2/P3 pattern), with
  submission tracking.
- **Risk Register** — probability × impact scoring, a list and matrix
  view, mandatory review dates for high-priority risks.
- **Issue Log** — priority tracking with automatic staleness detection for
  unresolved high-priority issues.
- **Notifications & Warnings** — a persistent tray surfacing all 12
  warning types from the spec, each one click-through to the relevant
  screen.
- **Reports & Export** — Financial Report, Technical Report Annex, Risk
  Register, and Person-Month Declaration as Excel workbooks; a
  Project Status Report as a printable PDF; and an **EU Grants Time
  Declaration export**.
- **Save button and status** in the sidebar — greyed out when there's
  nothing to save, with a status line reading "All changes saved,"
  "Unsaved changes," "Saving…," or the real error if a save fails.
- **Hidden autosave file** — the per-mutation safety-copy sibling file is
  dot-prefixed (`.yourfile.ercbudget.autosave`) so it doesn't clutter
  Finder/Explorer.
- **Currency formatting** — `€ 12,345.67` throughout the Dashboard, Work
  Packages, Personnel, Travel, Equipment, Other Costs, and Subcontracting
  screens.
- **Content-Security-Policy** enforced at the webview level.

## Compatibility

- Requires a `.ercbudget` file produced by **ERC Budget v1.7.0 or later**
  (file format version `1.0` or `1.1`; ERC Execution upgrades either to
  `1.1` automatically the first time it saves).
- Opening a file for the first time in ERC Execution never modifies your
  planned budget data — it only adds a new, separate execution-tracking
  section. Older files without that section get sensible empty defaults.
- Files remain fully readable in the Budget App after being opened and
  edited in ERC Execution — the Budget App simply ignores the
  execution-tracking section it doesn't understand.

## Known Limitations / Out of Scope

Unchanged from v1.1.1 — see that release's notes or the User Manual for
the full list (person-month tracking is per calendar month rather than
per reporting period, Work Package actual cost is personnel-only, budget
transfer flagging isn't implemented, travel actual cost is user-entered,
subcontracting checks are advisory only, single-user only, V2 modules are
out of scope for this release).

## Security

No change since v1.1.1 — see `security-review-m3.md`. No credentials are
stored anywhere and the app makes no network calls in normal operation.

## Upgrade Instructions

Install alongside your existing ERC Budget installation; the two apps are
independent and don't need to be the same version, as long as ERC Budget
is v1.7.0 or later. If you have v1.1.1 installed, this is a drop-in
replacement — no data migration involved.

## Feedback

Please report anything that looks wrong compared to your actual project
data, especially numbers that don't match what you see in the Budget App.
