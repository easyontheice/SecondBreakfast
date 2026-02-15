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
- Onboarding sortRoot selection works.
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

## 4) Build installers

```bash
npm run tauri build
```

Expected artifacts (Windows):
- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/`

See `docs/BUILD.md` for Ubuntu packaging steps.

## 5) Publish release

- Create GitHub release from tag `vX.Y.Z`.
- Upload installer artifacts.
- Include release notes with key changes and known issues.
