// Typed bridge to the Rust backend. One file so the surface stays visible.

import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";

export interface SelectionBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Point {
  x: number;
  y: number;
}

export interface SearchResult {
  provider: "tracemoe" | "yandex" | "lens" | "pinterest";
  url?: string;
  payload?: unknown;
}

export const backend = {
  // Settings
  getSettings: () => invoke<Record<string, unknown>>("get_settings"),
  setSettings: (patch: Record<string, unknown>) => invoke<void>("set_settings", { patch }),

  // Overlay lifecycle
  startOverlay: () => invoke<void>("start_overlay"),
  closeOverlay: () => invoke<void>("close_overlay"),
  debugLog: (message: string) => invoke<void>("debug_log", { message }),
  openUrl: (url: string) => invoke<void>("open_url", { url }),

  // Capture: given the lasso polygon (CSS px), crop the cached screenshot
  // and return it as JPEG base64 (no data: prefix).
  cropSelection: (polygon: Point[]) => invoke<string>("crop_selection", { polygon }),

  // Providers
  searchTraceMoe: (jpegBase64: string) => invoke<unknown>("search_tracemoe", { jpegBase64 }),
  submitSelection: (jpegBase64: string) => invoke<void>("submit_selection", { jpegBase64 }),
  searchYandex: (jpegBase64: string) => invoke<unknown>("search_yandex", { jpegBase64 }),
  openLens: (jpegBase64: string) => invoke<unknown>("open_lens", { jpegBase64 }),

  // Utilities
  copyImage: (jpegBase64: string) => invoke<boolean>("copy_image", { jpegBase64 }),
  saveImage: (jpegBase64: string) => invoke<string | null>("save_image", { jpegBase64 }),
  scanQr: (pngBase64: string) => invoke<{ text: string; x: number; y: number; width: number; height: number } | null>("scan_qr", { pngBase64 }),
  ocr: (pngBase64: string, langTag: string) => invoke<unknown>("ocr", { pngBase64, langTag }),

  // Music
  startMusicRecognition: () => invoke<void>("start_music_recognition"),
  stopMusicRecognition: () => invoke<void>("stop_music_recognition"),

  // Translation
  translateSelection: (jpegBase64: string, targetLang: string) =>
    invoke<unknown>("translate_selection", { jpegBase64, targetLang }),
};

export const events = {
  onHotkey: (handler: () => void) => listen("zeroflow://hotkey", () => handler()),
  onMusicResult: (handler: (payload: unknown) => void) =>
    listen("zeroflow://music-result", (e) => handler(e.payload)),
  onToast: (handler: (payload: { message: string; isError?: boolean }) => void) =>
    listen<{ message: string; isError?: boolean }>("zeroflow://toast", (e) => handler(e.payload)),
  onOverlayFrame: (handler: (dataUrl: string) => void) =>
    listen<string>("overlay-frame", (e) => handler(e.payload)),
  onOverlayToggle: (handler: (dataUrl: string) => void) =>
    listen<string>("toggle-overlay", (e) => handler(e.payload)),
};

export { emit };
