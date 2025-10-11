<script lang="ts">
  import { vaultOpened } from "$lib/stores/vault";
  import VaultPicker from "$lib/components/VaultPicker.svelte";
  import CountryList from "$lib/components/CountryList.svelte";
  import WorldMap from "$lib/components/WorldMap.svelte";

  let viewMode: "map" | "list" = "map";
</script>

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

      <!-- Content (map or list) -->
      {#if viewMode === "map"}
        <WorldMap />
      {:else}
        <CountryList />
      {/if}
    {/if}
  </div>
</main>
