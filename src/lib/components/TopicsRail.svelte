<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { topics, loadTopics } from "$lib/stores/topics";

  onMount(async () => {
    await loadTopics();
  });

  function goToTopic(topicId: string) {
    goto(`/topic/${topicId}`);
  }

  function goToTopicsPage() {
    goto("/topics");
  }
</script>

<div
  class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
>
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">
      Topics
    </h2>
    <button
      onclick={goToTopicsPage}
      class="text-sm text-mapanote-blue-600 hover:text-mapanote-blue-700 dark:text-mapanote-blue-400"
    >
      View all →
    </button>
  </div>

  {#if $topics.length === 0}
    <p class="text-gray-500 dark:text-gray-400 text-sm">
      No topics yet. Create your first topic to organize notes across countries!
    </p>
  {:else}
    <div class="flex gap-3 overflow-x-auto pb-2 -mx-2 px-2 scrollbar-thin">
      {#each $topics.slice(0, 10) as topic}
        <button
          onclick={() => goToTopic(topic.id)}
          class="flex-shrink-0 px-4 py-3 rounded-lg border-2 transition
                 hover:border-mapanote-blue-500 hover:shadow-md
                 bg-white dark:bg-gray-700
                 border-gray-200 dark:border-gray-600
                 min-w-[200px]"
          style="border-left: 4px solid {topic.color || '#3B82F6'}"
        >
          <div class="text-left">
            <h3 class="font-semibold text-gray-900 dark:text-gray-100 mb-1">
              {topic.title}
              {#if topic.pinned}
                <span class="text-amber-500">📌</span>
              {/if}
            </h3>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              {topic.countries.length} countries • {topic.note_count} notes
            </p>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .scrollbar-thin::-webkit-scrollbar {
    height: 6px;
  }

  .scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
  }

  .scrollbar-thin::-webkit-scrollbar-thumb {
    background: rgba(156, 163, 175, 0.5);
    border-radius: 3px;
  }

  .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background: rgba(156, 163, 175, 0.7);
  }
</style>
