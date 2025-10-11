<script lang="ts">
  import { addNote, currentCountry } from "$lib/stores/vault";
  import type { AddNoteRequest } from "$lib/types";

  export let onSuccess: () => void = () => {};

  let text = "";
  let tagsInput = "";
  let visibility: "private" | "internal" | "publishable" = "internal";
  let pinned = false;
  let isSubmitting = false;
  let error = "";

  async function handleSubmit() {
    if (!text.trim() || !$currentCountry) return;

    isSubmitting = true;
    error = "";

    try {
      const request: AddNoteRequest = {
        country_slug: $currentCountry.slug,
        text: text.trim(),
        tags: tagsInput
          .split(",")
          .map((t) => t.trim())
          .filter((t) => t.length > 0),
        also: [],
        visibility,
        pinned,
      };

      await addNote(request);

      // Clear form
      text = "";
      tagsInput = "";
      pinned = false;

      onSuccess();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to add note";
      console.error("Error adding note:", err);
    } finally {
      isSubmitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Ctrl+Enter or Cmd+Enter to submit
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      handleSubmit();
    }
  }
</script>

<div
  class="add-note-form bg-white dark:bg-gray-800 rounded-lg border-2 border-mapanote-blue-200 dark:border-mapanote-blue-800 p-4 mb-6"
>
  <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
    Add Note
  </h3>

  {#if error}
    <div
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 mb-3"
    >
      <p class="text-red-800 dark:text-red-200 text-sm">{error}</p>
    </div>
  {/if}

  <form on:submit|preventDefault={handleSubmit}>
    <!-- Note text -->
    <textarea
      bind:value={text}
      on:keydown={handleKeydown}
      placeholder="Write your note... (Ctrl+Enter to save)"
      rows="4"
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
             rounded-lg bg-white dark:bg-gray-700
             text-gray-900 dark:text-gray-100
             placeholder-gray-400 dark:placeholder-gray-500
             focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
             resize-y mb-3"
    ></textarea>

    <!-- Tags input -->
    <input
      type="text"
      bind:value={tagsInput}
      placeholder="Tags (comma-separated): politics, current, economy"
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
             rounded-lg bg-white dark:bg-gray-700
             text-gray-900 dark:text-gray-100 text-sm
             placeholder-gray-400 dark:placeholder-gray-500
             focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
             mb-3"
    />

    <!-- Options row -->
    <div class="flex items-center gap-4 mb-3">
      <!-- Visibility -->
      <div class="flex items-center gap-2">
        <label class="text-sm text-gray-600 dark:text-gray-400"
          >Visibility:</label
        >
        <select
          bind:value={visibility}
          class="px-2 py-1 border border-gray-300 dark:border-gray-600
                 rounded bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100 text-sm"
        >
          <option value="internal">Internal</option>
          <option value="private">Private</option>
          <option value="publishable">Publishable</option>
        </select>
      </div>

      <!-- Pin checkbox -->
      <label class="flex items-center gap-2 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={pinned}
          class="rounded border-gray-300 text-mapanote-blue-600
                 focus:ring-mapanote-blue-500"
        />
        <span class="text-sm text-gray-700 dark:text-gray-300">📌 Pin note</span
        >
      </label>
    </div>

    <!-- Submit button -->
    <div class="flex justify-end">
      <button
        type="submit"
        disabled={!text.trim() || isSubmitting}
        class="px-6 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
               disabled:bg-gray-400 disabled:cursor-not-allowed
               text-white rounded-lg font-medium transition"
      >
        {#if isSubmitting}
          Adding...
        {:else}
          Add Note
        {/if}
      </button>
    </div>
  </form>

  <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
    💡 Tip: Press <kbd
      class="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs"
      >Ctrl+Enter</kbd
    > to save quickly
  </p>
</div>

<style>
  textarea:focus,
  input:focus,
  select:focus {
    outline: none;
  }

  kbd {
    font-family: monospace;
  }
</style>
