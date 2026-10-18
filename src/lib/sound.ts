import { settings } from "./stores/index.svelte.ts";

export function playSound(name: string) {
  if (!settings.uiSounds || typeof Audio === "undefined") return;
  const audio = new Audio(`/sounds/${name}.wav`);
  audio.volume = 0.45;
  void audio.play().catch(() => {});
}
