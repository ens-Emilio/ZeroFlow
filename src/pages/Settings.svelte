<script lang="ts">
  import { settings } from "../lib/stores/index.svelte.ts";
  import { backend } from "../lib/ipc.ts";

  let loaded = $state(false);
  let savedToast = $state(false);
  let saveTimer: ReturnType<typeof setTimeout>;

  $effect(() => {
    void backend.getSettings().then((data) => {
      Object.assign(settings, data);
      loaded = true;
    });
  });

  function save() {
    void backend.setSettings({ ...settings }).then(() => {
      savedToast = true;
      clearTimeout(saveTimer);
      saveTimer = setTimeout(() => (savedToast = false), 1500);
    });
  }

  const engines = ["google", "bing", "duckduckgo", "kagi", "qwant", "startpage"];
</script>

<main class="settings">
  <h1>ZeroFlow Settings</h1>
  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section>
      <h2>Hotkey</h2>
      <label class="row">
        <span>Overlay shortcut</span>
        <input type="text" bind:value={settings.hotkey} placeholder="Ctrl+Alt+Z" />
      </label>
      <label class="row checkbox">
        <input type="checkbox" bind:checked={settings.ignoreHotkeyInFullscreen} />
        <span>Ignore hotkey while a fullscreen app has focus</span>
      </label>
    </section>

    <section>
      <h2>Overlay</h2>
      <label class="row">
        <span>UI sounds</span>
        <input type="checkbox" bind:checked={settings.uiSounds} />
      </label>
      <label class="row">
        <span>Scan QR codes automatically</span>
        <input type="checkbox" bind:checked={settings.scanQrCodes} />
      </label>
      <label class="row">
        <span>Zoom while drawing the lasso</span>
        <input type="checkbox" bind:checked={settings.overlayZoom} />
      </label>
      <label class="row">
        <span>Lasso minimum diagonal (px)</span>
        <input type="number" bind:value={settings.lassoMinDiagonalPx} min="1" />
      </label>
    </section>

    <section>
      <h2>Text</h2>
      <label class="row">
        <span>OCR language (Tesseract tag, e.g. eng, por)</span>
        <input type="text" bind:value={settings.ocrLanguageTag} placeholder="eng" />
      </label>
      <label class="row">
        <span>Text search engine</span>
        <select bind:value={settings.textSearchEngine}>
          {#each engines as e}<option value={e}>{e}</option>{/each}
        </select>
      </label>
    </section>

    <section>
      <h2>Search</h2>
      <label class="row">
        <span>Image search provider</span>
        <select bind:value={settings.searchProviderId}>
          <option value="googlelens">Google Lens</option>
          <option value="yandeximages">Yandex Images</option>
          <option value="tracemoe">trace.moe</option>
        </select>
      </label>
    </section>

    <footer>
      <button onclick={save}>Save</button>
      {#if savedToast}<span class="saved">Saved</span>{/if}
    </footer>
  {/if}
</main>

<style>
  .settings {
    padding: 1.5rem 2rem;
    max-width: 560px;
    font-family: system-ui, sans-serif;
    color: #e4e4e7;
    background: #18181b;
    min-height: 100vh;
  }
  h1 { font-size: 1.25rem; margin: 0 0 1rem; }
  h2 { font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.06em;
       color: #a1a1aa; margin: 0 0 0.6rem; }
  section {
    border-top: 1px solid #27272a;
    padding: 1rem 0;
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.35rem 0;
  }
  .row.checkbox, .row span { font-size: 0.9rem; }
  input[type="text"], input[type="number"], select {
    background: #27272a;
    border: 1px solid #3f3f46;
    color: #e4e4e7;
    border-radius: 6px;
    padding: 0.35rem 0.6rem;
    font-size: 0.9rem;
    width: 14rem;
  }
  input[type="number"] { width: 6rem; }
  input[type="checkbox"] { transform: scale(1.15); }
  footer {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding-top: 1.2rem;
    border-top: 1px solid #27272a;
  }
  button {
    background: #7c5cff;
    color: white;
    border: none;
    border-radius: 8px;
    padding: 0.5rem 1.4rem;
    font-size: 0.9rem;
    cursor: pointer;
  }
  button:hover { background: #6a4be8; }
  .saved { color: #4ade80; font-size: 0.9rem; }
  .muted { color: #a1a1aa; }
</style>
