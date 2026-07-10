# ERC Budget Tool — Deployment Guide

**Version:** 1.0  
**Date:** 2026-07-10  
**Audience:** Engineers responsible for building, signing, and distributing releases

---

## Contents

1. Prerequisites
2. Development Build
3. Production Build
4. macOS Build and Code Signing
5. Windows Build and Code Signing
6. Release Checklist
7. CI/CD Pipeline Skeleton
8. Auto-Update
9. Distribution
10. Troubleshooting

---

## 1. Prerequisites

Install the following on the build machine before proceeding.

### All platforms

| Tool | Version | Install |
|---|---|---|
| Rust stable | ≥ 1.78 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | ≥ 20 LTS | https://nodejs.org or via `nvm` |
| npm | ≥ 10 | Ships with Node.js 20 |
| Tauri CLI v2 | ≥ 2.0 | `npm install` (in `erc-budget/`) includes it as a dev dependency |

### macOS only

- **Xcode Command Line Tools:** `xcode-select --install`
- **Apple Developer account** (paid) — required for code signing and notarization.
- **Certificates** in Keychain:
  - `Developer ID Application: <Your Name or Org> (<Team ID>)` — for code signing.
  - `Developer ID Installer: <Your Name or Org> (<Team ID>)` — for `.pkg` builds (optional).

### Windows only

- **Microsoft C++ Build Tools** (Visual Studio Build Tools 2022, "Desktop development with C++").
- **Windows SDK** (included with VS Build Tools).
- **Code signing certificate** — an EV (Extended Validation) or OV (Organization Validation) certificate from a trusted CA (e.g., DigiCert, Sectigo). Self-signed certificates cause SmartScreen warnings.
- **`signtool.exe`** — ships with the Windows SDK.

---

## 2. Development Build

```bash
cd erc-budget
npm install
npm run tauri dev
```

This starts the Tauri dev server with hot-reload. The app window opens automatically. No installer is produced.

To verify the test suite before building:
```bash
cd src-tauri && cargo test          # 142 Rust tests
cd ..         && npm run test:coverage  # ~107 TypeScript tests + coverage report
```

---

## 3. Production Build

```bash
cd erc-budget
npm run tauri build
```

Tauri compiles the Rust backend in `--release` mode, runs `vite build` on the frontend, then bundles everything into platform-native installers.

Output directory:
```
erc-budget/src-tauri/target/release/bundle/
├── macos/
│   ├── ERC Budget Tool.app
│   └── ERC Budget Tool.dmg
└── windows/
    ├── ERC Budget Tool_1.0.0_x64.msi
    └── ERC Budget Tool_1.0.0_x64-setup.exe
```

Expected build time: 3–8 minutes on a modern machine (dominated by Rust `--release` compilation).

**Universal macOS binary (Apple Silicon + Intel):**
```bash
npm run tauri build -- --target universal-apple-darwin
```
This cross-compiles for both `aarch64-apple-darwin` and `x86_64-apple-darwin` and merges them into a universal binary. Requires both targets to be installed:
```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

---

## 4. macOS Build and Code Signing

### 4.1 Configure signing in `tauri.conf.json`

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM123456)",
      "notarizationCredentials": {
        "appleId": "your@email.com",
        "appleIdPassword": "@keychain:AC_PASSWORD",
        "teamId": "TEAM123456"
      }
    }
  }
}
```

Store the app-specific password in Keychain (`AC_PASSWORD`) rather than in the config file:
```bash
security add-generic-password -a "your@email.com" -w "xxxx-xxxx-xxxx-xxxx" -s "AC_PASSWORD"
```

### 4.2 Build and sign

```bash
npm run tauri build
```

Tauri automatically:
1. Compiles and bundles.
2. Signs the `.app` with `codesign` using the `signingIdentity`.
3. Submits the `.dmg` to Apple Notary Service.
4. Staples the notarization ticket to the `.dmg`.

If notarization fails, check the submission log:
```bash
xcrun notarytool log <submission-id> --keychain-profile AC_PASSWORD
```

### 4.3 Verify the build

```bash
codesign --verify --deep --strict --verbose=2 \
    "src-tauri/target/release/bundle/macos/ERC Budget Tool.app"

spctl --assess --type execute -v \
    "src-tauri/target/release/bundle/macos/ERC Budget Tool.app"
```

Both commands must exit with code 0.

---

## 5. Windows Build and Code Signing

### 5.1 Configure signing in `tauri.conf.json`

For a file-based certificate (PFX):
```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

For an EV certificate on a USB token, use signtool directly in a post-build script (Tauri's automatic signing does not support hardware tokens).

### 5.2 Build

On a Windows machine:
```bash
npm run tauri build
```

Or cross-compile from macOS (requires the Windows cross-compilation toolchain, more complex — native Windows build is strongly preferred for production releases).

### 5.3 Sign manually (if using a hardware EV token)

```powershell
signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 `
    /n "Your Organization Name" `
    "src-tauri\target\release\bundle\msi\ERC Budget Tool_1.0.0_x64.msi"
```

### 5.4 Verify

```powershell
signtool verify /pa "ERC Budget Tool_1.0.0_x64.msi"
```

---

## 6. Release Checklist

Complete these steps in order before tagging a release.

**Code quality:**
- [ ] All Rust tests pass: `cargo test`
- [ ] All TypeScript tests pass: `npm run test:coverage`
- [ ] Coverage thresholds met (80/75/80/80)
- [ ] `cargo clippy -- -D warnings` produces zero warnings
- [ ] `cargo fmt --check` passes (no unformatted code)
- [ ] ESLint produces zero errors

**Version bump:**
- [ ] Update `version` in `erc-budget/package.json`
- [ ] Update `version` in `erc-budget/src-tauri/Cargo.toml`
- [ ] Update `version` in `erc-budget/src-tauri/tauri.conf.json`
- [ ] All three must match

**Changelog:**
- [ ] Entry added to `CHANGELOG.md` with date and list of changes

**Build verification:**
- [ ] `npm run tauri build` completes without errors on macOS
- [ ] `npm run tauri build` completes without errors on Windows
- [ ] Smoke-test the `.dmg` on a clean macOS machine (no development environment)
- [ ] Smoke-test the `.msi` on a clean Windows machine
- [ ] Code signing verified on macOS (`codesign --verify`)
- [ ] Code signing verified on Windows (`signtool verify`)
- [ ] Notarization stapled on macOS (Gatekeeper passes without bypass)

**Smoke tests (manual):**
- [ ] App launches on macOS and Windows
- [ ] Can create a new project
- [ ] Can add a personnel role and see the live dashboard update
- [ ] Can add equipment with depreciation cap
- [ ] Can add an itemised trip (verify country lookup works)
- [ ] Can save the project to a `.ercbudget` file and reopen it
- [ ] Excel export opens correctly in Excel and LibreOffice
- [ ] PDF export opens correctly

**Tag and release:**
- [ ] `git tag v1.0.0 && git push origin v1.0.0`
- [ ] Upload `.dmg` and `.msi` to the distribution location

---

## 7. CI/CD Pipeline Skeleton

The following GitHub Actions workflow builds and tests on every push to `main` and produces release artifacts on version tags.

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  release:
    types: [created]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: npm ci
        working-directory: erc-budget
      - run: cargo test
        working-directory: erc-budget/src-tauri
      - run: npm run test:coverage
        working-directory: erc-budget

  build-macos:
    needs: test
    runs-on: macos-14                    # Apple Silicon runner
    if: github.event_name == 'release'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin,x86_64-apple-darwin
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: npm ci
        working-directory: erc-budget
      - name: Import certificates
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
        run: |
          echo "$APPLE_CERTIFICATE" | base64 --decode > cert.p12
          security import cert.p12 -P "$APPLE_CERTIFICATE_PASSWORD" \
              -k ~/Library/Keychains/login.keychain-db -T /usr/bin/codesign
      - name: Build
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_ID_PASSWORD: ${{ secrets.APPLE_ID_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: npm run tauri build -- --target universal-apple-darwin
        working-directory: erc-budget
      - uses: actions/upload-artifact@v4
        with:
          name: macos-dmg
          path: erc-budget/src-tauri/target/release/bundle/macos/*.dmg

  build-windows:
    needs: test
    runs-on: windows-latest
    if: github.event_name == 'release'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: npm ci
        working-directory: erc-budget
      - name: Build
        run: npm run tauri build
        working-directory: erc-budget
      - uses: actions/upload-artifact@v4
        with:
          name: windows-msi
          path: erc-budget/src-tauri/target/release/bundle/msi/*.msi
```

Store secrets in GitHub repository Settings → Secrets and variables → Actions:

| Secret | Content |
|---|---|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID Application certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` |
| `APPLE_ID` | Apple ID email for notarization |
| `APPLE_ID_PASSWORD` | App-specific password for the Apple ID |
| `APPLE_TEAM_ID` | 10-character Apple Team ID |

---

## 8. Auto-Update

Tauri v2 includes a built-in updater plugin (`tauri-plugin-updater`). To enable auto-update:

**Step 1 — Enable the plugin in `Cargo.toml`:**
```toml
[dependencies]
tauri-plugin-updater = "2"
```

**Step 2 — Register in `lib.rs`:**
```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

**Step 3 — Configure the update endpoint in `tauri.conf.json`:**
```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://releases.example.com/erc-budget/{{target}}/{{arch}}/latest.json"
      ],
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

**Step 4 — Generate a signing key pair:**
```bash
npm run tauri signer generate -- --output keys/
```
Store the private key securely (not in the repository). Add the public key to `tauri.conf.json`.

**Step 5 — Sign release artifacts:**
```bash
npm run tauri signer sign -- --private-key-path keys/private.key \
    src-tauri/target/release/bundle/macos/*.dmg
```

**Step 6 — Host a `latest.json` file** at the endpoint URL with the structure Tauri expects:
```json
{
  "version": "1.1.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2026-09-01T00:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "...",
      "url": "https://releases.example.com/erc-budget/ERC.Budget.Tool_1.1.0_aarch64.dmg"
    },
    "windows-x86_64": {
      "signature": "...",
      "url": "https://releases.example.com/erc-budget/ERC.Budget.Tool_1.1.0_x64.msi"
    }
  }
}
```

---

## 9. Distribution

**Institutional distribution (most common):** Share the signed `.dmg` and `.msi` directly via a university file share, SharePoint, or email. Users run the installer once; the app updates itself via the auto-updater thereafter (if configured).

**GitHub Releases:** Upload installers to a GitHub Release. This is a simple, free distribution channel. The release URL can be used as the auto-update endpoint.

**Direct download page:** A minimal HTML page with two download buttons (macOS / Windows) is sufficient. No app store listing is required.

**Do not distribute unsigned builds.** Unsigned macOS builds require users to bypass Gatekeeper manually (right-click → Open, then confirm), which is a poor experience for non-technical users. Unsigned Windows builds trigger a SmartScreen "Unknown publisher" warning, which most users will refuse.

---

## 10. Troubleshooting

**`cargo build` fails with "linker not found" on macOS:**
Install Xcode Command Line Tools: `xcode-select --install`.

**`cargo build` fails with "MSVC linker not found" on Windows:**
Install Microsoft C++ Build Tools. Run `rustup target add x86_64-pc-windows-msvc`.

**Notarization fails with "package is not signed":**
Ensure the `signingIdentity` in `tauri.conf.json` exactly matches the certificate Common Name in Keychain Access. Run `security find-identity -v -p codesigning` to list available identities.

**SmartScreen warning on Windows despite code signing:**
OV (Organization Validation) certificates still trigger SmartScreen warnings until the certificate builds reputation. EV (Extended Validation) certificates bypass SmartScreen immediately. Accumulate download volume to build reputation with an OV certificate.

**`npm run tauri build` fails with "Permission denied" on macOS:**
The output directory may be owned by root from a previous `sudo` run. Fix: `sudo chown -R $USER erc-budget/src-tauri/target/`.

**Universal binary build fails with "target not installed":**
```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
```

**WebView2 not found on Windows:**
WebView2 (required by Tauri on Windows) is pre-installed on Windows 10 (21H2+) and Windows 11. For older Windows 10 versions, the Tauri installer can bundle the WebView2 bootstrapper. Set in `tauri.conf.json`:
```json
{ "bundle": { "windows": { "webviewInstallMode": { "type": "embedBootstrapper" } } } }
```
