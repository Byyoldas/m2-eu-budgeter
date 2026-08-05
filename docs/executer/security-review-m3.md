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

**Gap:** `tauri-plugin-updater` is a declared dependency and granted
`updater:default` in `capabilities/default.json`, but `tauri.conf.json`
has no `plugins.updater` block (no `pubkey`/`endpoints`), and nothing in
the frontend ever calls `check()`. This was a deliberate deferral flagged
during Sprint E1 ("erc-budget's real signing key would be wrong here;
revisit in Milestone 3" — see project memory) and Milestone 3 is that
revisit.

**Decision (2026-08-04): keep it**, for a future real update pipeline —
same tradeoff erc-budget already made and ships with. The app will make
network calls to check for updates once that pipeline exists (its own
signing key, its own endpoint); it doesn't yet, so this remains inert for
now. "No network calls" describes v1.0's actual behavior, not a permanent
architectural constraint — that's now explicit rather than assumed.

**Correction, found the same day while verifying the CSP change below:**
the assumption that this gap was merely "inert" was wrong. Registering
`tauri_plugin_updater::Builder::new().build()` with no `plugins.updater`
config block doesn't degrade gracefully — it panics at startup, because
the plugin's `Config::pubkey` field is mandatory (not `Option`), and
Tauri passes the plugin config through as-is with no default fallback
when the block is absent. This was invisible to every prior sprint's
verification because `cargo build`/`cargo test` never exercise plugin
initialization — only actually launching the app via `tauri dev` does,
and that had never been done before this pass. In other words: erc-execution
could not launch, in dev or release builds, at any point before this fix.

**Fix:** the `.plugin(tauri_plugin_updater::Builder::new().build())` call
was removed from `lib.rs` (see comment there). The Cargo dependency and
the `updater:default` capability grant are left in place — the "keep it"
decision above still stands as a decision about the *dependency and
permission scaffolding*, since a real update pipeline can re-register the
plugin with a real `pubkey`/`endpoints` block when it's built. What
changed is that the plugin is no longer *registered* against an empty
config in the meantime, since that combination doesn't produce "inert," it
produces "the app doesn't start."

**Minor, same shape:** `shell:default` is also granted but never invoked
by the frontend (`tauri_plugin_shell::init()` is registered but nothing
calls `shell.open()` or similar). Lower priority than the updater finding
since the shell plugin's default permission set doesn't include arbitrary
command execution — but it's unused capability all the same, and could be
dropped for the same least-privilege reasoning if there's no near-term
plan to open external links from the app.

### 3. CSP is `null` — fixed

`tauri.conf.json`'s `app.security.csp` was `null`, meaning the webview
enforced no Content-Security-Policy. This was inherited from erc-budget's
own config (same value there), so it wasn't a regression introduced by
erc-execution — but for an app whose stated design goal is zero network
calls, an explicit CSP enforces that as a platform-level guarantee rather
than relying on code review to keep catching accidental additions.

**Fixed (2026-08-04):**

```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
img-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost;
object-src 'none'; base-uri 'self'; form-action 'none'
```

Notes on the non-default directives:
- `style-src 'unsafe-inline'` is required for the `<style>` block in
  `index.html` and for React's inline `style` props, both used throughout
  the app; there's no nonce/hash infrastructure to replace it with.
- `connect-src ... ipc: http://ipc.localhost` is Tauri v2's standard
  requirement for the webview-to-Rust IPC bridge to function at all.
- `object-src 'none'`, `base-uri 'self'`, and `form-action 'none'` are
  added hardening with no functional cost: nothing in the app uses
  plugins/embeds, injects a `<base>` tag, or needs a form to actually
  submit anywhere (every `<form onSubmit>` in the codebase calls
  `preventDefault()` and handles the submission in JS).
- No `img-src` beyond `'self' data:` was needed — the app loads no
  external or `asset://`-protocol images.

Verified by running the actual Tauri dev build (`pnpm tauri dev`) end to
end after the change: it compiles, launches, and the Rust process runs
without panicking or emitting Tauri-side CSP-configuration errors. This
also incidentally caught the updater startup crash in finding 2 above,
since that's the first time in the project's history this app was
actually launched rather than just compiled/unit-tested. Full in-webview
console verification (confirming zero CSP-violation warnings in the
DevTools console for style/IPC) still needs a human to check once, since
this environment has no way to attach to a native Tauri window's
DevTools console — flagged for the user to glance at during their first
local test pass.

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
is derived from the already-opened file's own path (dot-prefixed sibling,
`hidden_autosave_path()`, added 2026-08-05 so the shadow copy doesn't
clutter Finder/Explorer), not from any additional input. No findings here.

The frontend never imports `@tauri-apps/plugin-fs` directly — all file
I/O happens in Rust command handlers, not via IPC-exposed fs commands —
and indeed `capabilities/default.json` grants no `fs:*` permission at all,
so even if frontend code tried to call the fs plugin's JS API it would be
denied. This is the correct least-privilege state and needs no change.

## Summary

| # | Finding | Severity | Status |
|---|---|---|---|
| 1 | Credentials | — | No findings |
| 2 | Updater plugin registered with no config crashed the app at startup | High (app didn't launch) | **Fixed**: plugin unregistered; dependency/capability kept for a future pipeline |
| 2b | Shell plugin permission granted but unused | Info | Optional cleanup, not actioned |
| 3 | CSP is `null` | Low | **Fixed** — explicit policy set |
| 4 | PDF export missed escaping two fields | Medium (self-XSS via tampered file, no remote vector) | **Fixed** this session |
| 5 | File I/O / path handling | — | No findings |

Everything is fixable without touching business logic or the module
catalogue — this was a scoped audit, not a rewrite. All findings are now
either resolved or explicitly accepted (2b).
