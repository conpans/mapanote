<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { vaultOpened } from "$lib/stores/vault";
  import { topics } from "$lib/stores/topics";
  import { mapStats } from "$lib/stores/mapStats";
  import type { RecentActivity } from "$lib/types";
  import WelcomeScreen from "$lib/components/WelcomeScreen.svelte";
  import WorldMap from "$lib/components/WorldMap.svelte";
  import { goto } from "$app/navigation";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { openSearch } from "$lib/stores/searchStore";

  let recentActivities = $state<RecentActivity[]>([]);
  let isLoadingActivity = $state(false);

  // Derived stats
  let totalCountries = $derived($mapStats.size);
  let totalNotes = $derived(
    Array.from($mapStats.values()).reduce(
      (sum, stat) => sum + stat.noteCount,
      0
    )
  );
  let mostActiveCountry = $derived(() => {
    let maxNotes = 0;
    let topSlug = "";

    $mapStats.forEach((stat, slug) => {
      if (stat.noteCount > maxNotes) {
        maxNotes = stat.noteCount;
        topSlug = slug;
      }
    });

    return topSlug ? { slug: topSlug, count: maxNotes } : null;
  });

  // Pinned topics
  let pinnedTopics = $derived($topics.filter((t) => t.pinned));
  let recentTopics = $derived(
    $topics
      .filter((t) => !t.pinned)
      .sort(
        (a, b) =>
          new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
      )
      .slice(0, 5)
  );

  onMount(() => {
    if ($vaultOpened) {
      loadRecentActivity();
    }
  });

  async function loadRecentActivity() {
    isLoadingActivity = true;
    try {
      recentActivities = await invoke<RecentActivity[]>("get_recent_activity", {
        limit: 12,
      });
    } catch (error) {
      console.error("Failed to load recent activity:", error);
    } finally {
      isLoadingActivity = false;
    }
  }

  function handleVaultOpened(event: CustomEvent<string>) {
    console.log("Vault opened:", event.detail);
    loadRecentActivity();
  }

  function formatSlugToName(slug: string): string {
    return slug
      .split("-")
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(" ");
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffDays = Math.floor(
      (now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24)
    );

    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "Yesterday";
    if (diffDays < 7) return `${diffDays} days ago`;

    return date.toLocaleDateString();
  }
</script>

{#if !$vaultOpened}
  <WelcomeScreen on:vaultOpened={handleVaultOpened} />
{:else}
  <main class="min-h-screen bg-gray-50 dark:bg-gray-900 p-4">
    <div class="max-w-full mx-auto">
      <!-- Header -->
      <div class="mb-4">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-4xl font-bold text-mapanote-blue-600 mb-2">
              Mapanote
            </h1>
            <p class="text-gray-600 dark:text-gray-400">
              Local-first geopolitics notebook
            </p>
          </div>

          <div class="flex items-center gap-3">
            <ThemeToggle />

            <button
              onclick={() => goto("/topics")}
              class="px-4 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
           rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition
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
                  d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z"
                />
              </svg>
              <span>Topics</span>
            </button>

            <button
              onclick={openSearch}
              class="px-4 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
           rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition
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
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
              <span>Search</span>
              <kbd
                class="text-xs px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-700
               border border-gray-300 dark:border-gray-600"
              >
                Ctrl+K
              </kbd>
            </button>
          </div>
        </div>
      </div>

      <!-- Topics Rail -->
      {#if $topics.length > 0}
        <div class="mb-4">
          <div class="flex items-center gap-2 mb-2">
            <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
              Topics
            </h2>
            <span class="text-xs text-gray-500 dark:text-gray-400">
              ({$topics.length})
            </span>
          </div>

          <div class="flex gap-2 overflow-x-auto pb-2 scrollbar-thin">
            {#each pinnedTopics as topic}
              <button
                onclick={() => goto(`/topic/${topic.id}`)}
                class="flex-shrink-0 px-4 py-2 rounded-lg border-2 transition
                       hover:shadow-md"
                style="border-color: {topic.color || '#3b82f6'}; 
                       background-color: {topic.color
                  ? `${topic.color}15`
                  : '#dbeafe'}"
              >
                <div class="flex items-center gap-2">
                  <span
                    class="text-sm font-medium"
                    style="color: {topic.color || '#1e40af'}"
                  >
                    📌 {topic.title}
                  </span>
                </div>
              </button>
            {/each}

            {#each recentTopics as topic}
              <button
                onclick={() => goto(`/topic/${topic.id}`)}
                class="flex-shrink-0 px-4 py-2 bg-white dark:bg-gray-800
                       border border-gray-200 dark:border-gray-700 rounded-lg
                       hover:bg-gray-50 dark:hover:bg-gray-700 transition"
              >
                <span class="text-sm text-gray-700 dark:text-gray-300">
                  {topic.title}
                </span>
              </button>
            {/each}

            <button
              onclick={() => goto("/topics")}
              class="flex-shrink-0 px-4 py-2 bg-white dark:bg-gray-800
                     border border-gray-200 dark:border-gray-700 rounded-lg
                     hover:bg-gray-50 dark:hover:bg-gray-700 transition
                     text-gray-500 dark:text-gray-400 text-sm"
            >
              View all →
            </button>
          </div>
        </div>
      {/if}

      <!-- Stats Cards -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
        <div
          class="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700"
        >
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                Countries Tracked
              </p>
              <p
                class="text-3xl font-bold text-gray-900 dark:text-gray-100 mt-1"
              >
                {totalCountries}
              </p>
            </div>
            <div class="text-4xl">🌍</div>
          </div>
        </div>

        <div
          class="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700"
        >
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                Total Notes
              </p>
              <p
                class="text-3xl font-bold text-gray-900 dark:text-gray-100 mt-1"
              >
                {totalNotes}
              </p>
            </div>
            <div class="text-4xl">📝</div>
          </div>
        </div>

        <div
          class="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700"
        >
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                Most Active
              </p>
              {#if mostActiveCountry()}
                {@const country = mostActiveCountry()}
                {#if country}
                  <p
                    class="text-xl font-bold text-gray-900 dark:text-gray-100 mt-1"
                  >
                    {formatSlugToName(country.slug)}
                  </p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">
                    {country.count} notes
                  </p>
                {:else}
                  <p
                    class="text-xl font-bold text-gray-400 dark:text-gray-500 mt-1"
                  >
                    None yet
                  </p>
                {/if}
              {:else}
                <p
                  class="text-xl font-bold text-gray-400 dark:text-gray-500 mt-1"
                >
                  None yet
                </p>
              {/if}
            </div>
            <div class="text-4xl">🔥</div>
          </div>
        </div>
      </div>

      <!-- Main Content: Map + Activity Stream Side-by-Side -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 mt-4">
        <!-- Map (2/3 width) -->
        <div class="lg:col-span-2 min-h-0">
          <WorldMap />
        </div>

        <!-- Activity Stream (1/3 width) -->
        <div
          class="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700 h-fit max-h-[calc(100vh-350px)] overflow-y-auto"
        >
          <h2
            class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4"
          >
            Recent Activity
          </h2>

          {#if isLoadingActivity}
            <div class="space-y-3">
              {#each Array(5) as _}
                <div class="animate-pulse">
                  <div
                    class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4 mb-2"
                  ></div>
                  <div
                    class="h-3 bg-gray-200 dark:bg-gray-700 rounded w-1/2"
                  ></div>
                </div>
              {/each}
            </div>
          {:else if recentActivities.length === 0}
            <p
              class="text-gray-500 dark:text-gray-400 text-sm text-center py-8"
            >
              No notes yet. Start adding notes to countries!
            </p>
          {:else}
            <div class="space-y-3">
              {#each recentActivities as activity}
                <button
                  onclick={() => goto(`/country/${activity.country_slug}`)}
                  class="w-full text-left p-3 rounded-lg border border-gray-200 dark:border-gray-700
                         hover:bg-gray-50 dark:hover:bg-gray-700 transition group"
                >
                  <div class="flex items-start justify-between gap-2">
                    <div class="flex-1 min-w-0">
                      <h3
                        class="font-medium text-gray-900 dark:text-gray-100 truncate text-sm group-hover:text-mapanote-blue-600 dark:group-hover:text-mapanote-blue-400"
                      >
                        {activity.note_title}
                      </h3>
                      <div class="flex items-center gap-2 mt-1">
                        <span class="text-xs text-gray-600 dark:text-gray-400">
                          {activity.country_name}
                        </span>
                        {#if activity.source_type === "topic" && activity.topic_name}
                          <span
                            class="text-xs px-2 py-0.5 rounded"
                            style="background-color: {activity.topic_color
                              ? `${activity.topic_color}20`
                              : '#e0e7ff'}; color: {activity.topic_color ||
                              '#4f46e5'}"
                          >
                            📚 {activity.topic_name}
                          </span>
                        {/if}
                      </div>
                      <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        {formatDate(activity.note_date)}
                      </p>
                    </div>
                    <svg
                      class="w-4 h-4 text-gray-400 flex-shrink-0 mt-1"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M9 5l7 7-7 7"
                      />
                    </svg>
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </main>
{/if}

<style>
  .scrollbar-thin::-webkit-scrollbar {
    height: 6px;
  }

  .scrollbar-thin::-webkit-scrollbar-track {
    background: #f3f4f6;
    border-radius: 3px;
  }

  .scrollbar-thin::-webkit-scrollbar-thumb {
    background: #d1d5db;
    border-radius: 3px;
  }

  .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background: #9ca3af;
  }

  :global(.dark) .scrollbar-thin::-webkit-scrollbar-track {
    background: #374151;
  }

  :global(.dark) .scrollbar-thin::-webkit-scrollbar-thumb {
    background: #4b5563;
  }

  :global(.dark) .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }
</style>
