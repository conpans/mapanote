<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { openVault, isLoading } from "$lib/stores/vault";

  let selectedPath = "";
  let manualPath = "";
  let error = "";

  async function handleOpenVault() {
    try {
      error = "";

      console.log("Opening folder picker...");

      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Vault Folder",
      });

      console.log("Selected:", selected);

      if (selected) {
        const path = typeof selected === "string" ? selected : selected.path;
        console.log("Using path:", path);

        selectedPath = path;
        await openVault(path);
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      console.error("Error opening vault:", err);
    }
  }

  async function handleManualOpen() {
    try {
      error = "";
      if (!manualPath) {
        error = "Please enter a path";
        return;
      }
      await openVault(manualPath);
      selectedPath = manualPath;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      console.error("Error opening vault:", err);
    }
  }
</script>

<div
  class="vault-picker bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 max-w-md mx-auto"
>
  <h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
    Open a Vault
  </h2>

  <p class="text-gray-600 dark:text-gray-400 mb-6">
    Select your vault folder to get started with Mapanote.
  </p>

  {#if error}
    <div
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-4"
    >
      <p class="text-red-800 dark:text-red-200 text-sm font-medium">
        Error: {error}
      </p>
    </div>
  {/if}

  <button
    on:click={handleOpenVault}
    disabled={$isLoading}
    class="w-full bg-mapanote-blue-600 hover:bg-mapanote-blue-700 disabled:bg-gray-400
           text-white px-6 py-3 rounded-lg font-medium transition mb-4
           disabled:cursor-not-allowed"
  >
    {#if $isLoading}
      Opening...
    {:else}
      Open Vault Folder
    {/if}
  </button>

  <!-- Manual path input -->
  <div class="border-t border-gray-200 dark:border-gray-700 pt-4 mt-4">
    <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">
      Or enter path manually:
    </p>
    <div class="flex gap-2">
      <input
        type="text"
        bind:value={manualPath}
        placeholder="C:\Dev\mapanote\mapanote\examples\vault-sample"
        class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600
               rounded-lg bg-white dark:bg-gray-700
               text-gray-900 dark:text-gray-100 text-sm"
      />
      <button
        on:click={handleManualOpen}
        disabled={$isLoading}
        class="px-4 py-2 bg-gray-600 hover:bg-gray-700 disabled:bg-gray-400
               text-white rounded-lg text-sm transition
               disabled:cursor-not-allowed"
      >
        Open
      </button>
    </div>
  </div>

  {#if selectedPath}
    <p class="mt-4 text-sm text-green-600 dark:text-green-400">
      ✓ Selected: {selectedPath}
    </p>
  {/if}

  <div class="mt-6 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg">
    <p class="text-sm text-gray-600 dark:text-gray-400 mb-2 font-medium">
      Don't have a vault yet?
    </p>
    <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">
      Use the sample vault at:
    </p>
    <code
      class="block bg-gray-100 dark:bg-gray-700 p-2 rounded text-xs break-all"
    >
      C:\Dev\mapanote\mapanote\examples\vault-sample
    </code>
  </div>
</div>

<style>
  .vault-picker {
    min-height: 400px;
  }
</style>
