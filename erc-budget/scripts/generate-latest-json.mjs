#!/usr/bin/env node
/**
 * Builds the `latest.json` manifest the Tauri updater plugin polls
 * (see tauri.conf.json's `plugins.updater.endpoints`, which points at
 * `.../releases/latest/download/latest.json`).
 *
 * Reads the two updater signature files produced by `pnpm tauri build`
 * (macOS locally, Windows via CI — both require TAURI_SIGNING_PRIVATE_KEY /
 * TAURI_SIGNING_PRIVATE_KEY_PASSWORD to be set so `createUpdaterArtifacts`
 * actually signs its output) and writes a manifest referencing each
 * platform's GitHub release download URL.
 *
 * Usage:
 *   node scripts/generate-latest-json.mjs \
 *     --version 1.5.0 \
 *     --notes "What's new in this release" \
 *     --macos-sig target/release/bundle/macos/M2-EU\ Budgeter.app.tar.gz.sig \
 *     --macos-asset "M2-EU Budgeter_1.5.0_aarch64.app.tar.gz" \
 *     --windows-sig /path/to/M2-EU\ Budgeter_1.5.0_x64-setup.exe.sig \
 *     --windows-asset "M2-EU Budgeter_1.5.0_x64-setup.exe" \
 *     --out latest.json
 *
 * `--macos-asset` / `--windows-asset` are the exact filenames the release
 * assets will be uploaded under (used to build the download URL — note the
 * macOS updater artifact is the `.app.tar.gz`, not the `.dmg`, so it must be
 * uploaded to the release as its own asset alongside the dmg).
 */
import { readFileSync, writeFileSync } from 'node:fs';

const REPO = 'Byyoldas/m2-eu-budgeter';

function arg(name, required = true) {
  const idx = process.argv.indexOf(`--${name}`);
  if (idx === -1 || !process.argv[idx + 1]) {
    if (required) throw new Error(`Missing required --${name}`);
    return undefined;
  }
  return process.argv[idx + 1];
}

const version = arg('version');
const notes = arg('notes');
const macosSigPath = arg('macos-sig', false);
const macosAsset = arg('macos-asset', false);
const windowsSigPath = arg('windows-sig', false);
const windowsAsset = arg('windows-asset', false);
const outPath = arg('out', false) ?? 'latest.json';

if (!macosSigPath && !windowsSigPath) {
  throw new Error('At least one of --macos-sig or --windows-sig is required.');
}

const platforms = {};
const tag = `v${version}`;

if (macosSigPath) {
  if (!macosAsset) throw new Error('--macos-asset is required when --macos-sig is given.');
  platforms['darwin-aarch64'] = {
    signature: readFileSync(macosSigPath, 'utf-8').trim(),
    url: `https://github.com/${REPO}/releases/download/${tag}/${encodeURI(macosAsset)}`,
  };
}
if (windowsSigPath) {
  if (!windowsAsset) throw new Error('--windows-asset is required when --windows-sig is given.');
  platforms['windows-x86_64'] = {
    signature: readFileSync(windowsSigPath, 'utf-8').trim(),
    url: `https://github.com/${REPO}/releases/download/${tag}/${encodeURI(windowsAsset)}`,
  };
}

const manifest = {
  version,
  notes,
  pub_date: new Date().toISOString(),
  platforms,
};

writeFileSync(outPath, JSON.stringify(manifest, null, 2) + '\n');
console.log(`Wrote ${outPath}:`);
console.log(JSON.stringify(manifest, null, 2));
