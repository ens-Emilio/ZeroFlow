# Project progress checklist

This file is generated from the plan; the automation only changes the
checkboxes when the work is truly done. Treat it as the source of truth.

## Phase 0 — Risk Validation

- [x] Transparent overlay window works on KDE Wayland (validated 2026-10-13; GNOME/Hypr/X11 untested)
- [x] Lasso drag draws live on Canvas 2D, pointer capture works (Escape closes overlay)
- [x] xdg-desktop-portal Screenshot (one-shot, ashpd) delivers the frozen-frame backdrop on each hotkey press
- [ ] Google Lens URL opens in default browser (fallback for now)

Runtime notes (KDE Wayland / KWin / WebKitGTK 4.1):
- `WEBKIT_DISABLE_DMABUF_RENDERER=1` is set in main.rs: without it every
  show() of a webview dies with Wayland protocol error 71 (Gdk-Message).
- Overlay windows are never shown/hidden in place: each hotkey press builds a
  fresh fullscreen window (WebviewWindowBuilder) and closing destroys it
  (close(), never hide() - a hidden overlay leaves a dead handle on this stack
  and get_webview_window returns None afterwards).
- The frame is captured BEFORE the overlay is built so the screenshot never
  contains the overlay itself; the frontend recovers the pending frame via
  get_pending_frame() on mount (the page may load after toggle-overlay fired).
- Hotkey: Ctrl+Alt+Z via the GlobalShortcuts portal (Ctrl+Alt+Space conflicts
  with another running app and never reaches us).

## Phase 1 — Core Pipeline

- [x] Crop frame to lasso bounding box → JPEG base64 (bbox, like the original)
- [x] trace.moe provider (POST https://api.trace.moe/search, detached Rust task)
- [x] Result via KDE notification (direct D-Bus; the Tauri plugin never reached the daemon)
- [x] Toolbar chips: Search anime, Copy image (wl-copy), Save image (rfd dialog)
- [ ] Persistent ScreenCast session (ashpd) + PipeWire frame grab
- [x] QR scan (rxing on cropped frame; QR chip opens URLs in the browser)
- [x] OCR via Tesseract (leptess) on cropped frame (Text chip copies text)
- [ ] Text hit-test (port of `OcrTextHitTester`) → selectable words
- [x] QR chip on the toolbar
- [x] Settings window (JSON + Tauri invoke, Settings.svelte + tray menu)

## Phase 2 — Visual Search

- [ ] Google Lens: decision (a) xdg-open, (b) WebKitGTK + JS, (c) CEF
- [ ] Yandex Images: upload flow port
- [ ] Pinterest: HTTP flow port
- [ ] Built-in results window (only if Lens needs WebKit/CEF)
- [ ] Provider selector in toolbar / settings

## Phase 3 — Text, Translation, Music

- [ ] Text search engines (Google/Bing/DDG/Kagi/Qwant/Startpage) — pure URL builders
- [ ] Translation: `GoogleImageTranslationSigner` + protocol port
- [ ] Music: PipeWire monitor → 16 kHz mono → Shazam API port
- [ ] Music history (local JSON)
- [ ] Onboarding flow

## Phase 4 — Polish & Ship

- [ ] HiDPI, multi-monitor handling
- [ ] Wayland screen-lock: overlay does not appear when screen is locked (by design)
- [ ] AppImage (primary build)
- [ ] Flatpak (portal ready)
- [ ] AUR (PKGBUILD)
- [ ] Auto-start `.desktop` file
- [ ] GitHub Actions: linux build matrix

## Licensing

- [ ] GPL-3.0-or-later LICENSE file present
- [ ] THIRD_PARTY_NOTICES.txt and THIRD_PARTY_LICENSES copied from upstream
- [ ] All new files carry the SPDX header or a short comment noting GPL-3.0-or-later
- [ ] No new dependencies with incompatible licenses
