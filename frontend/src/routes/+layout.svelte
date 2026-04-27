<script lang="ts">
  import Navbar from '$lib/components/Navbar.svelte';
</script>

<div class="app">
  <Navbar />
  <main class="main-content">
    <slot />
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: #1a1a1a;
    color: #d4d4d4;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(a) {
    color: inherit;
    text-decoration: none;
  }

  .app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .main-content {
    flex: 1;
    padding: 24px 20px;
    max-width: 1400px;
    margin: 0 auto;
    width: 100%;
  }
</style>
