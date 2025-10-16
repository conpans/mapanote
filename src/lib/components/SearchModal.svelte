<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  interface SearchResult {
    country_slug: string;
    country_name: string;
    note_id: string;
    note_title: string;
    note_date: string;
    snippet: string;
    tags: string[];
  }

  let query = $state("");
  let results = $state<SearchResult[]>([]);
  let isSearching = $state(false);
  let selectedIndex = $state(0);

  // Auto-search when query changes
  $effect(() => {
    if (query.length >= 2) {
      performSearch();
    } else {
      results = [];
      selectedIndex = 0;
    }
  });

  async function performSearch() {
    if (query.trim().length < 2) return;

    isSearching = true;
    try {
      results = await invoke<SearchResult[]>("search_notes", {
        query: query.trim(),
      });
      selectedIndex = 0;
    } catch (error) {
      console.error("Search failed:", error);
      results = [];
    } finally {
      isSearching = false;
    }
  }

  function handleResultClick(result: SearchResult) {
    goto(`/country/${result.country_slug}`);
    onClose();
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
      e.preventDefault();
      handleResultClick(results[selectedIndex]);
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  onMount(() => {
    // Auto-focus search input
    const input = document.getElementById("search-input");
    if (input) {
      input.focus();
    }
  });
</script>

<div
  class="fixed inset-0 bg-black bg-opacity-50 flex items-start justify-center pt-20 z-50 p-4"
  on:click={handleBackdropClick}
  on:keydown={handleKeydown}
  role="button"
  tabindex="0"
>
  <div
    class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl max-w-2xl w-full max-h-[70vh] flex flex-col"
  >
    <!-- Search Input -->
    <div class="p-4 border-b border-gray-200 dark:border-gray-700">
      <div class="relative">
        <svg
          class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400"
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
          id="search-input"
          type="text"
          bind:value={query}
          placeholder="Search notes... (title or content)"
          class="w-full pl-10 pr-4 py-3 bg-gray-50 dark:bg-gray-900
                 border-0 rounded-lg
                 text-gray-900 dark:text-gray-100
                 placeholder-gray-400 dark:placeholder-gray-500
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:outline-none"
        />
      </div>
    </div>

    <!-- Results -->
    <div class="flex-1 overflow-y-auto p-2">
      {#if query.length < 2}
        <div class="p-8 text-center text-gray-500 dark:text-gray-400">
          <svg
            class="w-12 h-12 mx-auto mb-3 opacity-50"
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
          <p>Type at least 2 characters to search</p>
        </div>
      {:else if isSearching}
        <div class="p-8 text-center text-gray-500 dark:text-gray-400">
          <p>Searching...</p>
        </div>
      {:else if results.length === 0}
        <div class="p-8 text-center text-gray-500 dark:text-gray-400">
          <p>No notes found for "{query}"</p>
        </div>
      {:else}
        <div class="space-y-1">
          {#each results as result, index (result.note_id)}
            <button
              on:click={() => handleResultClick(result)}
              class="w-full text-left p-3 rounded-lg transition
                     {index === selectedIndex
                ? 'bg-mapanote-blue-100 dark:bg-mapanote-blue-900'
                : 'hover:bg-gray-100 dark:hover:bg-gray-700'}"
            >
              <div class="flex items-start justify-between mb-1">
                <h4 class="font-semibold text-gray-900 dark:text-gray-100">
                  {result.note_title}
                </h4>
                <span class="text-xs text-gray-500 dark:text-gray-400 ml-2">
                  {result.note_date}
                </span>
              </div>

              <p
                class="text-sm text-gray-600 dark:text-gray-400 mb-2 line-clamp-2"
              >
                {result.snippet}
              </p>

              <div class="flex items-center gap-2 flex-wrap">
                <span
                  class="text-xs px-2 py-1 rounded bg-gray-200 dark:bg-gray-700
                       text-gray-700 dark:text-gray-300"
                >
                  {result.country_slug}
                </span>
                {#each result.tags as tag}
                  <span
                    class="text-xs px-2 py-1 rounded bg-mapanote-blue-100 dark:bg-mapanote-blue-900
                         text-mapanote-blue-700 dark:text-mapanote-blue-300"
                  >
                    {tag}
                  </span>
                {/each}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div
      class="p-3 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-500 dark:text-gray-400 flex items-center justify-between"
    >
      <div class="flex gap-4">
        <span>↑↓ Navigate</span>
        <span>↵ Open</span>
        <span>Esc Close</span>
      </div>
      <div>{results.length} result{results.length !== 1 ? "s" : ""}</div>
    </div>
  </div>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
