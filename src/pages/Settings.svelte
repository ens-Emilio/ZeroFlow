<script lang="ts">
  import { settings } from "../lib/stores/index.svelte.ts";
  import { backend } from "../lib/ipc.ts";
  import Onboarding from "../lib/overlay/Onboarding.svelte";

  let loaded = $state(false);
  let savedToast = $state(false);
  let showOnboarding = $state(false);
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

  function finishOnboarding() {
    showOnboarding = false;
    settings.onboardingCompleted = true;
    save();
  }

  const engines = [
    { id: "matchimagesearch", name: "Match image search" },
    { id: "google", name: "Google" },
    { id: "bing", name: "Bing" },
    { id: "duckduckgo", name: "DuckDuckGo" },
    { id: "kagi", name: "Kagi" },
    { id: "qwant", name: "Qwant" },
    { id: "startpage", name: "Startpage" },
  ];
</script>

<main class="settings" aria-label="ZeroFlow settings">
  <h1>ZeroFlow Settings</h1>
  {#if !loaded}
    <p class="muted">Loading…</p>
  {:else}
    <section>
      <h2>Hotkey</h2>
      <label class="row">
        <span>Overlay shortcut</span>
        <input type="text" bind:value={settings.hotkeyGesture} placeholder="Ctrl+Alt+Z" aria-describedby="hotkey-restart" />
        <small id="hotkey-restart" class="muted">Changes take effect after restarting ZeroFlow.</small>
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
        <select bind:value={settings.textSearchEngineId}>
          {#each engines as engine}<option value={engine.id}>{engine.name}</option>{/each}
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
          <option value="pinterest">Pinterest</option>
        </select>
      </label>
    </section>
    <section class="preview-section" aria-live="polite">
      <h2>Live preview</h2>
      <div class="preview" class:preview-zoom={settings.overlayZoom}>
        <div class="preview-toolbar">
          <span class="preview-label">Selection actions</span>
          <span class="preview-provider">{settings.searchProviderId}</span>
        </div>
        <div class="preview-canvas">
          <div class="preview-lasso" aria-hidden="true"></div>
          <span class="preview-hint">Drag to select</span>
          {#if settings.overlayZoom}<span class="preview-lens">Zoom</span>{/if}
        </div>
        <div class="preview-meta">
          <span>Search: {engines.find((engine) => engine.id === settings.textSearchEngineId)?.name ?? settings.textSearchEngineId}</span>
          <span>Minimum: {settings.lassoMinDiagonalPx}px</span>
          <span>{settings.overlayZoom ? "Zoom enabled" : "Zoom disabled"}</span>
        </div>
      </div>
    </section>

    <section>
      <h2>General</h2>
      <label class="row">
        <span>Show onboarding now</span>
        <button class="secondary" type="button" onclick={() => { settings.onboardingCompleted = false; save(); showOnboarding = true; }}>Show onboarding</button>
      </label>
      <label class="row">
        <span>Keep music history</span>
        <input type="checkbox" bind:checked={settings.saveMusicHistory} onchange={save} />
      </label>
    </section>

    <footer>
      <button onclick={save}>Save</button>
      {#if savedToast}<span class="saved">Saved</span>{/if}
    </footer>
  {/if}
  {#if showOnboarding}
    <Onboarding onDone={finishOnboarding} />
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
  .preview-section { padding-bottom: 0.5rem; }
  .preview {
    overflow: hidden;
    border: 1px solid #3f3f46;
    border-radius: 8px;
    background: #111113;
  }
  .preview-toolbar, .preview-meta {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.6rem 0.75rem;
    color: #a1a1aa;
    font-size: 0.75rem;
  }
  .preview-label { color: #e4e4e7; }
  .preview-provider { color: #8ab4f8; }
  .preview-canvas {
    position: relative;
    height: 8rem;
    margin: 0 0.5rem;
    border: 1px solid #27272a;
    background: #202124;
  }
  .preview-lasso {
    position: absolute;
    left: 24%;
    top: 26%;
    width: 42%;
    height: 43%;
    border: 2px solid #4285f4;
    border-radius: 48% 52% 44% 56%;
    background: rgba(66, 133, 244, 0.12);
  }
  .preview-hint { position: absolute; right: 0.7rem; bottom: 0.6rem; color: #71717a; font-size: 0.75rem; }
  .preview-lens { position: absolute; left: 0.7rem; top: 0.6rem; color: #8ab4f8; font-size: 0.7rem; }
  .preview-meta { justify-content: flex-start; flex-wrap: wrap; border-top: 1px solid #27272a; }
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
  :global(:focus-visible) { outline: 2px solid #4285f4; outline-offset: 2px; }
</style>
