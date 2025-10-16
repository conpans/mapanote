<script lang="ts">
  import { createTopic } from "$lib/stores/topics";
  import { invoke } from "@tauri-apps/api/core";
  import type { CountryMetadata } from "$lib/stores/vault";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let title = $state("");
  let summary = $state("");
  let color = $state("#3B82F6");
  let selectedCountries = $state<string[]>([]);
  let isSubmitting = $state(false);
  let error = $state("");
  let allCountries = $state<CountryMetadata[]>([]);
  let searchQuery = $state("");

  // Load countries
  $effect(() => {
    loadCountries();
  });

  async function loadCountries() {
    try {
      allCountries = await invoke<CountryMetadata[]>(
        "get_all_countries_metadata"
      );
    } catch (err) {
      console.error("Failed to load countries:", err);
    }
  }

  // Filter countries by search
  $effect(() => {
    const filtered = allCountries.filter((c) =>
      c.name.toLowerCase().includes(searchQuery.toLowerCase())
    );
    return filtered;
  });

  function toggleCountry(slug: string) {
    if (selectedCountries.includes(slug)) {
      selectedCountries = selectedCountries.filter((s) => s !== slug);
    } else {
      selectedCountries = [...selectedCountries, slug];
    }
  }

  async function handleSubmit() {
    if (!title.trim()) return;

    isSubmitting = true;
    error = "";

    try {
      await createTopic(
        title.trim(),
        summary.trim() || undefined,
        color,
        selectedCountries
      );
      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to create topic";
    } finally {
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
  on:click={handleBackdropClick}
  on:keydown={(e) => e.key === "Escape" && onClose()}
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
        Create New Topic
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
          placeholder="e.g., EU Expansion, Belt and Road Initiative"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                 rounded-lg bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent"
          autofocus
        />
      </div>

      <!-- Summary -->
      <div class="mb-4">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
        >
          Summary (optional)
        </label>
        <textarea
          bind:value={summary}
          rows="3"
          placeholder="Brief description of this topic..."
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                 rounded-lg bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
                 resize-none"
        ></textarea>
      </div>

      <!-- Color -->
      <div class="mb-4">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
        >
          Color
        </label>
        <div class="flex items-center gap-3">
          <input
            type="color"
            bind:value={color}
            class="w-12 h-10 rounded cursor-pointer"
          />
          <input
            type="text"
            bind:value={color}
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600
                   rounded-lg bg-white dark:bg-gray-700
                   text-gray-900 dark:text-gray-100 text-sm font-mono"
          />
        </div>
      </div>

      <!-- Countries -->
      <div class="mb-6">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
        >
          Related Countries ({selectedCountries.length} selected)
        </label>

        <!-- Search -->
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search countries..."
          class="w-full px-3 py-2 mb-3 border border-gray-300 dark:border-gray-600
                 rounded-lg bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100 text-sm
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent"
        />

        <!-- Country list -->
        <div
          class="border border-gray-300 dark:border-gray-600 rounded-lg max-h-60 overflow-y-auto"
        >
          {#each allCountries.filter((c) => c.name
              .toLowerCase()
              .includes(searchQuery.toLowerCase())) as country}
            <label
              class="flex items-center gap-2 px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer border-b border-gray-200 dark:border-gray-700 last:border-b-0"
            >
              <input
                type="checkbox"
                checked={selectedCountries.includes(country.slug)}
                on:change={() => toggleCountry(country.slug)}
                class="rounded border-gray-300 text-mapanote-blue-600 focus:ring-mapanote-blue-500"
              />
              <span class="text-sm text-gray-700 dark:text-gray-300">
                {country.name} ({country.iso2})
              </span>
            </label>
          {/each}
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-2 justify-end">
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
          on:click={handleSubmit}
          disabled={!title.trim() || isSubmitting}
          class="px-6 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
                 disabled:bg-gray-400 disabled:cursor-not-allowed
                 text-white rounded-lg font-medium transition"
        >
          {#if isSubmitting}
            Creating...
          {:else}
            Create Topic
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  input:focus,
  textarea:focus {
    outline: none;
  }
</style>
