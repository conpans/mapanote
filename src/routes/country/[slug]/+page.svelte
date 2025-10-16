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
  import AddNoteForm from "$lib/components/AddNoteForm.svelte";
  import EditNoteModal from "$lib/components/EditNoteModal.svelte";
  import ExportMenu from "$lib/components/ExportMenu.svelte";
  import type { Note } from "$lib/types";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import LoadingSkeleton from "$lib/components/LoadingSkeleton.svelte";
  import { getTopicsForCountry } from "$lib/stores/topics";
  import type { Topic } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  // Get slug from URL
  let slug = $derived($page.params.slug);

  // Filter state
  let selectedTags: string[] = [];
  let dateFilter: "all" | "7d" | "30d" = "all";

  // Edit modal state
  let editingNote: Note | null = null;
  let showExportMenu = false;

  let countryTopics = $state<Topic[]>([]);
  let relatedCountries = $state<
    Map<string, { countries: string[]; topic: Topic }>
  >(new Map());
  let countriesMetadata = $state<Map<string, any>>(new Map());

  async function loadCountryTopics() {
    if (!$currentCountry) return;

    try {
      countryTopics = await getTopicsForCountry($currentCountry.slug);

      // Build related countries map
      const relatedMap = new Map();

      for (const topic of countryTopics) {
        // Get full topic details to see all countries
        const fullTopic = await invoke<any>("get_topic", { topicId: topic.id });
        const otherCountries = fullTopic.countries.filter(
          (c: string) => c !== $currentCountry.slug
        );

        if (otherCountries.length > 0) {
          relatedMap.set(topic.id, {
            topic,
            countries: otherCountries,
          });
        }
      }

      relatedCountries = relatedMap;
    } catch (error) {
      console.error("Failed to load country topics:", error);
    }
  }

  async function loadCountriesMetadata() {
    try {
      const metadata = await invoke<any[]>("get_all_countries_metadata");
      countriesMetadata = new Map(metadata.map((c) => [c.slug, c]));
    } catch (error) {
      console.error("Failed to load countries metadata:", error);
    }
  }

  // Watch for slug changes and reload data
  $effect(() => {
    if (!slug) {
      console.error("No slug provided");
      return;
    }

    console.log("Slug changed to:", slug);

    // Load all data for the new country
    (async () => {
      try {
        console.log("Loading country with slug:", slug);
        await loadCountry(slug);
        console.log("Country loaded:", $currentCountry);
        await loadCountryTopics();
        await loadCountriesMetadata();
      } catch (error) {
        console.error("Failed to load country:", error);
      }
    })();
  });

  // Computed: all unique tags from notes
  let allTags = $derived.by(() => {
    const tagSet = new Set<string>();
    $currentNotes.forEach((note) => {
      note.tags.forEach((tag) => tagSet.add(tag));
    });
    return Array.from(tagSet).sort();
  });

  // Computed: filtered and sorted notes
  let filteredNotes = $derived.by(() => {
    let notes = [...$currentNotes];

    // Filter by tags
    if (selectedTags.length > 0) {
      notes = notes.filter((n) =>
        selectedTags.some((tag) => n.tags.includes(tag))
      );
    }

    // Filter by date
    if (dateFilter !== "all") {
      const now = new Date();
      const daysAgo = dateFilter === "7d" ? 7 : 30;
      const cutoff = new Date(now.getTime() - daysAgo * 24 * 60 * 60 * 1000);

      notes = notes.filter((n) => {
        const noteDate = new Date(n.date);
        return noteDate >= cutoff;
      });
    }

    // Sort by date (newest first)
    notes.sort((a, b) => b.date.localeCompare(a.date));

    return notes;
  });

  function goBack() {
    goto("/");
  }

  function toggleTag(tag: string) {
    if (selectedTags.includes(tag)) {
      selectedTags = selectedTags.filter((t) => t !== tag);
    } else {
      selectedTags = [...selectedTags, tag];
    }
  }

  function handleEditNote(note: Note) {
    editingNote = note;
  }

  function closeEditModal() {
    editingNote = null;
  }

  function handleAddSuccess() {
    // Scroll to top after adding
    window.scrollTo({ top: 0, behavior: "smooth" });
  }
</script>

<main class="min-h-screen bg-gray-50 dark:bg-gray-900">
  {#if $isLoading}
    <div class="max-w-6xl mx-auto p-8">
      <LoadingSkeleton type="country" count={3} />
    </div>
  {:else if $currentCountry}
    <!-- Country Header -->
    <div
      class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700"
    >
      <div class="max-w-6xl mx-auto p-8">
        <button
          onclick={goBack}
          class="text-mapanote-blue-600 hover:underline mb-4 inline-flex items-center gap-1"
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
          Back to countries
        </button>

        <ThemeToggle />

        <div class="flex items-start justify-between">
          <div>
            <div class="flex items-center gap-4 mb-2">
              <h1 class="text-4xl font-bold text-gray-900 dark:text-gray-100">
                {$currentCountry.name}
              </h1>
              <span
                class="text-2xl text-gray-500 dark:text-gray-400 uppercase font-mono"
              >
                {$currentCountry.iso2}
              </span>
            </div>

            <p class="text-lg text-gray-600 dark:text-gray-400 mb-2">
              {$currentCountry.region}
              {#if $currentCountry.subregion && $currentCountry.subregion !== "Unknown"}
                · {$currentCountry.subregion}
              {/if}
            </p>

            {#if $currentCountry.summary}
              <p class="text-gray-700 dark:text-gray-300 max-w-3xl">
                {$currentCountry.summary}
              </p>
            {/if}

            <!-- Export button - temporarily disabled -->
            <!--
<button
  onclick={() => (showExportMenu = true)}
  class="mt-4 px-4 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600
         hover:bg-gray-50 dark:hover:bg-gray-600 rounded-lg transition
         flex items-center gap-2 text-gray-700 dark:text-gray-300"
>
  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
    />
  </svg>
  Export
</button>
-->
          </div>

          <!-- Stats -->
          <div class="flex flex-col items-end gap-2 text-sm">
            <div class="text-gray-600 dark:text-gray-400">
              <span class="font-semibold text-gray-900 dark:text-gray-100">
                {$currentNotes.length}
              </span> notes
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="max-w-6xl mx-auto p-8">
      <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <!-- Main Content (Left Side) -->
        <div class="lg:col-span-3 space-y-6">
          <!-- Seen in Topics Section -->
          {#if countryTopics.length > 0}
            <div>
              <h2
                class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-3"
              >
                Seen in Topics
              </h2>
              <div class="flex flex-wrap gap-2">
                {#each countryTopics as topic}
                  <a
                    href="/topic/{topic.id}"
                    class="inline-flex items-center gap-2 px-3 py-2 rounded-lg
                 border-2 hover:border-mapanote-blue-500 transition
                 bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700"
                    style="border-left: 4px solid {topic.color || '#3B82F6'}"
                  >
                    <span
                      class="text-sm font-medium text-gray-900 dark:text-gray-100"
                    >
                      {topic.title}
                    </span>
                    {#if topic.pinned}
                      <span class="text-xs">📌</span>
                    {/if}
                  </a>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Add Note Form -->
          <AddNoteForm onSuccess={handleAddSuccess} />

          <!-- Filters Bar -->
          <div
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
          >
            <div class="flex items-center justify-between mb-3">
              <h3
                class="text-sm font-semibold text-gray-700 dark:text-gray-300"
              >
                Filters
              </h3>
              {#if selectedTags.length > 0 || dateFilter !== "all"}
                <button
                  onclick={() => {
                    selectedTags = [];
                    dateFilter = "all";
                  }}
                  class="text-xs text-mapanote-blue-600 hover:underline"
                >
                  Clear all
                </button>
              {/if}
            </div>

            <!-- Date filter -->
            <div class="flex items-center gap-2 mb-3">
              <span class="text-sm text-gray-600 dark:text-gray-400">Show:</span
              >
              <button
                onclick={() => (dateFilter = "all")}
                class="px-3 py-1 rounded text-sm transition {dateFilter ===
                'all'
                  ? 'bg-mapanote-blue-600 text-white'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
              >
                All time
              </button>
              <button
                onclick={() => (dateFilter = "7d")}
                class="px-3 py-1 rounded text-sm transition {dateFilter === '7d'
                  ? 'bg-mapanote-blue-600 text-white'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
              >
                Last 7 days
              </button>
              <button
                onclick={() => (dateFilter = "30d")}
                class="px-3 py-1 rounded text-sm transition {dateFilter ===
                '30d'
                  ? 'bg-mapanote-blue-600 text-white'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
              >
                Last 30 days
              </button>
            </div>

            <!-- Tag filter -->
            {#if allTags.length > 0}
              <div>
                <span
                  class="text-sm text-gray-600 dark:text-gray-400 mb-2 block"
                  >Tags:</span
                >
                <div class="flex flex-wrap gap-2">
                  {#each allTags as tag}
                    <button
                      onclick={() => toggleTag(tag)}
                      class="px-3 py-1 rounded-full text-sm transition {selectedTags.includes(
                        tag
                      )
                        ? 'bg-mapanote-blue-600 text-white'
                        : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
                    >
                      {tag}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>

          <!-- Notes Section -->
          <div>
            <h2
              class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4"
            >
              Notes
              <span class="text-gray-500 dark:text-gray-400 font-normal">
                ({filteredNotes.length}{filteredNotes.length !==
                $currentNotes.length
                  ? ` of ${$currentNotes.length}`
                  : ""})
              </span>
            </h2>

            {#if filteredNotes.length === 0}
              <div
                class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-8 text-center"
              >
                <p class="text-gray-500 dark:text-gray-400">
                  {selectedTags.length > 0 || dateFilter !== "all"
                    ? "No notes match the current filters."
                    : `No notes yet for ${$currentCountry.name}. Add your first note above!`}
                </p>
                {#if selectedTags.length > 0 || dateFilter !== "all"}
                  <button
                    onclick={() => {
                      selectedTags = [];
                      dateFilter = "all";
                    }}
                    class="mt-3 text-mapanote-blue-600 hover:underline text-sm"
                  >
                    Clear filters
                  </button>
                {/if}
              </div>
            {:else}
              <div class="space-y-4">
                {#each filteredNotes as note (note.id)}
                  <div
                    class="note-card bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 hover:shadow-md transition-shadow"
                  >
                    <!-- Note header -->
                    <div class="flex items-start justify-between mb-3">
                      <div class="flex-1">
                        <h4
                          class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2"
                        >
                          {note.title}
                        </h4>
                        <div
                          class="flex items-center gap-3 flex-wrap text-sm text-gray-500 dark:text-gray-400"
                        >
                          <span
                            class="font-medium text-gray-700 dark:text-gray-300"
                          >
                            {note.date}
                          </span>

                          {#if note.tags.length > 0}
                            <span>·</span>
                            <div class="flex gap-2">
                              {#each note.tags as tag}
                                <button
                                  onclick={() => toggleTag(tag)}
                                  class="bg-mapanote-blue-100 dark:bg-mapanote-blue-900
                             text-mapanote-blue-700 dark:text-mapanote-blue-300
                             px-2 py-1 rounded text-xs hover:bg-mapanote-blue-200
                             dark:hover:bg-mapanote-blue-800 transition"
                                >
                                  {tag}
                                </button>
                              {/each}
                            </div>
                          {/if}
                        </div>
                      </div>

                      <!-- Edit button -->
                      <button
                        onclick={() => handleEditNote(note)}
                        class="text-gray-400 hover:text-mapanote-blue-600 dark:hover:text-mapanote-blue-400 transition ml-4"
                        title="Edit note"
                      >
                        <svg
                          class="w-5 h-5"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                          />
                        </svg>
                      </button>
                    </div>

                    <!-- Note content -->
                    <div class="prose dark:prose-invert max-w-none">
                      <p
                        class="text-gray-700 dark:text-gray-300 whitespace-pre-wrap leading-relaxed"
                      >
                        {note.content}
                      </p>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- Sidebar (Right Side) -->
        <div class="lg:col-span-1">
          {#if relatedCountries.size > 0}
            <div
              class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 sticky top-4"
            >
              <h3
                class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4"
              >
                Also seen in...
              </h3>

              <div class="space-y-4">
                {#each Array.from(relatedCountries.values()) as { topic, countries }}
                  <div>
                    <div class="flex items-center gap-2 mb-2">
                      <div
                        class="w-1 h-4 rounded"
                        style="background-color: {topic.color || '#3B82F6'}"
                      ></div>
                      <span
                        class="text-xs font-medium text-gray-600 dark:text-gray-400"
                      >
                        via {topic.title}
                      </span>
                    </div>

                    <div class="space-y-1 ml-3">
                      {#each countries as countrySlug}
                        {@const metadata = countriesMetadata.get(countrySlug)}
                        <button
                          onclick={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            console.log("Clicked country:", countrySlug);
                            console.log("Full name:", metadata?.name);
                            console.log(
                              "Navigating to:",
                              `/country/${countrySlug}`
                            );
                            goto(`/country/${countrySlug}`);
                          }}
                          type="button"
                          class="block w-full text-left text-sm text-mapanote-blue-600 dark:text-mapanote-blue-400
         hover:text-mapanote-blue-800 dark:hover:text-mapanote-blue-300
         hover:underline hover:translate-x-1 transition-all
         py-1 px-2 rounded hover:bg-mapanote-blue-50 dark:hover:bg-mapanote-blue-900/20
         cursor-pointer"
                        >
                          → {metadata?.name || countrySlug}
                        </button>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {:else}
    <div class="flex items-center justify-center h-screen">
      <div class="text-center">
        <p class="text-gray-600 dark:text-gray-400 mb-4">Country not found</p>
        <button onclick={goBack} class="text-mapanote-blue-600 hover:underline">
          ← Back to countries
        </button>
      </div>
    </div>
  {/if}
</main>

<!-- Edit Modal (conditionally rendered) -->
{#if editingNote}
  <EditNoteModal note={editingNote} onClose={closeEditModal} />
{/if}

<!-- Export Menu - temporarily disabled -->
<!--
{#if showExportMenu && $currentCountry}
  <ExportMenu
    countrySlug={$currentCountry.slug}
    countryTitle={$currentCountry.name}
    onClose={() => (showExportMenu = false)}
  />
{/if}
-->

<style>
  .note-card {
    transition:
      box-shadow 0.2s ease,
      transform 0.1s ease;
  }

  .note-card:hover {
    transform: translateY(-1px);
  }
</style>
