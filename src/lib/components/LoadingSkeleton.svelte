<script lang="ts">
  export let type: "map" | "country" | "stats" | "text" = "text";
  export let count: number = 1;
</script>

<div class="animate-pulse">
  {#if type === "map"}
    <!-- Map skeleton -->
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-8"
    >
      <div class="h-96 bg-gray-200 dark:bg-gray-700 rounded"></div>
    </div>
  {:else if type === "country"}
    <!-- Country detail skeleton -->
    <div class="space-y-4">
      <!-- Header -->
      <div class="h-12 bg-gray-200 dark:bg-gray-700 rounded w-1/3"></div>
      <div class="h-6 bg-gray-200 dark:bg-gray-700 rounded w-1/4"></div>
      <div class="h-20 bg-gray-200 dark:bg-gray-700 rounded w-full"></div>

      <!-- Notes -->
      <div class="space-y-4 mt-8">
        {#each Array(count) as _}
          <div
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
          >
            <div
              class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/4 mb-3"
            ></div>
            <div
              class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-full mb-2"
            ></div>
            <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-5/6"></div>
          </div>
        {/each}
      </div>
    </div>
  {:else if type === "stats"}
    <!-- Stats cards skeleton -->
    <div class="grid grid-cols-4 gap-4">
      {#each Array(4) as _}
        <div
          class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4"
        >
          <div
            class="h-8 bg-gray-200 dark:bg-gray-700 rounded w-1/2 mb-2"
          ></div>
          <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4"></div>
        </div>
      {/each}
    </div>
  {:else}
    <!-- Generic text skeleton -->
    {#each Array(count) as _}
      <div class="h-4 bg-gray-200 dark:bg-gray-700 rounded w-full mb-2"></div>
    {/each}
  {/if}
</div>

<style>
  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .animate-pulse {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }
</style>
