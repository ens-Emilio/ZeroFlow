# Project progress checklist

This file is generated from the plan; the automation only changes the
checkboxes when the work is truly done. Treat it as the source of truth.

## Phase 0 — Risk Validation

- [ ] Transparent overlay window works on GNOME Wayland, KDE Wayland, Hyprland, X11
- [ ] Lasso drag draws live on Canvas 2D, pointer capture works (Escape cancels)
- [ ] xdg-desktop-portal ScreenCast session persists across hotkey presses
- [ ] Google Lens URL opens in default browser (fallback for now)

## Phase 1 — Core Pipeline

- [ ] Persistent ScreenCast session (ashpd) + PipeWire frame grab
- [ ] Crop frame to lasso polygon (imageproc) → JPEG base64
- [ ] trace.moe provider (REST)
- [ ] QR scan (rxing on cropped frame)
- [ ] OCR via Tesseract (leptess) on cropped frame
- [ ] Text hit-test (port of `OcrTextHitTester`) → selectable words
- [ ] Toolbar chips: Copy image, Save image, QR chip, Translate
- [ ] Settings window (JSON + Tauri invoke)

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
