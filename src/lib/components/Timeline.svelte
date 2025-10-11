<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";

  interface TimelineEntry {
    note_id: string;
    country_slug: string;
    country_title: string;
    date: string;
    tags: string[];
    text: string;
    visibility: string;
    pinned: boolean;
  }

  let entries: TimelineEntry[] = [];
  let isLoading = true;
  let groupBy: "day" | "week" | "month" = "day";
  let limit = 50;

  onMount(async () => {
    await loadTimeline();
  });

  async function loadTimeline() {
    isLoading = true;

    try {
      // Use search with empty query to get all notes
      const results = await invoke<TimelineEntry[]>("search_notes", {
        query: "",
      });

      // Already sorted by date (newest first) from backend
      entries = results.slice(0, limit);
    } catch (error) {
      console.error("Failed to load timeline:", error);
    } finally {
      isLoading = false;
    }
  }

  function handleNoteClick(entry: TimelineEntry) {
    goto(`/country/${entry.country_slug}`);
  }

  // Group entries by date
  $: groupedEntries = (() => {
    const groups = new Map<string, TimelineEntry[]>();

    entries.forEach((entry) => {
      const date = new Date(entry.date);
      let key: string;

      if (groupBy === "day") {
        key = entry.date;
      } else if (groupBy === "week") {
        const startOfWeek = new Date(date);
        startOfWeek.setDate(date.getDate() - date.getDay());
        key = startOfWeek.toISOString().split("T")[0];
      } else {
        key = entry.date.substring(0, 7); // YYYY-MM
      }

      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key)!.push(entry);
    });

    return Array.from(groups.entries()).sort((a, b) =>
      b[0].localeCompare(a[0])
    );
  })();

  function formatGroupLabel(dateStr: string): string {
    const date = new Date(dateStr);

    if (groupBy === "day") {
      const today = new Date();
      const yesterday = new Date(today);
      yesterday.setDate(yesterday.getDate() - 1);

      if (dateStr === today.toISOString().split("T")[0]) {
        return "Today";
      } else if (dateStr === yesterday.toISOString().split("T")[0]) {
        return "Yesterday";
      }

      return date.toLocaleDateString("en-US", {
        weekday: "long",
        year: "numeric",
        month: "long",
        day: "numeric",
      });
    } else if (groupBy === "week") {
      const endOfWeek = new Date(date);
      endOfWeek.setDate(date.getDate() + 6);
      return `Week of ${date.toLocaleDateString("en-US", { month: "short", day: "numeric" })} - ${endOfWeek.toLocaleDateString("en-US", { month: "short", day: "numeric" })}`;
    } else {
      return date.toLocaleDateString("en-US", {
        year: "numeric",
        month: "long",
      });
    }
  }
</script>

<div class="timeline-container">
  <!-- Header -->
  <div
    class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 mb-6"
  >
    <div class="flex items-center justify-between mb-4">
      <div>
        <h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100">
          Timeline
        </h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
          Recent activity across all countries
        </p>
      </div>

      <!-- Group by controls -->
      <div class="flex gap-2">
        <button
          on:click={() => (groupBy = "day")}
          class="px-3 py-2 rounded text-sm transition {groupBy === 'day'
            ? 'bg-mapanote-blue-600 text-white'
            : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
        >
          By Day
        </button>
        <button
          on:click={() => (groupBy = "week")}
          class="px-3 py-2 rounded text-sm transition {groupBy === 'week'
            ? 'bg-mapanote-blue-600 text-white'
            : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
        >
          By Week
        </button>
        <button
          on:click={() => (groupBy = "month")}
          class="px-3 py-2 rounded text-sm transition {groupBy === 'month'
            ? 'bg-mapanote-blue-600 text-white'
            : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
        >
          By Month
        </button>
      </div>
    </div>

    <div class="text-sm text-gray-600 dark:text-gray-400">
      Showing {entries.length} most recent notes
    </div>
  </div>

  <!-- Timeline entries -->
  {#if isLoading}
    <div class="flex items-center justify-center py-12">
      <div class="text-gray-600 dark:text-gray-400">Loading timeline...</div>
    </div>
  {:else if entries.length === 0}
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-12 text-center"
    >
      <p class="text-gray-500 dark:text-gray-400">No notes in your vault yet</p>
    </div>
  {:else}
    <div class="space-y-8">
      {#each groupedEntries as [dateKey, groupEntries]}
        <div class="timeline-group">
          <!-- Date header -->
          <div class="sticky top-0 bg-gray-50 dark:bg-gray-900 py-3 mb-4 z-10">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {formatGroupLabel(dateKey)}
            </h3>
            <div class="text-sm text-gray-500 dark:text-gray-400">
              {groupEntries.length}
              {groupEntries.length === 1 ? "note" : "notes"}
            </div>
          </div>

          <!-- Notes in this group -->
          <div class="space-y-3">
            {#each groupEntries as entry}
              <button
                on:click={() => handleNoteClick(entry)}
                class="w-full text-left bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700
                       p-4 hover:shadow-md hover:border-mapanote-blue-300 dark:hover:border-mapanote-blue-700 transition"
              >
                <div class="flex items-start justify-between mb-2">
                  <div class="flex items-center gap-3">
                    <span
                      class="font-semibold text-gray-900 dark:text-gray-100"
                    >
                      {entry.country_title}
                    </span>
                    {#if entry.pinned}
                      <span class="text-amber-600 dark:text-amber-400">📌</span>
                    {/if}
                    {#if entry.tags.length > 0}
                      <div class="flex gap-2">
                        {#each entry.tags as tag}
                          <span
                            class="text-xs bg-mapanote-blue-100 dark:bg-mapanote-blue-900
                                     text-mapanote-blue-700 dark:text-mapanote-blue-300 px-2 py-1 rounded"
                          >
                            {tag}
                          </span>
                        {/each}
                      </div>
                    {/if}
                  </div>
                  <span class="text-sm text-gray-500 dark:text-gray-400">
                    {entry.date}
                  </span>
                </div>

                <p
                  class="text-sm text-gray-700 dark:text-gray-300 line-clamp-2"
                >
                  {entry.text}
                </p>
              </button>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .timeline-container {
    max-width: 4xl;
    margin: 0 auto;
  }

  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
