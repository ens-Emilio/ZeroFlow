# ZeroFlow Linux packaging

ZeroFlow is packaged by Tauri for the desktop environments supported by the xdg desktop
portal. The release configuration emits both an AppImage and a Debian package.

## Build prerequisites

Install Rust, Node.js 20 or newer, pnpm, and the native libraries listed in
[CONTRIBUTING.md](../CONTRIBUTING.md). The AppImage target also needs the host tooling used
by Tauri's bundler. On Debian or Ubuntu, install `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`,
`libtesseract-dev`, `libleptonica-dev`, `libpipewire-0.3-dev`, `libclang-dev`,
`build-essential`, `curl`, `wget`, and `file`.

## Reproducible local build

```sh
pnpm install --frozen-lockfile
pnpm tauri build
```

The artifacts are written below `src-tauri/target/release/bundle/`:

- `deb/` contains the Debian package.
- `appimage/` contains the AppImage.

The bundle includes the application icon and the optional sound resources declared in
`src-tauri/tauri.conf.json`.

## Validation

Run the checks before publishing:

```sh
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
pnpm check
pnpm build
pnpm tauri build
```

The final local validation also started the release binary for five seconds and confirmed
HTTP reachability for Lens, trace.moe, and 0x0.st. A compositor smoke test still requires a
real keyboard/session: the current Wayland compositor rejects the virtual keyboard protocol.

For a clean-machine smoke test, install the Debian package or run the AppImage, grant the
Global Shortcuts and ScreenCast portal permissions, then verify `Ctrl+Alt+Z`, lasso capture,
copy/save, one visual search, OCR, QR, translation, and music recognition. The visual-search
browser opens through the user's default browser with `xdg-open`; no WebView2 or Windows
browser host is bundled.

The Windows reference test suite compiles with .NET 9 and `EnableWindowsTargeting=true`, but
cannot run on Linux because it requires `Microsoft.WindowsDesktop.App 9.0`. Run it on Windows
or a Windows VM.

The package does not include an automatic updater. Users update through the package manager
or by replacing the AppImage with a newer release.
