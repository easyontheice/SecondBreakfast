# Build and Packaging

## Windows (local native build)

Prerequisites:

- Node.js 20+
- Rust toolchain
- Visual Studio Build Tools (C++ workload)
- Tauri prerequisites for Windows

Steps:

```bash
npm install
npm run build
npm run tauri build
```

Expected installer output (default Tauri paths):

- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/` (if NSIS target is enabled/available)

## Ubuntu (local native build)

Prerequisites (Ubuntu 22.04+ suggested):

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  libwebkit2gtk-4.1-dev
```

Install Rust and Node.js, then:

```bash
npm install
npm run build
npm run tauri build
```

Expected installer output:

- `src-tauri/target/release/bundle/deb/`
- `src-tauri/target/release/bundle/appimage/` (if configured)

## Notes

- Cross-building Windows installers on Ubuntu or Ubuntu installers on Windows is not covered in this baseline.
- Keep build metadata in `src-tauri/tauri.conf.json` aligned with product/version before release.
