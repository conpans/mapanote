<script lang="ts">
  import { vaultOpened } from "$lib/stores/vault";
  import VaultPicker from "$lib/components/VaultPicker.svelte";
  import CountryList from "$lib/components/CountryList.svelte";
</script>

<main class="min-h-screen bg-gray-50 dark:bg-gray-900 p-8">
  <div class="max-w-6xl mx-auto">
    {#if !$vaultOpened}
      <!-- Show vault picker if no vault is open -->
      <div class="flex items-center justify-center min-h-[80vh]">
        <VaultPicker />
      </div>
    {:else}
      <!-- Show country list when vault is open -->
      <div class="mb-6">
        <h1 class="text-4xl font-bold text-mapanote-blue-600 mb-2">Mapanote</h1>
        <p class="text-gray-600 dark:text-gray-400">
          Local-first geopolitics notebook
        </p>
      </div>

      <CountryList />
    {/if}
  </div>
</main>
