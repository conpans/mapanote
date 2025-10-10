<script lang="ts">
  import { page } from "$app/stores";

  // Get country slug from URL
  $: slug = $page.params.slug;
</script>

<main class="min-h-screen bg-gray-50 dark:bg-gray-900 p-8">
  <div class="max-w-5xl mx-auto">
    <h1 class="text-4xl font-bold text-mapanote-blue-600 mb-4">
      Country: {slug}
    </h1>
    <p class="text-gray-600 dark:text-gray-400">Country page coming soon...</p>
    <a
      href="/"
      class="text-mapanote-blue-600 hover:underline mt-4 inline-block"
    >
      ← Back to map
    </a>
  </div>
</main>
