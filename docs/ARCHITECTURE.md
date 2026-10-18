# ZeroFlow — Architecture

## Bird's-eye view

```
┌──────────────────────── Linux desktop (GNOME/KDE/Hyprland/X11) ─────────────────────────┐
│                                                                                         │
│  ┌────────────── Tauri app (zeroflow binary) ───────────────┐                            │
│  │                                                          │                            │
│  │  ┌── Rust backend (src-tauri/src) ──┐   ┌── Frontend ──┐ │                            │
│  │  │ settings    JSON @ XDG_CONFIG   │◄──┤ Svelte 5      │ │                            │
│  │  │ capture     portal + PipeWire   │IPC│ runes stores  │ │                            │
│  │  │ providers   tracemoe/yandex/    │   │ lasso canvas  │ │                            │
│  │  │             pinterest/lens      │   │ zoom camera   │ │                            │
│  │  │ ocr         Tesseract (leptess) │   │ toolbar/toast │ │                            │
│  │  │ qr          rxing               │   │ settings UI   │ │                            │
│  │  │ shazam      HTTP + audio loop   │   └───────────────┘ │                            │
│  │  │ translate   signed Google proto │                     │                            │
│  │  │ tray        notify-rust, SNI    │                     │                            │
│  │  └─────────────────────────────────┘                     │                            │
│  └──────────────────────────────────────────────────────────┘                            │
│                                                                                         │
│  xdg-desktop-portal (ScreenCast)  PipeWire  StatusNotifier  libnotify                   │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

## Windows

| Label     | Purpose                              | Notes                                        |
|-----------|--------------------------------------|----------------------------------------------|
| `overlay` | Transparent fullscreen lasso overlay | Created hidden; shown on hotkey              |
| `settings`| Settings dialog                       | Material-style, single instance              |
| `music`   | Music recognition results + history   | Shown when a track is found or from tray      |

The frontend routes by window label (`getCurrentWindow().label` in `src/main.ts`), so one
Vite bundle serves all three windows.

## Selection flow (the core loop)

```
hotkey (global-shortcut plugin)
  → toggle_overlay: show overlay window
  → frontend asks Rust to grab_frame()          [Phase 0: portal session must persist]
  → backdrop <img> shows the frozen screen
  → user drags: LassoPath samples pointer (src/lib/overlay/lasso.ts)
  → release: bounds + polygon → crop_selection() → JPEG base64
  → toolbar: provider buttons, copy, save, translate, QR chip
  → provider flow (providers.rs) or clipboard (tray.rs / OS)
  → overlay hides
```

Everything after `crop_selection` is provider work; the overlay itself knows nothing
about search engines.

## Capture strategy

- **Wayland (and X11, uniform path):** `xdg-desktop-portal` ScreenCast. One consent
  prompt; ZeroFlow keeps the PipeWire stream node alive for instant capture.
- **X11 only fallback (Phase 4 if needed):** direct XSHM/XFixes grab, no portal.
- The portal session is the reason Flatpak is an attractive target: the sandbox
  already speaks portal natively.

## Audio (Shazam)

PipeWire monitor stream (system output, not microphone) → downmix mono → 16 kHz →
POST to Shazam endpoint with the Pixel user-agent (see `shazam.rs`). The throttle
and location-spoof behavior port from `ShazamRequestThrottle`/`ShazamLocationProvider`.

## Settings

`$XDG_CONFIG_HOME/zeroflow/settings.json`, camelCase JSON mirroring CircleFlow's
`AppSettings` record field-for-field, so defaults and validation port 1:1.

## What deliberately does NOT port

- Velopack updater → AppImage/Flatpak packages instead (distro update flows).
- Win32 tray/notification code → StatusNotifierItem + libnotify.
- WebView2 → default browser or a built-in WebKitGTK window (Phase 2 decision).
- WPF everything → Svelte (see docs/PLAN.md module map).

## Search coordination

Visual searches enter `src-tauri/src/search_coordinator.rs`. The coordinator selects one
provider, applies one retry only for transient failures, and leaves browser navigation to
the existing provider operations. Browser-host classes from the Windows implementation are
not ported: Wayland and X11 use `xdg-open` through the desktop portal.

## Historical reference

The Windows/WPF implementation is retained under `reference/CTS/` for behavior comparison
and license provenance. It is vendored and excluded from GitHub language statistics. The
Linux runtime does not depend on its Win32, COM, WebView2, or STA dispatcher code.

## Packaging

`src-tauri/tauri.conf.json` builds AppImage and Debian packages. See
`docs/PACKAGING.md` for required host dependencies and validation steps.
