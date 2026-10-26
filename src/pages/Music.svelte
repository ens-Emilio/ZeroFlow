<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { backend } from "../lib/ipc.ts";
  import { musicState, showToast } from "../lib/stores/index.svelte.ts";

  type MusicResult = { status: string; track?: any; recognition?: any; error?: string };
  let busy = $state(false);
  let stopping = $state(false);
  let result = $state<MusicResult | null>(null);
  let history = $state<any[]>([]);

  onMount(() => {
    let unlisten: (() => void) | null = null;
    void (async () => {
      try { history = (await backend.getMusicHistory()) as any[]; } catch {}
      unlisten = await listen<MusicResult>("zeroflow://music-result", (event) => {
        result = event.payload;
        musicState.lastResult = event.payload;
        busy = false;
        stopping = false;
        if (event.payload.status === "matched") {
          showToast(`Matched: ${event.payload.track?.title ?? "?"} - ${event.payload.track?.artist ?? "?"}`);
          void refreshHistory();
        } else if (event.payload.status === "no_match") showToast("No match; try with louder audio", true);
        else if (event.payload.status === "rate_limited") showToast("Shazam rate-limited, try again later", true);
        else if (event.payload.status === "cancelled") showToast("Music listening stopped");
        else if (event.payload.status === "service_error") showToast(`Service error: ${event.payload.error ?? ""}`, true);
      });
    })();
    return () => { try { unlisten?.(); } catch {} };
  });

  async function clearHistory() {
    if (!window.confirm("Clear all saved music history? This cannot be undone.")) return;
    try { await backend.clearMusicHistory(); history = []; showToast("History cleared"); } catch (e) { showToast(String(e), true); }
  }

  async function refreshHistory() { try { history = (await backend.getMusicHistory()) as any[]; } catch {} }

  async function recognize() {
    if (busy) return;
    busy = true;
    result = null;
    try {
      await backend.startMusicRecognition();
      showToast("Listening... keep music playing (12s)");
    } catch (e) {
      result = { status: "service_error", error: String(e) };
      showToast(String(e), true);
      busy = false;
    }
  }

  async function stop() {
    if (!busy || stopping) return;
    stopping = true;
    try {
      await backend.stopMusicRecognition();
    } catch (e) {
      stopping = false;
      showToast(`Could not stop music listening: ${e}`, true);
    }
  }

</script>
<main class="music" aria-label="Music recognition">
  <h1>ZeroFlow — Music</h1>
  <p class="hint">Plays from speakers/monitor. Captures what is currently audible.</p>
  <div class="actions">
    <button onclick={recognize} disabled={busy} aria-busy={busy}>{busy ? "Listening…" : "Identify music"}</button>
    {#if busy}<button class="secondary" onclick={stop} disabled={stopping}>{stopping ? "Stopping…" : "Stop"}</button>{/if}
  </div>
  {#if result}
    <section class="result">
      {#if result.status === "matched"}
        <div class="card">
          {#if result.recognition?.cover_art_url}<img src={result.recognition.cover_art_url} alt="" width="96" height="96" />{/if}
          <div>
            <div class="title">{result.recognition?.title ?? result.track?.title}</div>
            <div class="artist">{result.recognition?.artist ?? result.track?.artist}</div>
            {#if result.recognition?.album}<div class="album">{result.recognition.album}</div>{/if}
            {#if result.recognition?.share_url}<a href={result.recognition.share_url} target="_blank" rel="noreferrer">Open in Shazam</a>{/if}
          </div>
        </div>
      {:else if result.status === "no_match"}
        <p class="muted">No match. Make sure music is playing audibly and try again.</p>
      {:else if result.status === "rate_limited"}
        <p class="muted">Rate-limited by Shazam. Wait a minute and retry.</p>
      {:else}
        <p class="muted error">Error: {result.error ?? "unknown"}</p>
      {/if}
    </section>
  {/if}
  {#if history.length}
    <section class="history" aria-label="Listening history">
      <h2>History</h2>
      <button class="secondary small" onclick={clearHistory} aria-label="Clear history">Clear</button>
      {#each history as h}
        <div class="card small">
          {#if h.cover_url}<img src={h.cover_url} alt="" width="48" height="48" />{/if}
          <div>
            <div class="title">{h.title} - {h.artist}</div>
            {#if h.album}<div class="album">{h.album}</div>{/if}
            {#if h.shazam_url}<a href={h.shazam_url} target="_blank" rel="noreferrer">Shazam</a>{/if}
          </div>
        </div>
      {/each}
    </section>
  {/if}
</main>

<style>
  .music { padding: 1.25rem 1.5rem; max-width: 560px; font-family: system-ui, sans-serif; color: #e0e0e0; background: #1a1a1a; min-height: 100vh; }
  h1 { font-size: 1.15rem; font-weight: 700; margin: 0 0 0.25rem; }
  .hint { font-size: 0.85rem; opacity: 0.6; margin: 0 0 1rem; }
  .actions { display: flex; gap: 0.6rem; margin-bottom: 1rem; }
  button { padding: 0.55rem 1rem; border-radius: 999px; border: 0; background: #4285f4; color: #fff; font-weight: 600; cursor: pointer; }
  button:disabled { opacity: 0.5; cursor: default; }
  button.secondary { background: #3a3a3a; }
  .card { display: flex; gap: 0.9rem; align-items: center; background: #222; border-radius: 12px; padding: 0.9rem; }
  .card img { border-radius: 8px; }
  .title { font-weight: 700; }
  .artist { opacity: 0.8; }
  .album { opacity: 0.55; font-size: 0.85rem; }
  .muted { opacity: 0.6; }
  .muted.error { color: #f28b82; opacity: 1; }
  a { color: #8ab4f8; }
  :global(:focus-visible) { outline: 2px solid #4285f4; outline-offset: 2px; }
</style>
