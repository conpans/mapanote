<script lang="ts">
  import { vaultOpened } from "$lib/stores/vault";
  import VaultPicker from "$lib/components/VaultPicker.svelte";
  import CountryList from "$lib/components/CountryList.svelte";
  import WorldMap from "$lib/components/WorldMap.svelte";
  import SearchModal from "$lib/components/SearchModal.svelte";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let viewMode: "map" | "list" = "map";
  let showSearch = false;

  // ← ADD: Stats tracking
  interface VaultStats {
    country_count: number;
    note_count: number;
    pinned_count: number;
    tag_count: number;
  }

  let stats: VaultStats | null = null;

  // Load stats when vault opens
  $: if ($vaultOpened && !stats) {
    loadStats();
  }

  async function loadStats() {
    try {
      stats = await invoke<VaultStats>("get_vault_stats");
    } catch (error) {
      console.error("Failed to load stats:", error);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "k") {
      e.preventDefault();
      showSearch = true;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<main class="min-h-screen bg-gray-50 dark:bg-gray-900 p-8">
  <div class="max-w-7xl mx-auto">
    {#if !$vaultOpened}
      <!-- Show vault picker if no vault is open -->
      <div class="flex items-center justify-center min-h-[80vh]">
        <VaultPicker />
      </div>
    {:else}
      <!-- Header with view toggle -->
      <div class="mb-6">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-4xl font-bold text-mapanote-blue-600 mb-2">
              Mapanote
            </h1>
            <p class="text-gray-600 dark:text-gray-400">
              Local-first geopolitics notebook
            </p>
          </div>

          <div class="flex items-center gap-3">
            <!-- Timeline button -->
            <!-- ← ADD -->
            <button
              on:click={() => goto("/timeline")}
              class="px-4 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
                     rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition
                     flex items-center gap-2 text-gray-700 dark:text-gray-300"
            >
              <svg
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <span>Timeline</span>
            </button>

            <!-- Search button -->
            <!-- ← ADD -->
            <button
              on:click={() => (showSearch = true)}
              class="px-4 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
                     rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition
                     flex items-center gap-2 text-gray-700 dark:text-gray-300"
            >
              <svg
                class="w-5 h-5"
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
              <span>Search</span>
              <kbd
                class="text-xs bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded"
                >Ctrl+K</kbd
              >
            </button>

            <!-- View mode toggle -->
            <div
              class="flex bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-1"
            >
              <button
                on:click={() => (viewMode = "map")}
                class="px-4 py-2 rounded transition {viewMode === 'map'
                  ? 'bg-mapanote-blue-600 text-white'
                  : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'}"
              >
                🗺️ Map
              </button>
              <button
                on:click={() => (viewMode = "list")}
                class="px-4 py-2 rounded transition {viewMode === 'list'
                  ? 'bg-mapanote-blue-600 text-white'
                  : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'}"
              >
                📋 List
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Stats card -->
      <!-- ← ADD -->
      {#if stats}
        <div class="grid grid-cols-4 gap-4 mb-6">
          <div
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
          >
            <div class="text-3xl font-bold text-mapanote-blue-600">
              {stats.country_count}
            </div>
            <div class="text-sm text-gray-600 dark:text-gray-400 mt-1">
              Countries
            </div>
          </div>

          <div
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
          >
            <div class="text-3xl font-bold text-mapanote-blue-600">
              {stats.note_count}
            </div>
            <div class="text-sm text-gray-600 dark:text-gray-400 mt-1">
              Total Notes
            </div>
          </div>

          <div
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
          >
            <div class="text-3xl font-bold text-amber-600 dark:text-amber-400">
              {stats.pinned_count}
            </div>
            <div class="text-sm text-gray-600 dark:text-gray-400 mt-1">
              📌 Pinned
            </div>
          </div>

          <div
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
          >
            <div class="text-3xl font-bold text-gray-900 dark:text-gray-100">
              {stats.tag_count}
            </div>
            <div class="text-sm text-gray-600 dark:text-gray-400 mt-1">
              Unique Tags
            </div>
          </div>
        </div>
      {/if}

      <!-- Content (map or list) -->
      {#if viewMode === "map"}
        <WorldMap />
      {:else}
        <CountryList />
      {/if}
    {/if}
  </div>
</main>

<!-- Search modal -->
<!-- ← ADD -->
{#if showSearch}
  <SearchModal onClose={() => (showSearch = false)} />
{/if}
