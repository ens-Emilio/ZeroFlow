import { mount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App.svelte";
import Overlay from "./lib/overlay/Overlay.svelte";

const label = getCurrentWindow().label;

// Route by window label: overlay, settings, music.
const target = document.getElementById("app")!;

if (label === "settings") {
  mount(App, { target, props: { mode: "settings" } });
} else if (label === "music") {
  mount(App, { target, props: { mode: "music" } });
} else {
  mount(Overlay, { target });
}
