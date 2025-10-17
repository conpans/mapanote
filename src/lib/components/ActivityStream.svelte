<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import type { Note, CountryMetadata } from "$lib/types";

  interface ActivityItem {
    note: Note;
    countrySlug: string;
    countryName: string;
    isTopicNote: boolean;
    topicName?: string;
  }

  let activities = $state<ActivityItem[]>([]);
  let isLoading = $state(true);
  let countriesMetadata = $state<Map<string, CountryMetadata>>(new Map());

  onMount(async () => {
    await loadActivities();
  });

  async function loadActivities() {
    try {
      // Load country metadata
      const metadata = await invoke<CountryMetadata[]>(
        "get_all_countries_metadata"
      );
      countriesMetadata = new Map(metadata.map((c) => [c.slug, c]));

      // Get all countries
      const countries = await invoke<CountryMetadata[]>(
        "get_all_countries_metadata"
      );

      const allActivities: ActivityItem[] = [];

      // Collect notes from each country
      for (const country of countries.slice(0, 20)) {
        // Limit to prevent slowdown
        try {
          const notes = await invoke<Note[]>("get_country_notes", {
            slug: country.slug,
          });

          for (const note of notes) {
            allActivities.push({
              note,
              countrySlug: country.slug,
              countryName: country.name,
              isTopicNote: !!note.topic_id,
              topicName: note.topic_id ? "Topic" : undefined,
            });
          }
        } catch (err) {
          console.error(`Failed to load notes for ${country.slug}:`, err);
        }
      }

      // Sort by date (newest first) and take top 10
      activities = allActivities
        .sort((a, b) => b.note.date.localeCompare(a.note.date))
        .slice(0, 10);
    } catch (error) {
      console.error("Failed to load activities:", error);
    } finally {
      isLoading = false;
    }
  }

  function goToCountry(slug: string) {
    goto(`/country/${slug}`);
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffTime = Math.abs(now.getTime() - date.getTime());
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "Yesterday";
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    return date.toLocaleDateString();
  }
</script>

<div
  class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
>
  <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
    Recent Activity
  </h2>

  {#if isLoading}
    <div class="space-y-3">
      {#each Array(5) as _}
        <div class="animate-pulse">
          <div
            class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4 mb-2"
          ></div>
          <div class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
        </div>
      {/each}
    </div>
  {:else if activities.length === 0}
    <p class="text-gray-500 dark:text-gray-400 text-sm">
      No recent activity. Start by adding notes to your countries!
    </p>
  {:else}
    <div class="space-y-3">
      {#each activities as activity}
        <button
          onclick={() => goToCountry(activity.countrySlug)}
          class="w-full text-left p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition group"
        >
          <div class="flex items-start justify-between mb-1">
            <div class="flex-1">
              <h4
                class="font-medium text-gray-900 dark:text-gray-100 group-hover:text-mapanote-blue-600 dark:group-hover:text-mapanote-blue-400"
              >
                {activity.note.title}
              </h4>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                {activity.countryName}
                {#if activity.isTopicNote}
                  <span
                    class="ml-1 px-1.5 py-0.5 bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded"
                  >
                    📚 Topic
                  </span>
                {/if}
              </p>
            </div>
            <span
              class="text-xs text-gray-400 dark:text-gray-500 whitespace-nowrap ml-2"
            >
              {formatDate(activity.note.date)}
            </span>
          </div>
          <p class="text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
            {activity.note.content}
          </p>
        </button>
      {/each}
    </div>
  {/if}
</div>
