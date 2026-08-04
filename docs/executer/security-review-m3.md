# Security Review — erc-execution (Milestone 3, Week 15)

Performed 2026-08-04 against the Non-Functional Requirements bar stated in
`execution-requirements.md` §6: **"No credentials stored; no network calls
from the app."** Scope: `erc-execution/` only (Rust backend + React
frontend). `erc-core` was included since erc-execution depends on it
directly.

## Method

Static review — full-text search across `erc-execution/src`,
`erc-execution/src-tauri/src`, and `erc-core/src` for credential handling,
network APIs, injection sinks, and file-path handling, plus a read-through
of `tauri.conf.json` and `capabilities/default.json`. No dynamic scanning
or dependency CVE audit was performed (out of scope for this pass).

## Findings

### 1. No credentials stored — confirmed, no findings

Searched for API keys, tokens, passwords, and credential-adjacent
identifiers across all Rust and TypeScript source, plus `tauri.conf.json`,
`Cargo.toml`, and `package.json`. Nothing found. The only persisted state
is the `.ercbudget` project file (plain JSON, no secrets) and its
`.autosave` sibling. No environment-variable-based secrets are read
anywhere (`std::env::var` isn't called at all; the only `std::env` use is
`temp_dir()` in test helpers).

### 2. No network calls — confirmed for actual behavior, one latent gap

No `fetch`, `XMLHttpRequest`, `axios`, `reqwest`, or raw `tokio::net`
usage exists anywhere in the app's own code. All IPC is local
(Tauri's own command bridge), and all persistence is local disk I/O via
`std::fs`.

**Gap:** `tauri-plugin-updater` is a declared dependency, registered as a
plugin in `lib.rs`, and granted `updater:default` in
`capabilities/default.json` — but `tauri.conf.json` has no
`plugins.updater` block (no `pubkey`/`endpoints`), and nothing in the
frontend ever calls `check()`. This was a deliberate deferral flagged
during Sprint E1 ("erc-budget's real signing key would be wrong here;
revisit in Milestone 3" — see project memory) and Milestone 3 is that
revisit.

Today this is inert: no endpoint is configured, so even if something
called `check()` it would fail rather than reach out. But the capability
is fully wired (permission + plugin + dependency), which is a weaker
guarantee than "no network calls" as a structural property of the app —
it currently holds only because nothing invokes it, not because the app
is incapable of it.

**Decision (2026-08-04): keep it**, for a future real update pipeline —
same tradeoff erc-budget already made and ships with. The app will make
network calls to check for updates once that pipeline exists (its own
signing key, its own endpoint); it doesn't yet, so this remains inert for
now. "No network calls" describes v1.0's actual behavior, not a permanent
architectural constraint — that's now explicit rather than assumed.

**Minor, same shape:** `shell:default` is also granted but never invoked
by the frontend (`tauri_plugin_shell::init()` is registered but nothing
calls `shell.open()` or similar). Lower priority than the updater finding
since the shell plugin's default permission set doesn't include arbitrary
command execution — but it's unused capability all the same, and could be
dropped for the same least-privilege reasoning if there's no near-term
plan to open external links from the app.

### 3. CSP is `null`

`tauri.conf.json`'s `app.security.csp` is `null`, meaning the webview
enforces no Content-Security-Policy. This is inherited from erc-budget's
own config (same value there), so it's not a regression introduced by
erc-execution — but for an app whose stated design goal is zero network
calls, an explicit CSP (e.g. `default-src 'self'; connect-src 'none'`)
would enforce that as a platform-level guarantee rather than relying on
code review to keep catching accidental additions. Recommend setting one
before v1.0, independent of the updater decision above.

### 4. Injection: one real gap, found and fixed

Every screen renders through React's own auto-escaping (no
`dangerouslySetInnerHTML`, no `eval`, no raw `innerHTML` anywhere in the
codebase — verified by search). The one exception is
`src/export/pdfExporter.ts`, which builds a full HTML document as a
template string for the Project Status Report PDF export (per
`execution-architecture.md` §9.4's "HTML + `window.print()`" design) and
writes it via `document.write()` in a new window — bypassing React
entirely.

Most interpolated fields there were already passed through an `escapeHtml`
helper, but two were not: a work package's derived `status` (a closed
enum, not exploitable in practice) and a reporting period's
`submission_deadline` (a free-text string field). The latter is
constrained to a valid ISO date by `validate_reporting_period` on every
write — but that validation runs at write time, not at load time, so a
hand-edited `.ercbudget` file (or one produced by some future tool that
skips validation) could smuggle markup into this field and have it execute
when the PDF export runs. **Fixed**: both fields are now escaped, with a
regression test (`pdfExporter.test.ts`) asserting an `<img
onerror=...>` payload in `submission_deadline` renders as inert text.

### 5. File I/O / path handling

`open_execution_project` takes a path from the frontend and reads it with
`std::fs::read_to_string`; `save_execution`/`auto_save` write with
`std::fs::write`. The path always originates from the OS-native file-open
dialog (`@tauri-apps/plugin-dialog`, invoked once in `Welcome.tsx`) — the
user explicitly picks the file via their own OS session, so this isn't a
path-traversal issue in the usual server-boundary sense; the user already
has whatever filesystem access their OS account grants. The autosave path
is derived from the already-opened file's own path
(`.with_extension("ercbudget.autosave")`), not from any additional input.
No findings here.

The frontend never imports `@tauri-apps/plugin-fs` directly — all file
I/O happens in Rust command handlers, not via IPC-exposed fs commands —
and indeed `capabilities/default.json` grants no `fs:*` permission at all,
so even if frontend code tried to call the fs plugin's JS API it would be
denied. This is the correct least-privilege state and needs no change.

## Summary

| # | Finding | Severity | Status |
|---|---|---|---|
| 1 | Credentials | — | No findings |
| 2 | Updater plugin fully wired but unconfigured/unused | Low (latent, not active) | **Decided: keep**, for a future update pipeline |
| 2b | Shell plugin permission granted but unused | Info | Optional cleanup, not actioned |
| 3 | CSP is `null` | Low | Still open — recommend fixing before v1.0 |
| 4 | PDF export missed escaping two fields | Medium (self-XSS via tampered file, no remote vector) | **Fixed** this session |
| 5 | File I/O / path handling | — | No findings |

Everything is fixable without touching business logic or the module
catalogue — this was a scoped audit, not a rewrite. Item 3 (CSP) is still
an open config-only decision; item 4 is already committed-ready.
