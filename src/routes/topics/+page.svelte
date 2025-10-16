<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { topics, loadTopics, isLoadingTopics } from "$lib/stores/topics";
  import CreateTopicModal from "$lib/components/CreateTopicModal.svelte";
  import LoadingSkeleton from "$lib/components/LoadingSkeleton.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";

  let showCreateModal = $state(false);
  let filter: "all" | "pinned" | "recent" = $state("all");

  onMount(() => {
    loadTopics();
  });

  // Filter topics
  let filteredTopics = $derived(() => {
    const all = $topics;
    if (filter === "pinned") {
      return all.filter((t) => t.pinned);
    } else if (filter === "recent") {
      return [...all]
        .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
        .slice(0, 10);
    }
    return all;
  });

  function goBack() {
    goto("/");
  }
</script>

<main class="min-h-screen bg-gray-50 dark:bg-gray-900 p-4">
  <div class="max-w-7xl mx-auto">
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
          Back to map
        </button>

        <ThemeToggle />
      </div>

      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-2">
            Topics
          </h1>
          <p class="text-gray-600 dark:text-gray-400">
            Organize notes across multiple countries
          </p>
        </div>

        <button
          on:click={() => (showCreateModal = true)}
          class="px-4 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
                 text-white rounded-lg font-medium transition
                 flex items-center gap-2"
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
              d="M12 4v16m8-8H4"
            />
          </svg>
          New Topic
        </button>
      </div>
    </div>

    <!-- Filters -->
    <div class="flex gap-2 mb-6">
      <button
        on:click={() => (filter = "all")}
        class="px-4 py-2 rounded-lg transition {filter === 'all'
          ? 'bg-mapanote-blue-600 text-white'
          : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700'}"
      >
        All Topics
      </button>
      <button
        on:click={() => (filter = "pinned")}
        class="px-4 py-2 rounded-lg transition {filter === 'pinned'
          ? 'bg-mapanote-blue-600 text-white'
          : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700'}"
      >
        📌 Pinned
      </button>
      <button
        on:click={() => (filter = "recent")}
        class="px-4 py-2 rounded-lg transition {filter === 'recent'
          ? 'bg-mapanote-blue-600 text-white'
          : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700'}"
      >
        Recent
      </button>
    </div>

    <!-- Topics Grid -->
    {#if $isLoadingTopics}
      <LoadingSkeleton type="list" count={6} />
    {:else if filteredTopics().length === 0}
      <div
        class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-12 text-center"
      >
        <svg
          class="w-16 h-16 mx-auto mb-4 text-gray-400"
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
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
          No topics yet
        </h3>
        <p class="text-gray-600 dark:text-gray-400 mb-4">
          Create your first topic to organize notes across countries
        </p>
        <button
          on:click={() => (showCreateModal = true)}
          class="px-6 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
                 text-white rounded-lg font-medium transition"
        >
          Create Topic
        </button>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each filteredTopics() as topic}
          <button
            on:click={() => goto(`/topic/${topic.id}`)}
            class="bg-white dark:bg-gray-800 rounded-lg border-2 border-gray-200 dark:border-gray-700
                   hover:border-mapanote-blue-500 dark:hover:border-mapanote-blue-500
                   p-4 text-left transition group"
            style="border-left: 4px solid {topic.color || '#3B82F6'}"
          >
            <div class="flex items-start justify-between mb-2">
              <h3
                class="text-lg font-semibold text-gray-900 dark:text-gray-100 group-hover:text-mapanote-blue-600 dark:group-hover:text-mapanote-blue-400"
              >
                {topic.title}
              </h3>
              {#if topic.pinned}
                <span class="text-amber-500">📌</span>
              {/if}
            </div>

            {#if topic.summary}
              <p
                class="text-sm text-gray-600 dark:text-gray-400 mb-3 line-clamp-2"
              >
                {topic.summary}
              </p>
            {/if}

            <div class="flex items-center justify-between text-sm">
              <span class="text-gray-500 dark:text-gray-400">
                {topic.countries.length}
                {topic.countries.length === 1 ? "country" : "countries"}
              </span>
              <span class="text-gray-500 dark:text-gray-400">
                {topic.note_count}
                {topic.note_count === 1 ? "note" : "notes"}
              </span>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</main>

<!-- Create Modal -->
{#if showCreateModal}
  <CreateTopicModal onClose={() => (showCreateModal = false)} />
{/if}

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
