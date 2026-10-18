# Contributing to ZeroFlow

ZeroFlow is a Linux port of [CircleFlow](https://github.com/keekyslusus/CircleFlow) (Windows/WPF),
rebuilt with Tauri v2 + Svelte 5 + Rust. Ports of upstream logic must keep the port
provenance in the file header where the original file had one.

## Setup

System dependencies (Debian/Ubuntu names):

```sh
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev \
  libtesseract-dev libleptonica-dev \
  libpipewire-0.3-dev libclang-dev \
  build-essential curl wget file
```

Then:

```sh
pnpm install
pnpm tauri dev
```

## Layout

src-tauri/src/     Rust backend: settings, capture, providers, ocr, qr, shazam, translate, tray
src/lib/           Svelte frontend: overlay (lasso, zoom), stores, search engines, IPC types
docs/PLAN.md       Phased port plan and module map
reference/CTS/     Windows upstream reference - do not build on Linux
tests/             Windows compatibility tests retained for upstream reference

## Conventions

- One command per concern in Rust; keep `#[tauri::command]` functions thin, real logic in plain functions next to them.
- Frontend state is Svelte 5 runes (`$state`, `$derived`); no state libraries.
- Overlay drawing is Canvas 2D only; DOM/Svelte only for toolbar, toasts and settings chrome.
- When porting a CircleFlow file, note the source file in a comment at the top:
  `// Port of reference/CTS/Capture/LassoPathSampler.cs`
- User-visible strings live in one place per window; do not scatter literals.
- Match CircleFlow's behavior first, deviate only with a note in `docs/PLAN.md`.

## License

GPL-3.0-or-later, inherited from CircleFlow. Contributions are accepted under the same license.
