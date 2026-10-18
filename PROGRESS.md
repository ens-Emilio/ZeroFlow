# ZeroFlow — Progress Tracking

## Phase 0 — Risk Validation
- [x] Create-or-destroy overlay on hotkey (Ctrl+Alt+Z)
- [x] Lasso → crop selection (bounding box)
- [x] Fullscreen overlay capture via `gdk_pixbuf_get_from_surface`
- [x] Recovery via `get_pending_frame` (survive KWin/WebKitGTK hide/show)
- [x] Screen dim 45%
- [x] Hotkey: Ctrl+Alt+Z (Ctrl+Alt+Space blocked by Paseo)
- [x] Global shortcut via `xdg-desktop-portal` (Wayland) / `gnome-shell` (X11 fallback)

## Phase 1 — Core Pipeline
- [x] Crop frame to lasso bounding box → JPEG base64 (bbox, like the original)
- [x] trace.moe provider (POST https://api.trace.moe/search, detached Rust task)
- [x] Result via KDE notification (direct D-Bus; the Tauri plugin never reached the daemon)
- [x] Toolbar: Search anime, Copy image (wl-copy), Save image (rfd dialog), Close
- [x] Persistent ScreenCast session (ashpd) + PipeWire frame grab *(optional — current method works)*
- [x] QR scan (rxing on cropped frame; QR chip opens URLs in the browser)
- [x] OCR via Tesseract (leptess) on cropped frame (Text chip copies recognized text)
- [x] Text hit-test (port of `OcrTextHitTester`) → selectable words (TextOverlay.svelte)
- [x] Google Lens (direct upload to lens.google.com/v3/upload, opens results URL)
- [x] Yandex Images: upload flow port (via 0x0.st)
- [x] Pinterest: HTTP flow port (via 0x0.st)
- [x] Search text engine chip (busca trechos)
- [x] Translate chip (LibreTranslate text → clipboard)
- [x] Translate image chip (Google in-situ translation → preview replaces crop)
- [x] Settings window (real UI + tray menu) with JSON persistence

## Phase 2 - Visual Search
- [x] Google Lens: improve success rate (cookie warmup, bounded HTTP)
- [x] Visual search coordinator with one transient retry
- [x] Provider selector in toolbar / settings, including Pinterest
- [x] Browser fallback through xdg-open and desktop portal

## Phase 3 - Text, Translation, Music
- [x] Text search engines (Google/Bing/DDG/Kagi/Qwant/Startpage)
- [x] Translation: signed Google image protocol with fixture tests
- [x] Visible OCR translation result with clipboard copy feedback
- [x] Music recognition: PipeWire monitor -> 16 kHz mono -> Shazam API port
- [x] Music history and onboarding flow

## Phase 4 - Polish & Ship
- [x] Accessibility: toolbar role=toolbar, dialog roles, aria-busy, focus, visible focus outlines
- [x] Performance: lasso zoom is canvas-based; fingerprint off-main-thread with timeout
- [x] Documentation & packaging for AppImage and Debian bundles
- [x] Historical CTS reference moved to `reference/CTS/` and excluded from linguist stats

## Validation status
- [x] `pnpm check`, `pnpm build`, `cargo check` and `cargo test` pass
- [x] Final AppImage and Debian bundles generated
- [x] Release binary startup smoke passed; public provider hosts reachable
- [ ] Manual compositor, real audio and clean-machine validation require a target desktop environment
- [ ] Windows reference tests require WindowsDesktop runtime on Windows or a Windows VM
