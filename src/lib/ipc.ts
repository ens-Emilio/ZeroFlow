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

  // Visual search coordinator: one provider flow with bounded retry policy.
  searchVisual: (providerId: string, jpegBase64: string) => invoke<unknown>("search_visual", { providerId, jpegBase64 }),

  // Utilities
  copyImage: (jpegBase64: string) => invoke<boolean>("copy_image", { jpegBase64 }),
  saveImage: (jpegBase64: string) => invoke<string | null>("save_image", { jpegBase64 }),
  scanQr: (pngBase64: string) => invoke<{ text: string; x: number; y: number; width: number; height: number } | null>("scan_qr", { pngBase64 }),
  openSearch: (query: string, engine: string) => invoke<void>("open_search", { query, engine }),
  ocr: (pngBase64: string, langTag: string) =>
    invoke<{ words: { text: string; x: number; y: number; width: number; height: number }[]; languageTag: string }>("ocr", { pngBase64, langTag }),

  // Music
  getMusicHistory: () => invoke<unknown[]>("get_music_history"),
  clearMusicHistory: () => invoke<void>("clear_music_history"),
  startMusicRecognition: () => invoke<{ status: string; track?: unknown; recognition?: unknown; error?: string }>("start_music_recognition"),
  stopMusicRecognition: () => invoke<void>("stop_music_recognition"),

  translateText: (text: string, targetLang: string) =>
    invoke<{ translatedText: string }>("translate_text", { text, targetLang }),
  translateImage: (jpegBase64: string, targetLang: string) =>
    invoke<{ translatedPngBase64: string }>("translate_image", { jpegBase64, targetLang }),
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
