<script lang="ts">
  import Settings from "./pages/Settings.svelte";
  let { mode = "overlay" }: { mode?: string } = $props();
</script>

{#if mode === "overlay"}
  <main class="overlay"></main>
{:else if mode === "settings"}
  <Settings />
{:else}
  <main class="music"><h1>ZeroFlow Music</h1></main>
{/if}

<style>
  main.overlay {
    position: fixed;
    inset: 0;
    background: transparent;
  }
  main.music {
    padding: 2rem;
    font-family: system-ui, sans-serif;
    color: #e0e0e0;
    background: #1a1a1a;
  }
  h1 { font-size: 1.25rem; font-weight: 600; }
</style>
