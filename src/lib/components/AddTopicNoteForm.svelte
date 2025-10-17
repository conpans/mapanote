<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { Note, CountryMetadata } from "$lib/types";

  interface Props {
    topicId: string;
    topicCountries: string[]; // Available countries in this topic
    onSuccess?: () => void;
  }

  let { topicId, topicCountries, onSuccess }: Props = $props();

  let title = $state("");
  let content = $state("");
  let tagsInput = $state("");
  let selectedCountries = $state<string[]>([]);
  let isSubmitting = $state(false);
  let error = $state("");
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

  async function handleSubmit(e: Event) {
    e.preventDefault();

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

      await invoke<Note>("add_topic_note", {
        topicId,
        title: title.trim(),
        content: content.trim(),
        tags,
        countryTargets: selectedCountries,
      });

      // Reset form
      title = "";
      content = "";
      tagsInput = "";
      selectedCountries = [];

      if (onSuccess) onSuccess();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to add note";
    } finally {
      isSubmitting = false;
    }
  }
</script>

<div
  class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 mb-6"
>
  <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
    Add Note to Topic
  </h3>

  {#if error}
    <div
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 mb-4"
    >
      <p class="text-red-800 dark:text-red-200 text-sm">{error}</p>
    </div>
  {/if}

  <form onsubmit={handleSubmit}>
    <!-- Title -->
    <div class="mb-4">
      <label
        class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
      >
        Title <span class="text-red-500">*</span>
      </label>
      <input
        type="text"
        bind:value={title}
        placeholder="e.g., Simón Bolívar's Revolutionary Vision"
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
        Content <span class="text-red-500">*</span>
      </label>
      <textarea
        bind:value={content}
        rows="4"
        placeholder="Write your note... (Ctrl+Enter to save)"
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
        Countries <span class="text-red-500">*</span>
        <span class="text-xs text-gray-500"
          >({selectedCountries.length} selected)</span
        >
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
      <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
        Select which countries this note is about
      </p>
    </div>

    <!-- Tags -->
    <div class="mb-4">
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

    <!-- Submit -->
    <button
      type="submit"
      disabled={!title.trim() ||
        !content.trim() ||
        selectedCountries.length === 0 ||
        isSubmitting}
      class="w-full px-4 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
             disabled:bg-gray-400 disabled:cursor-not-allowed
             text-white rounded-lg font-medium transition"
    >
      {#if isSubmitting}
        Adding...
      {:else}
        Add Note
      {/if}
    </button>
  </form>
</div>

<style>
  textarea:focus,
  input:focus {
    outline: none;
  }
</style>
