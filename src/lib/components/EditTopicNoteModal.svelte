<script lang="ts">
  import { updateTopicNote, deleteTopicNote } from "$lib/stores/topics";
  import { invoke } from "@tauri-apps/api/core";
  import type { Note, CountryMetadata } from "$lib/types";

  interface Props {
    note: Note;
    topicId: string;
    topicCountries: string[];
    onClose: () => void;
  }

  let { note, topicId, topicCountries, onClose }: Props = $props();

  let title = $state(note.title);
  let content = $state(note.content);
  let tagsInput = $state(note.tags.join(", "));
  let selectedCountries = $state<string[]>(note.country_targets || []);
  let isSubmitting = $state(false);
  let error = $state("");
  let showDeleteConfirm = $state(false);
  let countriesMetadata = $state<Map<string, CountryMetadata>>(new Map());

  // Load country metadata
  $effect(() => {
    loadMetadata();
  });

  async function loadMetadata() {
    try {
      const metadata = await invoke<CountryMetadata[]>(
        "get_all_countries_metadata"
      );
      countriesMetadata = new Map(metadata.map((c) => [c.slug, c]));
    } catch (err) {
      console.error("Failed to load metadata:", err);
    }
  }

  function toggleCountry(slug: string) {
    if (selectedCountries.includes(slug)) {
      selectedCountries = selectedCountries.filter((s) => s !== slug);
    } else {
      selectedCountries = [...selectedCountries, slug];
    }
  }

  async function handleSave() {
    if (!title.trim() || !content.trim() || selectedCountries.length === 0) {
      error = "Please fill in all fields and select at least one country";
      return;
    }

    isSubmitting = true;
    error = "";

    try {
      const tags = tagsInput
        .split(",")
        .map((t) => t.trim())
        .filter((t) => t.length > 0);

      await updateTopicNote(
        topicId,
        note.id,
        title.trim(),
        content.trim(),
        tags,
        selectedCountries
      );

      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to update note";
    } finally {
      isSubmitting = false;
    }
  }

  async function handleDelete() {
    showDeleteConfirm = false;
    isSubmitting = true;

    try {
      await deleteTopicNote(topicId, note.id);
      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to delete note";
      isSubmitting = false;
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

<div
  class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
  onclick={handleBackdropClick}
  onkeydown={(e) => e.key === "Escape" && onClose()}
  role="button"
  tabindex="0"
>
  <div
    class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto"
  >
    <!-- Header -->
    <div
      class="border-b border-gray-200 dark:border-gray-700 p-4 flex items-center justify-between"
    >
      <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">
        Edit Topic Note
      </h2>
      <button
        onclick={onClose}
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

      <!-- Content -->
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

      <!-- Country Selection -->
      <div class="mb-4">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
        >
          Countries ({selectedCountries.length} selected)
        </label>
        <div class="flex flex-wrap gap-2">
          {#each topicCountries as countrySlug}
            {@const metadata = countriesMetadata.get(countrySlug)}
            <button
              type="button"
              onclick={() => toggleCountry(countrySlug)}
              class="px-3 py-1.5 rounded-lg text-sm transition {selectedCountries.includes(
                countrySlug
              )
                ? 'bg-mapanote-blue-600 text-white'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
            >
              {metadata?.name || countrySlug}
            </button>
          {/each}
        </div>
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
          placeholder="history, politics, revolution"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                 rounded-lg bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100 text-sm
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent"
        />
      </div>

      <!-- Actions -->
      <div class="flex items-center justify-between">
        <button
          onclick={() => (showDeleteConfirm = true)}
          disabled={isSubmitting}
          class="px-4 py-2 bg-red-600 hover:bg-red-700
                 disabled:bg-gray-400 disabled:cursor-not-allowed
                 text-white rounded-lg text-sm font-medium transition"
        >
          Delete Note
        </button>

        <div class="flex gap-2">
          <button
            onclick={onClose}
            disabled={isSubmitting}
            class="px-4 py-2 bg-gray-200 hover:bg-gray-300
                   dark:bg-gray-700 dark:hover:bg-gray-600
                   text-gray-800 dark:text-gray-200
                   rounded-lg font-medium transition"
          >
            Cancel
          </button>
          <button
            onclick={handleSave}
            disabled={!title.trim() ||
              !content.trim() ||
              selectedCountries.length === 0 ||
              isSubmitting}
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

<!-- Delete Confirmation -->
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
          onclick={() => (showDeleteConfirm = false)}
          class="px-4 py-2 bg-gray-200 hover:bg-gray-300
                 dark:bg-gray-700 dark:hover:bg-gray-600
                 text-gray-800 dark:text-gray-200 rounded-lg"
        >
          Cancel
        </button>
        <button
          onclick={handleDelete}
          class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg"
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
