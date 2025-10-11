<script lang="ts">
  import { countries, loadCountry } from "$lib/stores/vault";
  import { goto } from "$app/navigation";

  async function handleCountryClick(slug: string) {
    try {
      await loadCountry(slug);
      // Navigate to country page
      goto(`/country/${slug}`);
    } catch (error) {
      console.error("Failed to load country:", error);
    }
  }
</script>

<div class="country-list">
  <h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
    Countries
  </h2>

  {#if $countries.length === 0}
    <p class="text-gray-500 dark:text-gray-400">No countries found in vault.</p>
  {:else}
    <div
      class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3"
    >
      {#each $countries as country}
        <button
          on:click={() => handleCountryClick(country)}
          class="country-card bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700
                 border border-gray-200 dark:border-gray-700 rounded-lg p-4
                 text-left transition-colors duration-200"
        >
          <div
            class="text-lg font-semibold text-gray-900 dark:text-gray-100 uppercase"
          >
            {country}
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .country-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }

  .country-card {
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease;
  }
</style>
