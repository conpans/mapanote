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

  // Get slug from URL
  $: slug = $page.params.slug;

  // Filter state
  let selectedTags: string[] = [];
  let showPinnedOnly = false;
  let dateFilter: "all" | "7d" | "30d" = "all";

  // Edit modal state
  let editingNote: Note | null = null;
  let showExportMenu = false;

  // Load country data when component mounts
  onMount(async () => {
    try {
      await loadCountry(slug);
    } catch (error) {
      console.error("Failed to load country:", error);
    }
  });

  // Computed: all unique tags from notes
  $: allTags = (() => {
    const tagSet = new Set<string>();
    $currentNotes.forEach((note) => {
      note.tags.forEach((tag) => tagSet.add(tag));
    });
    return Array.from(tagSet).sort();
  })();

  // Computed: filtered and sorted notes
  $: filteredNotes = (() => {
    let notes = [...$currentNotes];

    // Filter by pinned
    if (showPinnedOnly) {
      notes = notes.filter((n) => n.pinned);
    }

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

    // Sort: pinned first, then by date (newest first)
    notes.sort((a, b) => {
      if (a.pinned !== b.pinned) {
        return a.pinned ? -1 : 1;
      }
      return b.date.localeCompare(a.date);
    });

    return notes;
  })();

  // Computed: pinned notes count
  $: pinnedCount = $currentNotes.filter((n) => n.pinned).length;

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

  function clearFilters() {
    selectedTags = [];
    showPinnedOnly = false;
    dateFilter = "all";
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

        <div class="flex items-start justify-between">
          <div>
            <div class="flex items-center gap-4 mb-2">
              <h1 class="text-4xl font-bold text-gray-900 dark:text-gray-100">
                {$currentCountry.title}
              </h1>
              <span
                class="text-2xl text-gray-500 dark:text-gray-400 uppercase font-mono"
              >
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

            <!-- Export button -->
            <!-- ← ADD -->
            <button
              on:click={() => (showExportMenu = true)}
              class="px-4 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600
                   hover:bg-gray-50 dark:hover:bg-gray-600 rounded-lg transition
                   flex items-center gap-2 text-gray-700 dark:text-gray-300"
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
                  d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                />
              </svg>
              Export
            </button>

            {#if $currentCountry.aliases.length > 0}
              <div
                class="mt-3 flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400"
              >
                <span>Also known as:</span>
                <span class="font-medium"
                  >{$currentCountry.aliases.join(", ")}</span
                >
              </div>
            {/if}
          </div>

          <!-- Stats -->
          <div class="flex flex-col items-end gap-2 text-sm">
            <div class="text-gray-600 dark:text-gray-400">
              <span class="font-semibold text-gray-900 dark:text-gray-100"
                >{$currentNotes.length}</span
              > notes
            </div>
            {#if pinnedCount > 0}
              <div class="text-gray-600 dark:text-gray-400">
                📌 <span class="font-semibold text-gray-900 dark:text-gray-100"
                  >{pinnedCount}</span
                > pinned
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="max-w-6xl mx-auto p-8">
      <!-- Add Note Form -->
      <AddNoteForm onSuccess={handleAddSuccess} />

      <!-- Filters Bar -->
      <div
        class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 mb-6"
      >
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
            Filters
          </h3>
          {#if selectedTags.length > 0 || showPinnedOnly || dateFilter !== "all"}
            <button
              on:click={clearFilters}
              class="text-xs text-mapanote-blue-600 hover:underline"
            >
              Clear all
            </button>
          {/if}
        </div>

        <!-- Date filter -->
        <div class="flex items-center gap-2 mb-3">
          <span class="text-sm text-gray-600 dark:text-gray-400">Show:</span>
          <button
            on:click={() => (dateFilter = "all")}
            class="px-3 py-1 rounded text-sm transition {dateFilter === 'all'
              ? 'bg-mapanote-blue-600 text-white'
              : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
          >
            All time
          </button>
          <button
            on:click={() => (dateFilter = "7d")}
            class="px-3 py-1 rounded text-sm transition {dateFilter === '7d'
              ? 'bg-mapanote-blue-600 text-white'
              : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
          >
            Last 7 days
          </button>
          <button
            on:click={() => (dateFilter = "30d")}
            class="px-3 py-1 rounded text-sm transition {dateFilter === '30d'
              ? 'bg-mapanote-blue-600 text-white'
              : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
          >
            Last 30 days
          </button>

          <div class="ml-auto">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                bind:checked={showPinnedOnly}
                class="rounded border-gray-300 text-mapanote-blue-600 focus:ring-mapanote-blue-500"
              />
              <span class="text-sm text-gray-700 dark:text-gray-300"
                >📌 Pinned only</span
              >
            </label>
          </div>
        </div>

        <!-- Tag filter -->
        {#if allTags.length > 0}
          <div>
            <span class="text-sm text-gray-600 dark:text-gray-400 mb-2 block"
              >Tags:</span
            >
            <div class="flex flex-wrap gap-2">
              {#each allTags as tag}
                <button
                  on:click={() => toggleTag(tag)}
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
      <div class="mb-6">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
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
              {selectedTags.length > 0 || showPinnedOnly || dateFilter !== "all"
                ? "No notes match the current filters."
                : `No notes yet for ${$currentCountry.title}`}
            </p>
            {#if selectedTags.length > 0 || showPinnedOnly || dateFilter !== "all"}
              <button
                on:click={clearFilters}
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
                  <div
                    class="flex items-center gap-3 flex-wrap text-sm text-gray-500 dark:text-gray-400"
                  >
                    <span class="font-medium text-gray-700 dark:text-gray-300"
                      >{note.date}</span
                    >

                    {#if note.pinned}
                      <span class="text-amber-600 dark:text-amber-400"
                        >📌 Pinned</span
                      >
                    {/if}

                    {#if note.tags.length > 0}
                      <span>·</span>
                      <div class="flex gap-2">
                        {#each note.tags as tag}
                          <button
                            on:click={() => toggleTag(tag)}
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

                    {#if note.also.length > 0}
                      <span>·</span>
                      <span class="text-xs">
                        also: {note.also.join(", ")}
                      </span>
                    {/if}

                    <span class="ml-auto text-xs capitalize opacity-60">
                      {note.visibility}
                    </span>
                  </div>

                  <!-- Edit button -->
                  <button
                    on:click={() => handleEditNote(note)}
                    class="text-gray-400 hover:text-mapanote-blue-600 dark:hover:text-mapanote-blue-400 transition"
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

                <!-- Note text -->
                <div class="prose dark:prose-invert max-w-none">
                  <p
                    class="text-gray-700 dark:text-gray-300 whitespace-pre-wrap leading-relaxed"
                  >
                    {note.text}
                  </p>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
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

<!-- Edit Modal (conditionally rendered) -->
{#if editingNote}
  <EditNoteModal note={editingNote} onClose={closeEditModal} />
{/if}

<!-- Export Menu -->
<!-- ← ADD -->
{#if showExportMenu && $currentCountry}
  <ExportMenu
    countrySlug={$currentCountry.slug}
    countryTitle={$currentCountry.title}
    onClose={() => (showExportMenu = false)}
  />
{/if}

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
