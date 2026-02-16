# Release Checklist

## 1) Pre-release checks

- Ensure local branch is up to date:

```bash
git checkout main
git pull origin main
```

- Confirm app builds and tests pass:

```bash
npm install
npm run build
cd src-tauri
cargo check
cargo test acceptance_ -- --nocapture
cd ..
```

- Confirm manual desktop smoke checks:

```bash
npm run tauri dev
```

Checklist:
- Onboarding SecondBreakfast selection works.
- Run Now and Dry Run are functional.
- Watcher pause/resume works.
- Undo last run works for normal and conflict cases.

## 2) Version bump

Use the local script:

```bash
node tools/bump-version.mjs 0.1.1
```

Or npm script:

```bash
npm run bump:version -- 0.1.1
```

This updates:
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

## 3) Commit and tag

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to v0.1.1"
git tag v0.1.1
git push origin main --tags
```

Pushing tag `vX.Y.Z` triggers:
- `.github/workflows/release-windows.yml`
- It builds Windows installers and uploads them to the GitHub release automatically.

## 4) Automatic release output

Expected release assets:
- `*.msi` from `src-tauri/target/release/bundle/msi/`
- `*-setup.exe` from `src-tauri/target/release/bundle/nsis/`

You can also run the release workflow manually from GitHub Actions using an existing tag.

## 5) Optional local packaging check

```bash
npm run tauri build
```

See `docs/BUILD.md` for Ubuntu packaging steps.
