<script lang="ts">
  import { updateNote, deleteNote, currentCountry } from "$lib/stores/vault";
  import type { Note } from "$lib/types";

  export let note: Note;
  export let onClose: () => void;

  let title = note.title;
  let content = note.content;
  let tagsInput = note.tags.join(", ");
  let isSubmitting = false;
  let error = "";
  let showDeleteConfirm = false; // ← ADD THIS

  async function handleSave() {
    if (!content.trim() || !title.trim() || !$currentCountry) return;

    isSubmitting = true;
    error = "";

    try {
      const tags = tagsInput
        .split(",")
        .map((t) => t.trim())
        .filter((t) => t.length > 0);

      await updateNote(
        $currentCountry.slug,
        note.id,
        title.trim(),
        content.trim(),
        tags
      );
      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to update note";
      console.error("Error updating note:", err);
    } finally {
      isSubmitting = false;
    }
  }

  // ← REPLACE handleDelete with these 3 functions:
  function handleDeleteClick() {
    showDeleteConfirm = true;
  }

  function cancelDelete() {
    showDeleteConfirm = false;
  }

  async function confirmDelete() {
    if (!$currentCountry) return;

    showDeleteConfirm = false;
    isSubmitting = true;
    error = "";

    try {
      await deleteNote($currentCountry.slug, note.id);
      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to delete note";
      console.error("Error deleting note:", err);
      isSubmitting = false;
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

<!-- Modal backdrop -->
<div
  class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
  on:click={handleBackdropClick}
  on:keydown={(e) => e.key === "Escape" && onClose()}
  role="button"
  tabindex="0"
>
  <!-- Modal content -->
  <div
    class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto"
  >
    <!-- Header -->
    <div
      class="border-b border-gray-200 dark:border-gray-700 p-4 flex items-center justify-between"
    >
      <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">
        Edit Note
      </h2>
      <button
        on:click={onClose}
        class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
      >
        <svg
          class="w-6 h-6"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </button>
    </div>

    <!-- Body -->
    <div class="p-6">
      {#if error}
        <div
          class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 mb-4"
        >
          <p class="text-red-800 dark:text-red-200 text-sm">{error}</p>
        </div>
      {/if}

      <!-- Date (read-only) -->
      <div class="mb-4">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
        >
          Date
        </label>
        <input
          type="text"
          value={note.date}
          disabled
          class="w-full px-3 py-2 bg-gray-100 dark:bg-gray-700
                 border border-gray-300 dark:border-gray-600 rounded-lg
                 text-gray-600 dark:text-gray-400 text-sm"
        />
      </div>

      <!-- Title -->
      <div class="mb-4">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
        >
          Title
        </label>
        <input
          type="text"
          bind:value={title}
          placeholder="Note title..."
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                 rounded-lg bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent"
        />
      </div>

      <!-- Note content -->
      <div class="mb-4">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
        >
          Content
        </label>
        <textarea
          bind:value={content}
          rows="8"
          placeholder="Write your note..."
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                 rounded-lg bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
                 resize-y"
        ></textarea>
      </div>

      <!-- Tags -->
      <div class="mb-6">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
        >
          Tags (comma-separated)
        </label>
        <input
          type="text"
          bind:value={tagsInput}
          placeholder="politics, sports, history"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                 rounded-lg bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100 text-sm
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent"
        />
      </div>

      <!-- Actions -->
      <div class="flex items-center justify-between">
        <!-- ← CHANGED: handleDelete to handleDeleteClick -->
        <button
          on:click={handleDeleteClick}
          disabled={isSubmitting}
          class="px-4 py-2 bg-red-600 hover:bg-red-700
                 disabled:bg-gray-400 disabled:cursor-not-allowed
                 text-white rounded-lg text-sm font-medium transition"
        >
          Delete Note
        </button>

        <div class="flex gap-2">
          <button
            on:click={onClose}
            disabled={isSubmitting}
            class="px-4 py-2 bg-gray-200 hover:bg-gray-300
                   dark:bg-gray-700 dark:hover:bg-gray-600
                   text-gray-800 dark:text-gray-200
                   rounded-lg font-medium transition"
          >
            Cancel
          </button>
          <button
            on:click={handleSave}
            disabled={!content.trim() || !title.trim() || isSubmitting}
            class="px-6 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
                   disabled:bg-gray-400 disabled:cursor-not-allowed
                   text-white rounded-lg font-medium transition"
          >
            {#if isSubmitting}
              Saving...
            {:else}
              Save Changes
            {/if}
          </button>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- ← ADD THIS: Delete Confirmation Dialog -->
{#if showDeleteConfirm}
  <div
    class="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-[60]"
  >
    <div
      class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl p-6 max-w-md mx-4"
    >
      <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
        Delete Note?
      </h3>
      <p class="text-gray-600 dark:text-gray-400 mb-6">
        Are you sure you want to delete this note? This action cannot be undone.
      </p>
      <div class="flex gap-3 justify-end">
        <button
          on:click={cancelDelete}
          class="px-4 py-2 bg-gray-200 hover:bg-gray-300
                 dark:bg-gray-700 dark:hover:bg-gray-600
                 text-gray-800 dark:text-gray-200
                 rounded-lg font-medium transition"
        >
          Cancel
        </button>
        <button
          on:click={confirmDelete}
          class="px-4 py-2 bg-red-600 hover:bg-red-700
                 text-white rounded-lg font-medium transition"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  textarea:focus,
  input:focus {
    outline: none;
  }
</style>
