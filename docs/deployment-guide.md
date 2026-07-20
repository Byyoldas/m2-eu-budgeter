# M2-EU Budgeter — Deployment Guide

**Version:** 1.6.0
**Date:** 2026-07-17
**Audience:** Whoever is cutting a release of M2-EU Budgeter

> This describes the **actual release process currently in use** for this project — GitHub Releases, an unsigned macOS build produced locally, and a Windows build produced by GitHub Actions CI, both wired into the Tauri auto-updater. There is **no Apple code signing/notarization and no Windows EV certificate signing** in this project. If your institution later requires either (e.g. for wider public distribution beyond direct download links), treat that as new work, not something this guide already covers — Apple Developer Program enrollment, Windows code-signing certificates, and CI secrets for both would all need to be set up from scratch.

---

## Contents

1. Prerequisites
2. Version Bump
3. Building macOS (local, signed for the updater)
4. Building Windows (GitHub Actions CI)
5. The Auto-Updater Signing Key
6. Publishing the Release
7. Post-Release Verification
8. Troubleshooting
9. First-Time Setup (if the signing key is ever lost/rotated)

---

## 1. Prerequisites

| Tool | Purpose |
|---|---|
| Rust stable, Node 20 LTS, pnpm | Same as `developer-guide.md` §1 |
| `gh` (GitHub CLI), authenticated | Pushing, watching CI runs, creating releases, downloading artifacts |
| `node` (for `scripts/generate-latest-json.mjs`) | Building the updater manifest |
| The updater private key | See §5 — required for **every** `pnpm tauri build`, not just releases, because `createUpdaterArtifacts: true` is always on |

Releases are built on macOS for the macOS installer, and on GitHub Actions (`windows-latest` runner) for the Windows installers — there is no cross-compilation in either direction.

---

## 2. Version Bump

Three files must all be updated to the same version and kept in sync — there is no single source of truth / no script that does this automatically:

- `package.json` → `"version"`
- `src-tauri/tauri.conf.json` → `"version"`
- `src-tauri/Cargo.toml` → `[package] version`

After editing `Cargo.toml`, run `cargo check` inside `src-tauri/` once to refresh `Cargo.lock` (the workspace lockfile at the repo root) — otherwise the commit will be missing the corresponding `Cargo.lock` change.

This project has followed a simple pattern: every user-visible change (bug fix or feature) gets its own minor version bump and its own release — there hasn't been a patch (`x.y.Z`) release yet, and no particular SemVer discipline beyond "bump the minor version, write what changed."

---

## 3. Building macOS (local, signed for the updater)

There is no CI job for macOS — it's built locally on a Mac and installed/distributed from there.

```bash
cd erc-budget
pkill -f "M2-EU Budgeter.app/Contents/MacOS/erc-budget" 2>/dev/null   # quit a running dev copy first
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/m2eubudgeter-updater.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
pnpm tauri build
```

This produces, under `src-tauri/target/release/bundle/`:
- `macos/M2-EU Budgeter.app` — the app bundle itself (unsigned, no Apple notarization)
- `dmg/M2-EU Budgeter_<version>_aarch64.dmg` — the installer end users download
- `macos/M2-EU Budgeter.app.tar.gz` + `.app.tar.gz.sig` — the **updater artifact**, used only by the in-app auto-updater, not by a human downloading the app directly

This project only builds for `aarch64` (Apple Silicon) — there is no Intel/`x86_64-apple-darwin` or universal binary build.

**Install/replace the local copy for your own testing:**
```bash
rm -rf "/Applications/M2-EU Budgeter.app"
cp -R "target/release/bundle/macos/M2-EU Budgeter.app" "/Applications/M2-EU Budgeter.app"
open "/Applications/M2-EU Budgeter.app"
```

**If `bundle_dmg.sh` fails:** a prior interrupted build can leave a DMG disk image mounted, which blocks the next build. Check `hdiutil info` for a stray `/Volumes/dmg.*` mount pointing at this project's `rw.*.dmg`, then `hdiutil detach <disk-identifier> -force` and retry.

---

## 4. Building Windows (GitHub Actions CI)

Windows installers are built by `.github/workflows/windows-build.yml`, which triggers automatically on every push to `main` (and via manual `workflow_dispatch`). It signs its output using the same updater key, stored as the `TAURI_SIGNING_PRIVATE_KEY` repository secret.

**You don't need to do anything to trigger it** — pushing your version-bump commit to `main` starts it. To wait for it and pull down the results:

```bash
git push origin main
gh run list --workflow=windows-build.yml --limit 3     # find the run that matches your commit
gh run watch <run-id> --exit-status                     # blocks until it finishes (~13-14 minutes)
gh run download <run-id> -n windows-installers -D <some-local-dir>
```

This produces `msi/M2-EU Budgeter_<version>_x64_en-US.msi` and `nsis/M2-EU Budgeter_<version>_x64-setup.exe`, each with a matching `.sig` file (`.msi.sig` / `.exe.sig`) alongside it. The `.exe` (NSIS) is the one referenced in the updater manifest (§5) — that's the standard Tauri v2 Windows updater artifact; the `.msi` is offered as an alternative manual-install option but isn't itself used by the auto-updater.

Like macOS, there is only one Windows target — `x64`. No `arm64` Windows build exists.

---

## 5. The Auto-Updater Signing Key

**Where it lives:** `~/.tauri/m2eubudgeter-updater.key` (private, unencrypted, generated with `--ci`) and `~/.tauri/m2eubudgeter-updater.key.pub` (public) on the machine that first set this up. The private key is also stored as the `TAURI_SIGNING_PRIVATE_KEY` GitHub Actions secret on the repo, for the Windows CI build. **It is not committed anywhere in the repository.** The public key *is* committed, embedded directly in `src-tauri/tauri.conf.json`'s `plugins.updater.pubkey`.

**If you're building on a machine that doesn't have this key**, every `pnpm tauri build` will fail with `A public key has been found, but no private key.` — you need a copy of `~/.tauri/m2eubudgeter-updater.key` (get it from wherever it's backed up, or from someone who has it; it is not recoverable from the public key or from GitHub).

**⚠️ If this key is ever permanently lost:** every already-installed copy of the app becomes permanently unable to auto-update via the old key (a new keypair means a new pubkey, which old installs won't trust) — every existing user would need to manually download the next version once. Back this file up somewhere durable (a password manager, an encrypted note) in addition to the local filesystem and the GitHub secret.

**Non-obvious gotcha:** even though the key was generated unencrypted (`--ci` flag), `tauri build` still tries to interactively prompt for a decryption password unless `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` is explicitly set (an empty string is fine) — otherwise it fails non-interactively with `os error 6` (no tty). Both the CI workflow and any local build must set both env vars.

---

## 6. Publishing the Release

This is the step most prone to silent failure, because of one GitHub behaviour worth internalizing: **GitHub silently replaces spaces with dots in uploaded release asset filenames.** If you upload a file called `M2-EU Budgeter_1.6.0_aarch64.dmg`, the asset that actually ends up live is named `M2-EU.Budgeter_1.6.0_aarch64.dmg` — a URL built from the *intended* filename will 404. **Always stage release assets under space-free filenames before calling `gh release create`.**

**Step 1 — stage everything with hyphens instead of spaces:**
```bash
STAGE=/tmp/release-<version>
mkdir -p "$STAGE"
cp "target/release/bundle/dmg/M2-EU Budgeter_<version>_aarch64.dmg" "$STAGE/M2-EU-Budgeter_<version>_aarch64.dmg"
cp "target/release/bundle/macos/M2-EU Budgeter.app.tar.gz" "$STAGE/M2-EU-Budgeter_<version>_aarch64.app.tar.gz"
cp "target/release/bundle/macos/M2-EU Budgeter.app.tar.gz.sig" "$STAGE/M2-EU-Budgeter_<version>_aarch64.app.tar.gz.sig"
cp "<windows-download-dir>/msi/M2-EU Budgeter_<version>_x64_en-US.msi" "$STAGE/M2-EU-Budgeter_<version>_x64_en-US.msi"
cp "<windows-download-dir>/nsis/M2-EU Budgeter_<version>_x64-setup.exe" "$STAGE/M2-EU-Budgeter_<version>_x64-setup.exe"
cp "<windows-download-dir>/nsis/M2-EU Budgeter_<version>_x64-setup.exe.sig" "$STAGE/M2-EU-Budgeter_<version>_x64-setup.exe.sig"
```

**Step 2 — build `latest.json`** (the updater manifest — see `scripts/generate-latest-json.mjs`):
```bash
node scripts/generate-latest-json.mjs \
  --version <version> \
  --notes "What changed in this release" \
  --macos-sig "$STAGE/M2-EU-Budgeter_<version>_aarch64.app.tar.gz.sig" \
  --macos-asset "M2-EU-Budgeter_<version>_aarch64.app.tar.gz" \
  --windows-sig "$STAGE/M2-EU-Budgeter_<version>_x64-setup.exe.sig" \
  --windows-asset "M2-EU-Budgeter_<version>_x64-setup.exe" \
  --out "$STAGE/latest.json"
```
This reads the two `.sig` files and writes a manifest whose `platforms.darwin-aarch64.url` / `platforms.windows-x86_64.url` point at `https://github.com/Byyoldas/m2-eu-budgeter/releases/download/v<version>/<asset>` — matching the space-free filenames from Step 1 exactly.

**Step 3 — create the release**, uploading the `.dmg`, `.app.tar.gz`, `.msi`, `.exe`, and `latest.json` (the `.sig` files themselves don't need to be uploaded — their content is already embedded inside `latest.json`):
```bash
gh release create v<version> \
  "$STAGE/M2-EU-Budgeter_<version>_aarch64.dmg" \
  "$STAGE/M2-EU-Budgeter_<version>_aarch64.app.tar.gz" \
  "$STAGE/M2-EU-Budgeter_<version>_x64_en-US.msi" \
  "$STAGE/M2-EU-Budgeter_<version>_x64-setup.exe" \
  "$STAGE/latest.json" \
  --title "v<version>" \
  --notes "..."
```

---

## 7. Post-Release Verification

`gh release view`/`gh release download` succeed even if the *public, unauthenticated* download path is broken (they're authenticated via your own `gh` login) — this can mask a real problem. Verify the exact path the updater and a random end user will actually hit:

```bash
gh release view v<version> --json assets --jq '.assets[].name'   # confirm no space-mangled names slipped through

curl -sL -o /dev/null -w "latest.json: HTTP %{http_code}\n" \
  "https://github.com/Byyoldas/m2-eu-budgeter/releases/latest/download/latest.json"
curl -sIL -o /dev/null -w "macOS asset: HTTP %{http_code}\n" \
  "https://github.com/Byyoldas/m2-eu-budgeter/releases/download/v<version>/M2-EU-Budgeter_<version>_aarch64.app.tar.gz"
curl -sIL -o /dev/null -w "Windows asset: HTTP %{http_code}\n" \
  "https://github.com/Byyoldas/m2-eu-budgeter/releases/download/v<version>/M2-EU-Budgeter_<version>_x64-setup.exe"
```

All three must return `HTTP 200`. If they 404 and the repo is public, wait ~10-15 seconds (GitHub's asset CDN has a brief propagation delay right after upload) and retry before assuming something's actually wrong.

**If any of these 404 persistently, check whether the repository is private.** GitHub Releases on a private repo are only downloadable by authenticated requests — `gh` works fine because it carries your token, but the installed app has no credentials and hits exactly the same 404 a bare `curl` does. This repo was flipped from private to public specifically to fix this (in v1.5.0) — if it's ever made private again, the auto-updater silently stops working for everyone until it's public again or the release hosting moves elsewhere (e.g. a dedicated public releases-only repo).

---

## 8. Troubleshooting

**`pnpm tauri build` fails with "A public key has been found, but no private key":** you're missing `TAURI_SIGNING_PRIVATE_KEY` (and possibly `TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""`) in the environment. See §5.

**`bundle_dmg.sh` fails on macOS:** almost always a stuck `hdiutil` mount from an interrupted prior build. See §3.

**Windows CI run fails at the "Build Windows installers" step:** check that the `TAURI_SIGNING_PRIVATE_KEY` repo secret is still set (`gh secret list`) — if it was ever rotated or removed, the signed build will fail the same way a local build does without the key.

**Update check silently finds nothing, even though you just published a newer version:** most likely `latest.json` wasn't uploaded to the release, or one of its asset URLs doesn't match the real (space-free) uploaded filename — see §6 and §7. `UpdateChecker.tsx` deliberately swallows check failures rather than showing an error, by design (a broken update check must never interrupt normal use of the app), so a broken manifest looks identical to "you're already up to date" from inside the app. Always verify with `curl` per §7 rather than trusting the in-app behavior alone.

**`cargo build` fails with "linker not found" on macOS:** install Xcode Command Line Tools (`xcode-select --install`).

---

## 9. First-Time Setup (if the signing key is ever lost/rotated)

Only needed if starting over — e.g. the key was lost and every existing install needs to be told about a new one via a manual one-time download.

```bash
mkdir -p ~/.tauri
pnpm tauri signer generate --ci -w ~/.tauri/m2eubudgeter-updater.key
```

Then:
1. Copy the printed public key into `src-tauri/tauri.conf.json`'s `plugins.updater.pubkey`.
2. `gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/m2eubudgeter-updater.key` to update the CI secret.
3. Back up `~/.tauri/m2eubudgeter-updater.key` somewhere durable.
4. Cut a new release as normal (§2-§7) — this becomes the last version anyone can reach via the *old* key's auto-updater, so make sure to actually distribute it (email, direct link, etc.) rather than relying on the updater to spread it.
