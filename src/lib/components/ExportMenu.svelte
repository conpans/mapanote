<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";
  import { writeTextFile } from "@tauri-apps/plugin-fs";

  export let countrySlug: string;
  export let countryTitle: string;
  export let onClose: () => void;

  let isExporting = false;
  let error = "";

  async function handleExportMarkdown() {
    isExporting = true;
    error = "";

    try {
      // Get markdown content from backend
      const markdown = await invoke<string>("export_country_markdown", {
        countrySlug,
      });

      // Open save dialog
      const filePath = await save({
        defaultPath: `${countrySlug}-export.md`,
        filters: [
          {
            name: "Markdown",
            extensions: ["md"],
          },
        ],
      });

      if (filePath) {
        // Write file
        await writeTextFile(filePath, markdown);
        alert(`Exported to ${filePath}`);
        onClose();
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      console.error("Export failed:", err);
    } finally {
      isExporting = false;
    }
  }
</script>

<div
  class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
  on:click={onClose}
  on:keydown={(e) => e.key === "Escape" && onClose()}
  role="button"
  tabindex="0"
>
  <div
    class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 max-w-md w-full"
    on:click|stopPropagation
    role="dialog"
    aria-modal="true"
  >
    <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
      Export {countryTitle}
    </h2>

    {#if error}
      <div
        class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 mb-4"
      >
        <p class="text-red-800 dark:text-red-200 text-sm">{error}</p>
      </div>
    {/if}

    <div class="space-y-3 mb-6">
      <button
        on:click={handleExportMarkdown}
        disabled={isExporting}
        class="w-full flex items-center gap-3 p-4 bg-gray-50 dark:bg-gray-700
               hover:bg-gray-100 dark:hover:bg-gray-600 rounded-lg transition
               disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <svg
          class="w-6 h-6 text-gray-600 dark:text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
        <div class="text-left flex-1">
          <div class="font-medium text-gray-900 dark:text-gray-100">
            {isExporting ? "Exporting..." : "Export as Markdown"}
          </div>
          <div class="text-sm text-gray-600 dark:text-gray-400">
            Single file with all notes
          </div>
        </div>
      </button>
    </div>

    <div class="flex justify-end gap-2">
      <button
        on:click={onClose}
        class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
      >
        Cancel
      </button>
    </div>
  </div>
</div>
