<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    currentTopic,
    loadTopic,
    updateTopic,
    deleteTopic,
    removeCountryFromTopic,
  } from "$lib/stores/topics";
  import { invoke } from "@tauri-apps/api/core";
  import type { Note } from "$lib/types";
  import type { CountryMetadata } from "$lib/stores/vault";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import LoadingSkeleton from "$lib/components/LoadingSkeleton.svelte";

  let topicId = $derived($page.params.id);

  let isEditing = $state(false);
  let editTitle = $state("");
  let editSummary = $state("");
  let editColor = $state("#3B82F6");
  let editPinned = $state(false);
  let isSaving = $state(false);
  let showDeleteConfirm = $state(false);
  let allNotes = $state<{ country: string; notes: Note[] }[]>([]);
  let countriesMetadata = $state<Map<string, CountryMetadata>>(new Map());

  onMount(async () => {
    if (!topicId) return;
    await loadTopic(topicId);
    if ($currentTopic) {
      await loadTopicNotes();
      await loadCountriesMetadata();
    }
  });

  async function loadTopicNotes() {
    if (!$currentTopic) return;

    const notesPromises = $currentTopic.countries.map(async (countrySlug) => {
      try {
        const notes = await invoke<Note[]>("get_country_notes", {
          slug: countrySlug,
        });
        return { country: countrySlug, notes };
      } catch (error) {
        console.error(`Failed to load notes for ${countrySlug}:`, error);
        return { country: countrySlug, notes: [] };
      }
    });

    allNotes = await Promise.all(notesPromises);
  }

  async function loadCountriesMetadata() {
    try {
      const metadata = await invoke<CountryMetadata[]>(
        "get_all_countries_metadata"
      );
      countriesMetadata = new Map(metadata.map((c) => [c.slug, c]));
    } catch (error) {
      console.error("Failed to load countries metadata:", error);
    }
  }

  function startEditing() {
    if (!$currentTopic) return;
    editTitle = $currentTopic.title;
    editSummary = $currentTopic.summary || "";
    editColor = $currentTopic.color || "#3B82F6";
    editPinned = $currentTopic.pinned;
    isEditing = true;
  }

  function cancelEditing() {
    isEditing = false;
  }

  async function saveChanges() {
    if (!$currentTopic || !editTitle.trim() || !topicId) return;

    isSaving = true;
    try {
      await updateTopic(
        topicId, // Now TypeScript knows it's not undefined
        editTitle.trim(),
        editSummary.trim() || undefined,
        editColor,
        editPinned
      );
      isEditing = false;
      await loadTopic(topicId);
    } catch (error) {
      console.error("Failed to update topic:", error);
    } finally {
      isSaving = false;
    }
  }

  async function handleRemoveCountry(countrySlug: string) {
    if (!$currentTopic || !topicId) return;
    if (!confirm(`Remove ${countrySlug} from this topic?`)) return;

    try {
      await removeCountryFromTopic(topicId, countrySlug);
      await loadTopic(topicId);
      await loadTopicNotes();
    } catch (error) {
      console.error("Failed to remove country:", error);
    }
  }

  async function handleDelete() {
    if (!$currentTopic) return;

    showDeleteConfirm = false;
    try {
      await deleteTopic($currentTopic.id);
      goto("/topics");
    } catch (error) {
      console.error("Failed to delete topic:", error);
    }
  }

  function goBack() {
    goto("/topics");
  }

  function goToCountry(slug: string) {
    goto(`/country/${slug}`);
  }
</script>

<main class="min-h-screen bg-gray-50 dark:bg-gray-900 p-4">
  <div class="max-w-7xl mx-auto">
    {#if !$currentTopic}
      <LoadingSkeleton type="country" count={3} />
    {:else}
      <!-- Header -->
      <div class="mb-6">
        <div class="flex items-center justify-between mb-4">
          <button
            on:click={goBack}
            class="text-mapanote-blue-600 hover:underline inline-flex items-center gap-1"
          >
            <svg
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M15 19l-7-7 7-7"
              />
            </svg>
            Back to topics
          </button>

          <ThemeToggle />
        </div>

        {#if isEditing}
          <!-- Edit Mode -->
          <div
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
          >
            <h2
              class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4"
            >
              Edit Topic
            </h2>

            <div class="space-y-4">
              <!-- Title -->
              <div>
                <label
                  class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                >
                  Title
                </label>
                <input
                  type="text"
                  bind:value={editTitle}
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                         rounded-lg bg-white dark:bg-gray-700
                         text-gray-900 dark:text-gray-100"
                />
              </div>

              <!-- Summary -->
              <div>
                <label
                  class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                >
                  Summary
                </label>
                <textarea
                  bind:value={editSummary}
                  rows="3"
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                         rounded-lg bg-white dark:bg-gray-700
                         text-gray-900 dark:text-gray-100 resize-none"
                ></textarea>
              </div>

              <!-- Color & Pinned -->
              <div class="flex items-center gap-4">
                <div class="flex-1">
                  <label
                    class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                  >
                    Color
                  </label>
                  <input
                    type="color"
                    bind:value={editColor}
                    class="w-20 h-10 rounded cursor-pointer"
                  />
                </div>

                <label class="flex items-center gap-2 cursor-pointer mt-6">
                  <input
                    type="checkbox"
                    bind:checked={editPinned}
                    class="rounded border-gray-300 text-mapanote-blue-600"
                  />
                  <span class="text-sm text-gray-700 dark:text-gray-300"
                    >📌 Pin topic</span
                  >
                </label>
              </div>

              <!-- Actions -->
              <div class="flex gap-2 justify-end pt-4">
                <button
                  on:click={cancelEditing}
                  disabled={isSaving}
                  class="px-4 py-2 bg-gray-200 hover:bg-gray-300
                         dark:bg-gray-700 dark:hover:bg-gray-600
                         text-gray-800 dark:text-gray-200 rounded-lg"
                >
                  Cancel
                </button>
                <button
                  on:click={saveChanges}
                  disabled={!editTitle.trim() || isSaving}
                  class="px-6 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
                         disabled:bg-gray-400 text-white rounded-lg"
                >
                  {isSaving ? "Saving..." : "Save Changes"}
                </button>
              </div>
            </div>
          </div>
        {:else}
          <!-- View Mode -->
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-4 mb-2">
                <div
                  class="w-1 h-12 rounded"
                  style="background-color: {$currentTopic.color || '#3B82F6'}"
                ></div>
                <div>
                  <h1
                    class="text-3xl font-bold text-gray-900 dark:text-gray-100"
                  >
                    {$currentTopic.title}
                    {#if $currentTopic.pinned}
                      <span class="text-amber-500">📌</span>
                    {/if}
                  </h1>
                  {#if $currentTopic.summary}
                    <p class="text-gray-600 dark:text-gray-400 mt-1">
                      {$currentTopic.summary}
                    </p>
                  {/if}
                </div>
              </div>

              <div
                class="flex gap-4 text-sm text-gray-500 dark:text-gray-400 mt-4"
              >
                <span>{$currentTopic.countries.length} countries</span>
                <span>•</span>
                <span>{$currentTopic.note_count} notes</span>
              </div>
            </div>

            <div class="flex gap-2">
              <button
                on:click={startEditing}
                class="px-4 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
                       hover:bg-gray-50 dark:hover:bg-gray-700 rounded-lg transition"
              >
                Edit
              </button>
              <button
                on:click={() => (showDeleteConfirm = true)}
                class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition"
              >
                Delete
              </button>
            </div>
          </div>
        {/if}
      </div>

      <!-- Countries Grid -->
      <div class="mb-6">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Countries ({$currentTopic.countries.length})
        </h2>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {#each $currentTopic.countries as countrySlug}
            {@const metadata = countriesMetadata.get(countrySlug)}
            {@const countryNotes =
              allNotes.find((n) => n.country === countrySlug)?.notes || []}
            <div
              class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
            >
              <div class="flex items-start justify-between mb-2">
                <button
                  on:click={() => goToCountry(countrySlug)}
                  class="text-left flex-1 hover:text-mapanote-blue-600"
                >
                  <h3 class="font-semibold text-gray-900 dark:text-gray-100">
                    {metadata?.name || countrySlug}
                  </h3>
                  <p class="text-sm text-gray-500 dark:text-gray-400">
                    {countryNotes.length}
                    {countryNotes.length === 1 ? "note" : "notes"}
                  </p>
                </button>

                <button
                  on:click={() => handleRemoveCountry(countrySlug)}
                  class="text-red-500 hover:text-red-700 text-sm"
                  title="Remove from topic"
                >
                  ✕
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- All Notes -->
      <div>
        <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
          All Notes ({allNotes.reduce((sum, c) => sum + c.notes.length, 0)})
        </h2>

        <div class="space-y-6">
          {#each allNotes as { country, notes }}
            {#if notes.length > 0}
              {@const metadata = countriesMetadata.get(country)}
              <div>
                <h3
                  class="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-3"
                >
                  {metadata?.name || country}
                </h3>
                <div class="space-y-3">
                  {#each notes as note}
                    <button
                      on:click={() => goToCountry(country)}
                      class="w-full text-left bg-white dark:bg-gray-800 rounded-lg
                             border border-gray-200 dark:border-gray-700 p-4
                             hover:border-mapanote-blue-500 transition"
                    >
                      <div class="flex items-start justify-between mb-2">
                        <h4
                          class="font-semibold text-gray-900 dark:text-gray-100"
                        >
                          {note.title}
                        </h4>
                        <span class="text-xs text-gray-500 dark:text-gray-400">
                          {note.date}
                        </span>
                      </div>
                      <p
                        class="text-sm text-gray-600 dark:text-gray-400 line-clamp-2"
                      >
                        {note.content}
                      </p>
                      {#if note.tags.length > 0}
                        <div class="flex gap-2 mt-2">
                          {#each note.tags as tag}
                            <span
                              class="text-xs px-2 py-1 rounded bg-mapanote-blue-100 dark:bg-mapanote-blue-900
                                     text-mapanote-blue-700 dark:text-mapanote-blue-300"
                            >
                              {tag}
                            </span>
                          {/each}
                        </div>
                      {/if}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  </div>
</main>

<!-- Delete Confirmation -->
{#if showDeleteConfirm}
  <div
    class="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-50"
  >
    <div
      class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl p-6 max-w-md mx-4"
    >
      <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
        Delete Topic?
      </h3>
      <p class="text-gray-600 dark:text-gray-400 mb-6">
        Are you sure you want to delete this topic? This action cannot be
        undone. Notes will not be deleted.
      </p>
      <div class="flex gap-3 justify-end">
        <button
          on:click={() => (showDeleteConfirm = false)}
          class="px-4 py-2 bg-gray-200 hover:bg-gray-300
                 dark:bg-gray-700 dark:hover:bg-gray-600
                 text-gray-800 dark:text-gray-200 rounded-lg"
        >
          Cancel
        </button>
        <button
          on:click={handleDelete}
          class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
