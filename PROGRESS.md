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

## Phase 2 — Visual Search (Optional Refinements)
- [ ] Google Lens: improve success rate (cookie warmup, better parsing)
- [ ] Built-in results window (only if Lens needs WebKit/CEF)
- [ ] Provider selector in toolbar / settings

## Phase 3 — Text, Translation, Music
- [x] Music recognition: parec monitor → 16 kHz mono → Shazam API port (SongRec fingerprint+codec+client, progressive 4/8/12s)
- [ ] Music history (local JSON)
- [ ] Onboarding flow

## Phase 4 — Polish & Ship
- [ ] Performance & memory profiling (long-run)
- [ ] Accessibility (focus order, contrast, screen reader labels)
- [ ] Documentation & packaging (Flathub, Snap, AppImage, .deb, .rpm)

