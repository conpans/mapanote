<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";

  export let onClose: () => void;

  interface SearchResult {
    note_id: string;
    country_slug: string;
    country_title: string;
    date: string;
    tags: string[];
    text: string;
    snippet: string;
    visibility: string;
    pinned: boolean;
  }

  let query = "";
  let results: SearchResult[] = [];
  let isSearching = false;
  let selectedIndex = 0;
  let searchInput: HTMLInputElement;

  onMount(() => {
    searchInput?.focus();
  });

  async function handleSearch() {
    if (query.trim().length < 2) {
      results = [];
      return;
    }

    isSearching = true;

    try {
      const searchResults = await invoke<SearchResult[]>("search_notes", {
        query: query.trim(),
      });
      results = searchResults;
      selectedIndex = 0;
    } catch (error) {
      console.error("Search failed:", error);
      results = [];
    } finally {
      isSearching = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === "Enter" && results.length > 0) {
      handleSelectResult(results[selectedIndex]);
    }
  }

  function handleSelectResult(result: SearchResult) {
    goto(`/country/${result.country_slug}`);
    onClose();
  }

  // Debounced search
  let searchTimeout: number;
  $: {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      if (query.trim().length >= 2) {
        handleSearch();
      } else {
        results = [];
      }
    }, 300) as unknown as number;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- Modal backdrop -->
<div
  class="fixed inset-0 bg-black bg-opacity-50 flex items-start justify-center pt-20 z-50"
  on:click={onClose}
  on:keydown={(e) => e.key === "Escape" && onClose()}
  role="button"
  tabindex="0"
>
  <!-- Search box -->
  <div
    class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl w-full max-w-2xl overflow-hidden"
    on:click|stopPropagation
    role="dialog"
    aria-modal="true"
  >
    <!-- Search input -->
    <div class="p-4 border-b border-gray-200 dark:border-gray-700">
      <div class="relative">
        <svg
          class="absolute left-3 top-3 w-5 h-5 text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
        <input
          bind:this={searchInput}
          bind:value={query}
          type="text"
          placeholder="Search notes..."
          class="w-full pl-10 pr-4 py-3 bg-gray-50 dark:bg-gray-700 border-0 rounded-lg
                 text-gray-900 dark:text-gray-100 placeholder-gray-500
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:outline-none"
        />
      </div>
    </div>

    <!-- Results -->
    <div class="max-h-96 overflow-y-auto">
      {#if isSearching}
        <div class="p-8 text-center text-gray-500 dark:text-gray-400">
          Searching...
        </div>
      {:else if query.trim().length < 2}
        <div class="p-8 text-center text-gray-500 dark:text-gray-400">
          Type at least 2 characters to search
        </div>
      {:else if results.length === 0}
        <div class="p-8 text-center text-gray-500 dark:text-gray-400">
          No results found for "{query}"
        </div>
      {:else}
        <div class="divide-y divide-gray-200 dark:divide-gray-700">
          {#each results as result, i (result.note_id)}
            <button
              on:click={() => handleSelectResult(result)}
              class="w-full text-left p-4 hover:bg-gray-50 dark:hover:bg-gray-700 transition
                     {i === selectedIndex
                ? 'bg-mapanote-blue-50 dark:bg-mapanote-blue-900/20'
                : ''}"
            >
              <div class="flex items-start justify-between mb-2">
                <div class="flex items-center gap-2">
                  <span class="font-semibold text-gray-900 dark:text-gray-100">
                    {result.country_title}
                  </span>
                  {#if result.pinned}
                    <span class="text-amber-600 dark:text-amber-400">📌</span>
                  {/if}
                </div>
                <span class="text-sm text-gray-500 dark:text-gray-400">
                  {result.date}
                </span>
              </div>

              {#if result.tags.length > 0}
                <div class="flex gap-2 mb-2">
                  {#each result.tags as tag}
                    <span
                      class="text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 px-2 py-1 rounded"
                    >
                      {tag}
                    </span>
                  {/each}
                </div>
              {/if}

              <p class="text-sm text-gray-700 dark:text-gray-300 line-clamp-2">
                {result.snippet}
              </p>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div
      class="p-3 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900"
    >
      <div
        class="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400"
      >
        <div class="flex items-center gap-4">
          <span><kbd class="kbd">↑↓</kbd> Navigate</span>
          <span><kbd class="kbd">Enter</kbd> Select</span>
          <span><kbd class="kbd">Esc</kbd> Close</span>
        </div>
        {#if results.length > 0}
          <span>{results.length} result{results.length !== 1 ? "s" : ""}</span>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .kbd {
    padding: 2px 6px;
    background: white;
    border: 1px solid #d1d5db;
    border-radius: 3px;
    font-family: monospace;
    font-size: 11px;
  }

  :global(.dark) .kbd {
    background: #374151;
    border-color: #4b5563;
  }

  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
