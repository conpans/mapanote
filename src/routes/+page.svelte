<script lang="ts">
  import { vaultOpened } from "$lib/stores/vault";
  import WelcomeScreen from "$lib/components/WelcomeScreen.svelte";
  import VaultPicker from "$lib/components/VaultPicker.svelte";
  import CountryList from "$lib/components/CountryList.svelte";
  import WorldMap from "$lib/components/WorldMap.svelte";
  import { goto } from "$app/navigation";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { openSearch } from "$lib/stores/searchStore";

  let viewMode: "map" | "list" = "map";

  function handleVaultOpened(event: CustomEvent<string>) {
    console.log("Vault opened:", event.detail);
  }
</script>

{#if !$vaultOpened}
  <WelcomeScreen on:vaultOpened={handleVaultOpened} />
{:else}
  <main class="min-h-screen bg-gray-50 dark:bg-gray-900 p-4">
    <div class="max-w-full mx-auto">
      <!-- Header with view toggle -->
      <div class="mb-4">
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
            <!-- Theme toggle -->
            <ThemeToggle />

            <!-- Timeline button -->
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

            <!-- Topics button -->
            <!-- ← ADD THIS -->
            <button
              on:click={() => goto("/topics")}
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
                  d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z"
                />
              </svg>
              <span>Topics</span>
            </button>

            <!-- Search button -->
            <button
              on:click={openSearch}
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
                class="text-xs px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-700
                       border border-gray-300 dark:border-gray-600"
              >
                Ctrl+K
              </kbd>
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

      <!-- Content (map or list) -->
      <div class="mt-4">
        {#if viewMode === "map"}
          <WorldMap />
        {:else}
          <CountryList />
        {/if}
      </div>
    </div>
  </main>
{/if}
