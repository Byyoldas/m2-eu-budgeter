# ERC Execution v1.1.1 — Release Notes

*This is the first published release of ERC Execution — tagged and
released as `erc-execution/v1.1.1`, distinct from ERC Budget's own release
line. `v1.1.0` was tagged and staged as a draft but never published; this
supersedes it with two more fixes that landed immediately after.*

## What's New

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

### EU Grants Time Declaration export

The Personnel screen has a **Time Declaration** button per person. It
fills in the person's name and calendar year on the real, official EU
Grants "Declaration of Days Worked on a Project" template — one `.docx`
per calendar year the person has records in (bundled as a `.zip` when they
span more than one year).

**Days Worked**, **Work Packages worked on**, and every signature field are
left blank for manual completion: the template's own footnote defines
"1 day" as *your institution's own standard working-day length*, which
this app has no way to know, so there's no reliable way to convert the
tracked FTE-fraction person-month data into an actual day count. The app
also doesn't track which Work Packages a person worked on in a given
month. Project acronym/number, Participant name, and Type of personnel
are blank too, since none of those are tracked anywhere in the app today.

### Save button and status

The sidebar has a **Save** button, greyed out when there's nothing to
save. The line underneath reads "All changes saved," "Unsaved changes,"
"Saving…," or — if a save actually fails — the real error, rather than
failing silently. The app already auto-saved 2 seconds after every
change; this adds an on-demand option and, more importantly, surfaces
failures instead of hiding them.

### Also new since the original build

- **Currency formatting.** Every displayed cost/amount value across the
  Dashboard, Work Packages, Personnel, Travel, Equipment, Other Costs, and
  Subcontracting screens now renders as `€ 12,345.67` instead of a raw
  multi-decimal string. Editable form fields are unaffected.
- **Content-Security-Policy.** The webview now runs under an explicit CSP
  (previously unset), enforcing "no network calls" as a platform-level
  guarantee.
- **Installer builds.** Windows (NSIS/MSI) and macOS (DMG) installers now
  build via GitHub Actions.
- **Hidden autosave file.** The per-mutation safety-copy sibling file is
  now dot-prefixed (`.yourfile.ercbudget.autosave`) instead of a plain
  visible file, so it doesn't clutter Finder/Explorer next to your actual
  project file.

### Fixed since the original 1.0.0 build

These were found while actually running the built app for the first time
(rather than only `cargo build`/`cargo test`), during the security review
and the first local test pass — the app had never previously been
launched outside of automated tests:

- The updater plugin was registered with no `plugins.updater` config
  block, which panicked the app on every startup, in both dev and release
  builds. Unregistered the plugin (dependency and capability permission
  kept in place for a future real update pipeline).
- The same missing config also broke the installer bundler outright
  (`createUpdaterArtifacts: true` requires that block to exist). Disabled
  updater-artifact generation until a real signing key/endpoint exists.
- A PDF export XSS gap: two fields in the Project Status Report's
  manually-built HTML weren't escaped (exploitable only via a hand-edited
  `.ercbudget` file, not remotely).

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

Deliberate scope decisions, noted here rather than silently shipped:

- **Person-month tracking is per calendar month, not per reporting
  period.** The 10% tolerance check is applied per month even though
  Reporting Periods exist; re-expressing it per-period is a candidate for
  a future release.
- **Work Package actual cost is personnel-only.** Travel/equipment/other
  costs/subcontracting aren't currently allocated down to individual work
  packages.
- **Budget transfer flagging (>10% moved between categories) isn't
  implemented** — no concrete "transfer" event exists in the data model.
- **Travel actual cost is user-entered, not rate-table-computed** — the
  itemized rate table is used only for the comparison figure in the
  overspend check.
- **Some date-range validations are skipped when a project has no call
  opening date set** in the Budget App, including the Time Declaration
  export's calendar-year mapping.
- **Subcontracting's competitive-tender and host-institution checks are
  advisory only** — no external registry to verify against.
- **Reporting Period coverage is advisory, not enforced.**
- **Single-user only.** No conflict resolution for concurrently-opened
  files.
- **V2 modules are not in this release**: Meeting Management, Action Item
  Tracker, Document Repository, Procurement Tracking, Excel Import.

## Security

An internal security review (`security-review-m3.md`) confirmed no
credentials are stored anywhere and the app makes no network calls in
normal operation, now enforced by an explicit CSP. The auto-update
capability (dependency + permission) is kept in place, unregistered, for
a future real update pipeline.

## Upgrade Instructions

Install alongside your existing ERC Budget installation; the two apps are
independent and don't need to be the same version, as long as ERC Budget
is v1.7.0 or later.

## Feedback

Please report anything that looks wrong compared to your actual project
data, especially numbers that don't match what you see in the Budget App.
