# ZeroFlow — Port Plan

## Phase 0 — Risk Validation (1 week)

**Goal:** Prove the core UX loop works on Linux before investing in features.

| Task | Status |
|---|---|
| Transparent overlay window (Tauri) on GNOME Wayland, KDE Wayland, Hyprland, X11 | ⬜ |
| Lasso drag draws live on Canvas 2D, pointer capture works | ⬜ |
| xdg-desktop-portal ScreenCast session persists across hotkey presses | ⬜ |
| Google Lens URL opens in default browser (fallback) | ⬜ |

**Exit criteria:** Press hotkey → overlay appears → drag lasso → release → toast says "captured" (no crash, no portal consent prompt every time).

## Phase 1 — Core Pipeline (2–3 weeks)

| Task | Dependencies |
|---|---|
| Persistent ScreenCast session (ashpd) + PipeWire frame grab | F0 |
| Crop frame to lasso polygon (imageproc) → JPEG base64 | F0 |
| trace.moe provider (REST, trivial) | — |
| QR scan (rxing on cropped frame) | crop |
| OCR via Tesseract (leptess) on cropped frame | crop |
| Text hit-test (port `OcrTextHitTester`) → selectable words | OCR |
| Toolbar chips: Copy image, Save image, QR chip, Translate | crop + OCR |
| Settings window (JSON + Tauri invoke) | — |

## Phase 2 — Visual Search (2–3 weeks)

| Task | Note |
|---|---|
| Google Lens: decide between (a) xdg-open, (b) WebKitGTK window with JS injection, (c) CEF | F0 validation |
| Yandex Images: port upload flow from CircleFlow | cookies + scrape |
| Pinterest: port HTTP flow from CircleFlow | — |
| Built-in results window (if Lens needs WebKit/CEF) | — |
| Provider selector in toolbar / settings | — |

## Phase 3 — Text, Translation, Music (2–3 weeks)

| Task | Source |
|---|---|
| Text search engines (Google/Bing/DDG/Kagi/Qwant/Startpage) | pure URL builders |
| Translation: port `GoogleImageTranslationSigner` + protocol | HTTP only |
| Music: PipeWire monitor → 16 kHz mono → Shazam API | `ShazamClient` port |
| Music history (local JSON) | — |
| Onboarding flow | — |

## Phase 4 — Polish & Ship (2 weeks)

- HiDPI, multi-monitor, Wayland screen-lock handling
- AppImage (primary), Flatpak (portal baked in), AUR
- Auto-start `.desktop`, systemd user service (optional)
- CI: GitHub Actions (linux build)

---

## Module Map (CircleFlow → ZeroFlow)

| CircleFlow (WPF/C#) | ZeroFlow (Tauri/Rust + Svelte) |
|---|---|
| `AppSettings`, `SettingsService` | `src-tauri/src/settings.rs` + `src/lib/stores/index.ts` |
| `LassoPathSampler`, `LassoBoundsCalculator` | `src/lib/overlay/lasso.ts` |
| `OverlayZoomCamera` | `src/lib/overlay/zoom.ts` |
| `OcrTextHitTester`, `TextSelectionRange` | `src/lib/overlay/text-select.ts` (Phase 1) |
| `TextSearchUrlBuilder` + engines | `src/lib/search/text-search-engines.ts` |
| `ShazamClient`, `ProgressiveMusicRecognizer` | `src-tauri/src/shazam.rs` + `cpal` |
| `TraceMoeProvider`, `YandexImagesProvider`, `PinterestProvider` | `src-tauri/src/providers.rs` |
| `GoogleLensBrowserOperation` | `src-tauri/src/providers.rs` (open_lens) |
| `GoogleImageTranslationSigner/Protocol` | `src-tauri/src/translate.rs` |
| `WindowsOcrRecognizer` | `src-tauri/src/ocr.rs` (Tesseract) |
| `QrCodeScanner` (ZXing.Net) | `src-tauri/src/qr.rs` (rxing) |
| `HotkeyRegistrar` / `HotkeyWindow` | Tauri `global-shortcut` plugin |
| `TrayIcon`, `NotificationCard` | `src-tauri/src/tray.rs` + `notify-rust` |
| `Velopack` updates | AppImage/Flatpak (no auto-update yet) |

---

## Open Questions / Risks

1. **Google Lens on Linux** — biggest risk. The original uses WebView2 with script injection. If WebKitGTK is blocked, CEF adds ~150 MB. Test early.
2. **Wayland global shortcuts** — no standard protocol. Tauri plugin uses `global-shortcuts-portal` (needs DE support). On GNOME, may require the user to set the shortcut in Settings → Keyboard → Shortcuts manually. Document this.
3. **Persistent ScreenCast** — portal consent is once per session. Keep the PipeWire node alive. If the stream dies (monitor change, sleep), restart silently.
4. **Overlay on Wayland** — transparent Tauri window may not get pointer events on some compositors. Fallback: fullscreen window with screenshot as backdrop (what CircleFlow does on Windows when DWM is weird).
5. **Shazam fingerprinting** — the original has a fingerprinting module. Verify if the HTTP API still works without it (it might, Shazam's public endpoint accepts raw audio chunks).

---

## Next Step

Run `pnpm install && pnpm tauri dev` inside the repo. The overlay window should appear and the lasso canvas should draw. If it doesn't, debug the Tauri transparent window on your compositor.