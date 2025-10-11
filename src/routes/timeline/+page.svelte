<script lang="ts">
  import { goto } from "$app/navigation";
  import { vaultOpened } from "$lib/stores/vault";
  import Timeline from "$lib/components/Timeline.svelte";

  // Redirect if no vault is open
  $: if (!$vaultOpened) {
    goto("/");
  }
</script>

<main class="min-h-screen bg-gray-50 dark:bg-gray-900 p-8">
  <div class="max-w-6xl mx-auto">
    <!-- Back button -->
    <button
      on:click={() => goto("/")}
      class="mb-6 text-mapanote-blue-600 hover:underline inline-flex items-center gap-1"
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

    <Timeline />
  </div>
</main>
