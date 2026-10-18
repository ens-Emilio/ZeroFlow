<script lang="ts">
  let { onDone }: { onDone: () => void } = $props();
</script>
<div class="onb" role="dialog" aria-label="Welcome to ZeroFlow">
  <h1>ZeroFlow</h1>
  <p>Circle to Search for Linux</p>
  <ol>
    <li>Press <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>Z</kbd> anywhere to capture the screen.</li>
    <li>Draw a lasso around what you want to search.</li>
    <li>Pick a chip: Search, Text, Translate, Music, QR, Copy, Save.</li>
  </ol>
  <p class="muted">Grant "Global Shortcuts" permission when asked by the portal.</p>
  <button onclick={onDone}>Got it</button>
</div>
<style>
  .onb { position: fixed; inset: 0; display: grid; place-content: center; background: rgba(0,0,0,0.78); color: #e4e4e7; text-align: center; gap: 0.6rem; z-index: 50; padding: 2rem; cursor: default; }
  h1 { font-size: 1.6rem; }
  ol { text-align: left; max-width: 28rem; margin: 0 auto; line-height: 1.6; }
  kbd { background: #27272a; border: 1px solid #3f3f46; border-radius: 4px; padding: 1px 6px; }
  .muted { opacity: 0.6; font-size: 0.85rem; }
  button { margin-top: 0.8rem; background: #4285f4; color: #fff; border: 0; border-radius: 999px; padding: 0.6rem 1.4rem; font-weight: 600; cursor: pointer; }
</style>
