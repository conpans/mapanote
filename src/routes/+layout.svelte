<script lang="ts">
  import "../app.css";
  import SearchModal from "$lib/components/SearchModal.svelte";
  import { showSearchModal, openSearch } from "$lib/stores/searchStore";

  // Keyboard shortcut: Ctrl+K or Cmd+K
  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "k") {
      e.preventDefault();
      openSearch();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<slot />

{#if $showSearchModal}
  <SearchModal onClose={() => showSearchModal.set(false)} />
{/if}
