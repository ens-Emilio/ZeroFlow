<div align="center">

# ZeroFlow

**Circle anything on screen. Results are ready before you let go.**

</div>

[Circle to Search](https://search.google/ways-to-search/circle-to-search/) from Android, now on your Linux desktop.
Fork of [CircleFlow](https://github.com/keekyslusus/CircleFlow) (Windows/WPF) rebuilt for Linux with Tauri + Svelte.

Press a shortcut, circle or select any area, and search it with Google Lens, Yandex Images, Pinterest or trace.moe. Recognize music with Shazam, read QR codes, translate on-screen text.

## Status

Functional Linux port. See [docs/PLAN_CONVERSION.md](docs/PLAN_CONVERSION.md) for the
conversion checklist and [PROGRESS.md](PROGRESS.md) for implementation status.

[![Languages](https://img.shields.io/github/languages/top/ens-Emilio/ZeroFlow)](https://github.com/ens-Emilio/ZeroFlow)

## Stack

- **Shell/core**: Rust (Tauri v2)
- **UI**: Svelte 5 (runes) + TypeScript, Canvas 2D overlay
- **Capture**: xdg-desktop-portal (ScreenCast) + PipeWire
- **Audio loopback**: PipeWire (for Shazam)
- **OCR**: Tesseract
- **QR**: rxing

## Build

Requirements: Rust, Node 20+, Tauri v2 CLI, plus system deps (`libtesseract`, `libpipewire`, `gtk3`, `webkit2gtk-4.1`).

```sh
pnpm install
pnpm tauri dev
```

## License

GPL-3.0-or-later (inherited from CircleFlow). See `LICENSE`, `THIRD_PARTY_NOTICES.txt`, `THIRD_PARTY_LICENSES`.
