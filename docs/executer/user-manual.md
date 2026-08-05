# ERC Execution — User Manual (v1.1.2)

ERC Execution tracks the day-to-day running of a Horizon Europe project
against the budget you planned in the **Budget Application** (erc-budget).
It never creates a budget itself — it opens a `.ercbudget` file the Budget
App already produced, and adds everything that happens *after* the budget
is approved: who's actually working on the project, what's been delivered,
what's actually been spent, and what needs attention.

## 1. Getting Started

### Requirements

- A `.ercbudget` file already created and saved by the Budget Application.
  ERC Execution never originates a project — there is no "New Project"
  option.

### Opening a project

1. Launch ERC Execution.
2. Click **Open .ercbudget File…** on the welcome screen.
3. Pick the `.ercbudget` file in the native file dialog.

The first time a given file is opened in ERC Execution, it silently adds
an execution-tracking section to the file (your planned budget data is
never touched) and pre-populates a default set of **Reporting Periods**
(18-month interim periods, with the final period covering whatever remains
of the project — see §12). Opening the same file again later picks up
right where you left off.

### Saving

Everything you do saves automatically — there's nothing to lose by
closing the window. Three mechanisms cover you:

- Every single change (adding a person, editing a milestone, deleting a
  risk, ...) is written immediately to a hidden `.autosave` sibling file
  next to your project (see below) — a safety copy, not the file you
  opened.
- A secondary 2-second-debounced save also runs from the UI side, writing
  back to your actual named `.ercbudget` file.
- A **Save** button in the sidebar lets you save on demand instead of
  waiting for the debounce. It's greyed out when there's nothing to save;
  the line underneath it reads "All changes saved," "Unsaved changes," or
  (if a save fails) the actual error, rather than failing silently.

## 2. Navigation

The left sidebar lists every module. Modules that aren't relevant yet
(reserved for a future release) appear greyed out and aren't clickable.

In the top-right corner, a **▲ Warnings: N** button is always visible once
a project is open — this is the notification tray (§15). Click it to see
every active warning across the whole project; clicking a warning jumps
you straight to the screen it's about.

## 3. Dashboard

The landing screen after opening a project. Shows:

- **Project header** — title, PI, call reference, duration, and how far
  into the project you currently are (current month, computed from the
  budget's call opening date; defaults to month 1 if that date was never
  set in the Budget App).
- **Planned vs. Actual** — every ERC budget category (A Personnel, B
  Subcontracting, C1 Travel, C2 Equipment, C3 Other Direct Costs, E
  Indirect) side by side, planned figure next to the actual figure derived
  from everything you've recorded elsewhere in the app.
- **CFS status (actual)** — whether the Certificate on Financial Statements
  threshold (€430,000) is being approached or exceeded, based on actual
  spend rather than the original plan.

## 4. Work Packages

Work Packages themselves come from the Budget App and can't be edited
here — this screen adds a live status overlay on top of them:

- **Status** — automatically derived: *Not Started* before the WP's start
  month, *Completed* once its end month has passed **and** every
  deliverable belonging to it has been Accepted, *At Risk* if any of its
  milestones or deliverables are overdue or rejected, otherwise *On
  Track*.
- **Planned vs. actual spend** — actual figures here currently reflect
  personnel costs only (the richest, most timing-sensitive category);
  other categories are covered project-wide on the Dashboard.
- **Leader / Notes** — click **Edit** on a WP card to assign a leader
  (from the project's planned personnel roles) and free-text notes.

A warning banner appears on a card if actual spend exceeds planned by more
than 5%.

## 5. Deliverables

Tracks each deliverable from planning through acceptance.

- **Adding one**: title, type (Report/Dataset/Software/Prototype/Dem/
  Ethics/Other), work package, planned month, responsible role, and
  dissemination level (Public/Restricted to Programme/Confidential). The
  deliverable number (e.g. `D2.1`) is assigned automatically — first
  deliverable under WP2 is `D2.1`, the next `D2.2`, and so on.
- **Overdue flag**: a deliverable is flagged overdue once its planned
  month has passed without a submission date recorded.
- **Rejecting one**: switching status to *Rejected* requires a revision
  note and a revised planned month — the app won't let you save a
  rejection without them.
- **CORDIS warning**: Public deliverables that haven't been marked as
  registered in CORDIS show an advisory warning (the app doesn't integrate
  with CORDIS directly — this is a reminder, not an automatic check).
- **Reporting period**: once Reporting Periods exist (they do by default,
  see §12), each deliverable shows which period its planned month falls
  into.

## 6. Personnel & Person-Month Tracking

Two linked sections:

- **Roster** — link a named person to one of the project's planned
  personnel roles (one person per role at a time), with their actual
  start/end dates.
- **Person-Month Records** — for each person, log reported FTE-months for
  a given project month (capped at 1.0, i.e. full-time) and, once
  reviewed, an approved figure. Approved records get an automatic
  estimated cost in EUR, computed the same way the Budget App projects
  salary costs (inflation-adjusted, currency-converted). The app also
  checks that the total approved across everyone doesn't exceed the
  month's planned total by more than 10%.

**Time Declaration export.** Each person in the Roster has a **Time
Declaration** button, which downloads the official EU Grants "Declaration
of Days Worked on a Project" form — one filled `.docx` per calendar year
they have person-month records in (bundled as a `.zip` if more than one
year). Only the person's name and the year are filled in automatically.
**Days Worked, Work Packages worked on, and both signature blocks are
left blank for you to complete by hand** — the app has no reliable way to
convert its tracked FTE fractions into an actual day count (that depends
on your institution's own working-day length, which isn't recorded here),
and it doesn't track which Work Packages a person worked on in a given
month. Project acronym, project number, participant name, and type of
personnel are also blank, since none of those are tracked in the app.
This export needs the project's call opening date to be set in the Budget
App — without it, project months can't be mapped to real calendar years.

## 7. Milestones

- **Adding one**: title, work package, planned month, and optionally links
  to one or more Deliverables.
- **Status**: you set it directly (*Not Started, On Track, Delayed,
  Completed, Cancelled*) — except *At Risk*, which the app applies
  automatically to any *Not Started* milestone whose planned month has
  passed, and which you can't set by hand.
- **Completing one**: use the **Complete** button, or set status to
  *Completed* directly. Either way, if the milestone has any linked
  deliverables, every one of them must already be *Accepted* — the app
  blocks completion otherwise.

## 8. Amendments

A log of formal grant amendments (budget reallocations, duration
extensions, work package scope changes, personnel changes, or other
changes) — request date, decision date, status, financial impact, and
which work packages are affected. This module is purely a record-keeping
log; it's informational only and never changes the numbers shown elsewhere
in the app (the Budget App's figures remain the single source of truth for
what's planned).

## 9. Travel

Log actual travel against the trips you planned in the Budget App. Pick
the planned trip and instance number (e.g. the 2nd of 3 planned
instances), the traveller (from your Personnel roster), the travel date,
and the actual cost. A warning appears if the actual cost for that
instance exceeds 120% of what was planned per instance.

## 10. Equipment

Log actual equipment purchases against planned equipment items. Until you
tick **Delivery confirmed**, a purchase doesn't count toward actual costs
(its eligible depreciation is shown as "—"). A warning appears if the
actual purchase cost exceeds 110% of the planned cost.

## 11. Other Costs

Log actual Category C3 (Other Direct Costs) expenditure. You can link an
entry to a planned other-cost item, or leave it unlinked as "unbudgeted" —
unbudgeted entries require a justification note before they can be saved.
A warning appears if an item's actual total exceeds 110% of its planned
amount.

## 12. Subcontracting

Log actual subcontracting lines (vendor, contract reference, amount, work
package) against the project's single planned subcontracting lump sum. The
total of all lines is hard-capped at the planned amount — the app won't
let you exceed it. Two advisory (non-blocking) warnings appear when
relevant: amounts over €200,000 (a reminder that competitive tendering
rules may apply) and when you mark the vendor as the host institution.

## 13. Reporting Periods

Periods are pre-populated automatically the first time you open a project
(18-month interim periods; the final period absorbs whatever remains of
the project, matching the standard ERC CoG pattern of P1 M1–18 / P2 M19–36
/ P3 M37–60 for a 5-year project). For each period you can:

- Set a submission deadline.
- Tick off the technical and financial report as submitted.
- Mark the period **Submitted** — only allowed once both reports are
  ticked.
- Add, edit, or delete periods yourself if the defaults don't match your
  project's actual reporting schedule.

If your periods don't fully cover the project with no gaps, or the last
period doesn't reach the project's final month, an advisory banner says
so — it won't stop you from working, just flags it.

## 14. Risk Register

- **Adding a risk**: title, description, work package, probability
  (Low/Medium/High), impact (Low/Medium/High), and identified date. The
  risk's **score** (1–9) and **priority** are calculated automatically —
  probability × impact, with a score of 6+ counted High priority, 3–5
  Medium, 1–2 Low.
- **High-priority risks** require a review date within 30 days of today —
  the app won't save one without it.
- **List / Matrix views**: toggle between a sortable list and a 3×3
  probability-by-impact grid showing how many open/mitigated risks fall in
  each cell.
- Once a risk is marked **Closed**, it's terminal — you can't reopen it.
  If the situation recurs, log a new risk instead.

## 15. Issue Log

Track problems as they come up during execution — description, work
package, priority, and optionally a link back to a Risk Register entry it
came from. An issue can't be marked **Closed** without a resolution note.
A High-priority issue still open more than 14 days after it was raised is
flagged as stale, both here and in the notification tray.

## 16. Reports & Export

Five export types, each a snapshot of exactly what's on screen at the
moment you export (nothing is recalculated separately at export time):

| Export | Format | Contents |
|---|---|---|
| Financial Report | Excel | Planned vs. actual per category, plus per-Work-Package breakdown |
| Technical Report Annex | Excel | Deliverable and milestone status tables |
| Project Status Report | PDF | One-page dashboard summary for the PI/coordinator |
| Risk Register | Excel | Full risk list, sorted by score |
| Person-Month Declaration | Excel | One sheet per reporting period, pre-filled with each person's reported/approved months and cost estimate |

Excel exports download directly. The PDF export opens a print-preview
window and triggers your browser/OS print dialog — choose "Save as PDF"
there to get a file (if the window doesn't appear, check whether your
browser blocked the popup).

## 17. The Notification Tray

The **▲ Warnings: N** button (top-right, on every screen) lists every
active warning across the whole project, most severe first:

| Icon | Meaning |
|---|---|
| 🔴 | Error — needs attention (e.g. an overdue deliverable, a reporting deadline inside 14 days, CFS threshold exceeded and unaddressed) |
| 🟡 | Warning — worth a look (e.g. a budget category over by 15%+, a stale high-priority issue) |
| ⚪ | Info (e.g. a planned role with nobody linked to it yet) |

Click any entry to jump straight to the relevant screen.

## 18. Frequently Asked Questions

**Can I create a new project in ERC Execution?**
No. Every project starts in the Budget Application; ERC Execution only
opens files the Budget App already saved.

**Does editing something here change my planned budget?**
No. The Budget App's figures are read-only from ERC Execution's side —
this app only adds execution-tracking data alongside them in the same
file.

**I lost internet access / am working offline — does anything break?**
No. ERC Execution makes no network calls at all; everything is local to
your machine.

**Can two people edit the same file at once?**
The app is single-user by design in this release — there's no
conflict-resolution mechanism if two people open the same file
simultaneously on different machines.

**What if the app crashes mid-edit?**
Check for a hidden `.<yourfile>.ercbudget.autosave` file next to your
original (dot-prefixed, so it won't show in Finder/Explorer unless you
enable hidden files) — every mutation is written there immediately, so
you should lose at most the change that was in flight.
