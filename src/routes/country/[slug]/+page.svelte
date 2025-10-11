<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import {
    loadCountry,
    currentCountry,
    currentNotes,
    isLoading,
  } from "$lib/stores/vault";
  import { goto } from "$app/navigation";

  // Get slug from URL
  $: slug = $page.params.slug;

  // Load country data when component mounts
  onMount(async () => {
    try {
      await loadCountry(slug);
    } catch (error) {
      console.error("Failed to load country:", error);
    }
  });

  function goBack() {
    goto("/");
  }
</script>

<main class="min-h-screen bg-gray-50 dark:bg-gray-900">
  {#if $isLoading}
    <div class="flex items-center justify-center h-screen">
      <div class="text-gray-600 dark:text-gray-400">
        Loading country data...
      </div>
    </div>
  {:else if $currentCountry}
    <!-- Country Header -->
    <div
      class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700"
    >
      <div class="max-w-6xl mx-auto p-8">
        <button
          on:click={goBack}
          class="text-mapanote-blue-600 hover:underline mb-4 inline-flex items-center"
        >
          ← Back to countries
        </button>

        <div class="flex items-start justify-between">
          <div>
            <div class="flex items-center gap-4 mb-2">
              <h1 class="text-4xl font-bold text-gray-900 dark:text-gray-100">
                {$currentCountry.title}
              </h1>
              <span class="text-2xl text-gray-500 dark:text-gray-400 uppercase">
                {$currentCountry.slug}
              </span>
            </div>

            <p class="text-lg text-gray-600 dark:text-gray-400 mb-2">
              {$currentCountry.region}
            </p>

            {#if $currentCountry.summary}
              <p class="text-gray-700 dark:text-gray-300 max-w-3xl">
                {$currentCountry.summary}
              </p>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Notes Section -->
    <div class="max-w-6xl mx-auto p-8">
      <h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-6">
        Notes ({$currentNotes.length})
      </h2>

      {#if $currentNotes.length === 0}
        <div
          class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-8 text-center"
        >
          <p class="text-gray-500 dark:text-gray-400">
            No notes yet for {$currentCountry.title}
          </p>
        </div>
      {:else}
        <div class="space-y-4">
          {#each $currentNotes as note}
            <div
              class="note-card bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
            >
              <!-- Note metadata -->
              <div
                class="flex items-center gap-3 text-sm text-gray-500 dark:text-gray-400 mb-3"
              >
                <span class="font-medium">{note.date}</span>

                {#if note.tags.length > 0}
                  <span>·</span>
                  <div class="flex gap-2">
                    {#each note.tags as tag}
                      <span
                        class="bg-mapanote-blue-100 dark:bg-mapanote-blue-900 text-mapanote-blue-700 dark:text-mapanote-blue-300 px-2 py-1 rounded text-xs"
                      >
                        {tag}
                      </span>
                    {/each}
                  </div>
                {/if}

                {#if note.also.length > 0}
                  <span>·</span>
                  <span class="text-xs">
                    also: {note.also.join(", ")}
                  </span>
                {/if}

                <span class="ml-auto text-xs">
                  {note.visibility}
                </span>
              </div>

              <!-- Note text -->
              <div class="prose dark:prose-invert max-w-none">
                <p class="text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
                  {note.text}
                </p>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {:else}
    <div class="flex items-center justify-center h-screen">
      <div class="text-center">
        <p class="text-gray-600 dark:text-gray-400 mb-4">Country not found</p>
        <button
          on:click={goBack}
          class="text-mapanote-blue-600 hover:underline"
        >
          ← Back to countries
        </button>
      </div>
    </div>
  {/if}
</main>

<style>
  .note-card {
    transition: box-shadow 0.2s ease;
  }

  .note-card:hover {
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }
</style>
