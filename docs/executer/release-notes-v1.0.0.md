# ERC Execution v1.0.0 — Release Notes (Draft)

*Status: draft, pending internal beta (Milestone 3, Week 15) and final
sign-off before tagging. Version numbers, dates, and the changelog below
reflect the state of `main` as of this draft.*

## What's New

This is the first release of **ERC Execution**, a companion desktop app to
the **ERC Budget** application (M2-EU Budgeter). It reads a `.ercbudget`
file already created in the Budget App and adds full project-execution
tracking on top of it — nothing in the planned budget is ever modified
from this app.

Built across seven sprints, all business logic shared with erc-budget via
the new `erc-core` crate rather than duplicated:

- **Project Dashboard** — planned-vs-actual per ERC budget category, CFS
  status tracking against actuals.
- **Work Package Management** — automatically derived status (Not
  Started/On Track/At Risk/Completed), leader assignment, notes.
- **Deliverable Tracking** — full lifecycle from planning through
  acceptance, automatic overdue detection, CORDIS registration reminders.
- **Personnel & Person-Month Tracking** — roster management and monthly
  FTE declarations with automatic salary cost estimation.
- **Milestone Tracking** — automatic at-risk detection, completion gated
  on linked deliverables being accepted.
- **Amendment Management** — a log of formal grant amendments (not part of
  the original module catalogue; designed for this release since the
  planning docs referenced it without a spec).
- **Travel, Equipment, Other Costs, and Subcontracting Tracking** — actual
  expenditure recording against each planned budget line, each with its
  own overspend warning threshold.
- **Financial Reporting** — planned-vs-actual reconciliation across all
  five ERC cost categories, reusing the Budget App's own calculation
  engine against actual instead of planned figures.
- **Reporting Period Management** — periods pre-populate automatically on
  first open (following the standard ERC CoG P1/P2/P3 pattern, generalised
  to any project duration), with submission tracking.
- **Risk Register** — probability × impact scoring, a list and matrix
  view, mandatory review dates for high-priority risks.
- **Issue Log** — priority tracking with automatic staleness detection for
  unresolved high-priority issues.
- **Notifications & Warnings** — a persistent tray surfacing all 12
  warning types from the spec (overdue deliverables, approaching
  deadlines, budget overruns, CFS compliance, stale risks/issues,
  unstaffed roles, and more), each one click-through to the relevant
  screen.
- **Reports & Export** — Financial Report, Technical Report Annex, Risk
  Register, and Person-Month Declaration as Excel workbooks; a
  Project Status Report as a printable PDF.

## Compatibility

- Requires a `.ercbudget` file produced by **ERC Budget v1.7.0 or later**
  (file format version `1.0` or `1.1`; ERC Execution upgrades either to
  `1.1` automatically the first time it saves).
- Opening a file for the first time in ERC Execution never modifies your
  planned budget data — it only adds a new, separate execution-tracking
  section to the file. Older files without that section get sensible
  empty defaults.
- Files remain fully readable in the Budget App after being opened and
  edited in ERC Execution — the Budget App simply ignores the
  execution-tracking section it doesn't understand.

## Known Limitations / Out of Scope for v1.0

These were deliberate scope decisions made while building against a spec
that, in places, depended on modules not yet built when the dependent
feature was implemented — noted here rather than silently shipped:

- **Person-month tracking is per calendar month, not per reporting
  period.** The 10% tolerance check (approved person-months vs. planned)
  is applied per month even though Reporting Periods now exist;
  re-expressing it per-period is a candidate for a future release.
- **Work Package actual cost is personnel-only.** The Dashboard shows
  actual spend across all categories project-wide, but the per-WP actual
  figure on the Work Packages screen only attributes personnel costs —
  travel/equipment/other-costs/subcontracting aren't currently allocated
  down to individual work packages.
- **Budget transfer flagging (>10% moved between categories) isn't
  implemented.** There's no concrete "transfer" event to detect in the
  current data model; the >15% category overrun warning is the only
  budget-variance signal today.
- **Travel actual cost is user-entered, not rate-table-computed.** The
  itemized EU rate-table calculation is still used to derive the
  *planned* comparison figure for the overspend check, but the actual
  cost itself is a direct entry rather than a receipt-driven calculation.
- **Some date-range validations are skipped when a project has no call
  opening date set** in the Budget App (this field is optional there).
  Anything anchored to real calendar dates — a person's start date vs.
  their role's planned start, equipment purchase dates vs. project end,
  etc. — is simply not checked in that case, rather than guessed at.
- **Subcontracting's competitive-tender and host-institution checks are
  advisory only**, not enforced — the app has no external registry to
  verify vendor status against.
- **Reporting Period coverage (no gaps, full project duration) is
  advisory, not enforced.** You can save periods that don't fully tile the
  project; the app flags it rather than blocking the save, since
  enforcing a whole-list invariant on a one-record-at-a-time edit would
  make normal editing painful.
- **Single-user only.** No conflict resolution if the same file is opened
  in two places at once (see also `execution-requirements.md`'s M-23,
  which designs — but doesn't implement — a future multi-user role model).
- **V2 modules are not in this release**: Meeting Management, Action Item
  Tracker, Document Repository, Procurement Tracking, and Excel Import
  were all explicitly scoped as post-MVP in the original requirements
  catalogue.

## Security

An internal security review (`security-review-m3.md`) confirmed no
credentials are stored anywhere and the app makes no network calls in
normal operation. One latent gap — an unused, unconfigured auto-update
capability inherited from the Budget App's project template — is flagged
for a decision before release: either remove it (recommended, since it's
currently dead weight) or wire up a real update pipeline. A minor
cross-site-scripting gap in the PDF export (exploitable only via a
hand-edited project file, not remotely) was found and fixed during the
review.

## Upgrade Instructions

This is the first release — there is nothing to upgrade from. Install
alongside your existing ERC Budget installation; the two apps are
independent and don't need to be the same version, as long as ERC Budget
is v1.7.0 or later.

## Feedback

This is a beta release pending internal testing against real ERC/Horizon
Europe project budgets. Please report anything that looks wrong compared
to your actual project data, especially numbers that don't match what you
see in the Budget App.
