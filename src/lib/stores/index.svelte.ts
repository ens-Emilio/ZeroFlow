// Minimal runes stores. Svelte 5 uses $state/$derived directly in components;
// this file holds app-level singletons only.

import type { Point } from "../overlay/lasso.ts";

export interface OcrWord {
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Settings {
  hotkeyGesture: string;
  ignoreHotkeyInFullscreen: boolean;
  uiSounds: boolean;
  scanQrCodes: boolean;
  overlayZoom: boolean;
  maxLongSidePx: number;
  paddingPx: number;
  hideDelayMilliseconds: number;
  lassoMinDiagonalPx: number;
  ocrLanguageTag: string;
  textSearchEngineId: string;
  textSearchInBuiltInBrowser: boolean;
  browserDataCleanupDays: number;
  saveMusicHistory: boolean;
  musicHistoryRetentionDays: number;
  searchProviderId: string;
  hiddenSearchProviderIds: string[];
  appLanguageTag: string;
  targetLang: string;
  onboardingCompleted: boolean;
}

export const settings = $state<Settings>({
  hotkeyGesture: "Ctrl+Alt+Z",
  ignoreHotkeyInFullscreen: true,
  uiSounds: true,
  scanQrCodes: true,
  overlayZoom: true,
  maxLongSidePx: 1600,
  paddingPx: 8,
  hideDelayMilliseconds: 60,
  lassoMinDiagonalPx: 12,
  ocrLanguageTag: "",
  textSearchEngineId: "matchimagesearch",
  textSearchInBuiltInBrowser: false,
  browserDataCleanupDays: 28,
  saveMusicHistory: true,
  musicHistoryRetentionDays: 28,
  searchProviderId: "googlelens",
  hiddenSearchProviderIds: ["yandeximages"],
  targetLang: "pt-BR",
  appLanguageTag: "",
  onboardingCompleted: false,
});

// Mutable overlay state during a drag session.
export interface OverlaySession {
  active: boolean;
  mode: "lasso" | "text" | "music" | "widget" | null;
  screenshotDataUrl: string | null;   // full-screen PNG/JPEG data URL
  screenshotNaturalSize: { w: number; h: number } | null;
  lassoPolygon: Point[] | null;
  selectionBounds: { x: number; y: number; width: number; height: number } | null;
  // Focused crop shown after the lasso completes, before choosing an action.
  croppedDataUrl: string | null;
  // OCR words for the selectable overlay (Text chip → Select words).
  ocrWords: OcrWord[];
}

export const overlaySession = $state<OverlaySession>({
  active: false,
  mode: null,
  screenshotDataUrl: null,
  screenshotNaturalSize: null,
  lassoPolygon: null,
  selectionBounds: null,
  croppedDataUrl: null,
  ocrWords: [],
});

export const musicState = $state({
  listening: false,
  lastResult: null as unknown | null,
  history: [] as unknown[],
});

export const toast = $state({ message: "", isError: false, visible: false });

export function showToast(message: string, isError = false) {
  toast.message = message;
  toast.isError = isError;
  toast.visible = true;
  setTimeout(() => (toast.visible = false), 3000);
}