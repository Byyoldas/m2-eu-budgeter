# Regression + Performance Pass — erc-execution (Milestone 3, Week 15)

Performed 2026-08-04, covering the "Regression testing" and "Performance
profiling" rows of `development-roadmap.md`'s Week 15 table. This is a
static/manual-preview pass, not the "internal beta w/ 3–5 real ERC/HE
budgets" item on the same table — that one needs real project files and is
still open.

## Automated test suite

| Suite | Count | Result |
|---|---|---|
| erc-core | 171 | ✅ pass |
| erc-budget (Rust) | 26 | ✅ pass |
| erc-budget (frontend) | 116 | ✅ pass |
| erc-execution (Rust) | 178 | ✅ pass |
| erc-execution (frontend) | 18 | ✅ pass |
| **Total** | **509** | **✅ all pass** |

Also clean: `cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `tsc --noEmit` and `vite build` for both
frontend apps (`erc-budget`, `erc-execution`).

## Manual click-through

Loaded a synthetic project into the browser preview covering the edge
cases the automated tests already assert individually, but not previously
exercised *together on the same dataset*: a rejected deliverable with a
revision note and revised planned month, an overdue deliverable with a
CORDIS warning, a milestone showing the derived At-Risk overlay, an
At-Risk work package next to an On-Track and a Not-Started one, an
unlinked personnel role, a stale high-priority issue, a closed risk next
to an open high-priority one overdue for review, multiple category
overrun warnings, a reporting period marked Submitted next to two Open
ones (one with a deadline inside 14 days), and both delivered/pending
equipment procurements.

Clicked through all 14 module screens (Dashboard, Work Packages,
Deliverables, Personnel, Milestones, Amendments, Travel, Equipment, Other
Costs, Subcontracting, Reporting Periods, Risk Register, Issue Log,
Reports & Export) plus the notification tray (10 warnings, one of each
severity present) and fired all 5 exports against this dataset.

**Result: zero console errors across the entire pass.** Every derived
field (overdue flags, status overlays, overspend warnings, revised-month
display, stale-issue badge) rendered as expected. Screenshots taken at
each screen for the record; not attached here.

## Performance spot-check

Not a full profiling pass (out of scope for this session — that needs
real usage data and a profiler, not a synthetic timing test) — a sanity
check against the spec's stated target of **<100ms per IPC command**.

`build_summary` is the function every mutating IPC command runs before
returning (the "full recalculation on every mutation" pattern used
throughout the app) — it's the single most expensive operation in the
whole backend, so timing it stands in for the worst case across all ~40
commands.

**Method:** a temporary Rust test (written, run in `--release` mode, and
removed afterward — not committed) built a synthetic project far larger
than any real ERC grant would have — 20 work packages (vs. a typical 5–8),
30 personnel roles, 200 person-month records, and 60 each of deliverables,
milestones, risks, issues, equipment procurements, actual cost entries,
and subcontracting lines — then called `build_summary` 20 times after a
warm-up call and averaged.

**Result: ~600µs (0.6ms) per call — roughly 165× under the 100ms target**,
at a scale well beyond realistic project size. This isn't surprising given
the architecture: everything is in-memory (no database, no disk I/O in the
hot path — persistence happens separately, after the summary is built),
and the computation is a handful of linear passes over small collections.
Tauri's own IPC transport (JSON serialization + webview bridge round-trip)
adds some latency on top of this, but that's a fixed, well-understood cost
Tauri apps generally keep in the low single-digit milliseconds — not
something `build_summary`'s ~0.6ms budget leaves any real risk of
threatening the 100ms target.

**No performance concerns found.** No further profiling work is
recommended before release; if this needs revisiting, real project sizes
during the internal beta are the more useful signal than further synthetic
testing.

## Not covered by this pass

- **Internal beta with real ERC/HE budgets** — needs the user's actual
  project files; can't be simulated.
- **UI update latency (<16ms target)** — the spec's own frame-budget
  target for UI updates; not measured here (would need real browser
  profiling tools, not a synthetic Rust-side timing test).
- **Installer builds / code signing** — separate Week 15 item, not started.
