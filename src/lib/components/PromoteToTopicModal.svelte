<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { deleteNote, loadCountry, currentCountry } from "$lib/stores/vault";
  import { loadTopics, topics } from "$lib/stores/topics";
  import type { Note, Topic } from "$lib/types";

  interface Props {
    note: Note;
    onClose: () => void;
    onSuccess?: () => void;
  }

  let { note, onClose, onSuccess }: Props = $props();

  let mode: "existing" | "new" = $state("existing");
  let selectedTopicId = $state("");
  let newTopicTitle = $state("");
  let newTopicSummary = $state("");
  let newTopicColor = $state("#3B82F6");
  let additionalCountries = $state<string[]>([]);
  let isSubmitting = $state(false);
  let error = $state("");

  // Color presets
  const colorPresets = [
    "#3B82F6", // Blue
    "#10B981", // Green
    "#F59E0B", // Amber
    "#EF4444", // Red
    "#8B5CF6", // Purple
    "#EC4899", // Pink
    "#14B8A6", // Teal
    "#F97316", // Orange
    "#06B6D4", // Cyan
    "#84CC16", // Lime
    "#F43F5E", // Rose
    "#6366F1", // Indigo
  ];

  async function handlePromote() {
    if (!$currentCountry) return;

    // Validate
    if (mode === "existing" && !selectedTopicId) {
      error = "Please select a topic";
      return;
    }

    if (mode === "new" && !newTopicTitle.trim()) {
      error = "Please enter a topic title";
      return;
    }

    isSubmitting = true;
    error = "";

    try {
      let topicId = selectedTopicId;

      // Create new topic if needed
      if (mode === "new") {
        const newTopic = await invoke<Topic>("create_topic", {
          title: newTopicTitle.trim(),
          summary: newTopicSummary.trim() || null,
          color: newTopicColor,
          countrySlugs: [$currentCountry.slug, ...additionalCountries],
        });

        topicId = newTopic.id;
        await loadTopics(); // Refresh topics list
      }

      // If using existing topic, make sure this country is in the topic's relations
      if (mode === "existing") {
        // Check if country is already in this topic
        const topicDetails = await invoke<any>("get_topic", { topicId });

        if (!topicDetails.countries.includes($currentCountry.slug)) {
          // Add this country to the topic
          await invoke("add_country_to_topic", {
            topicId,
            countrySlug: $currentCountry.slug,
          });
        }
      }

      // Add note to topic
      await invoke("add_topic_note", {
        topicId,
        title: note.title,
        content: note.content,
        tags: note.tags,
        countryTargets: [$currentCountry.slug, ...additionalCountries],
      });

      // Delete original country note
      await deleteNote($currentCountry.slug, note.id);

      // Wait a moment for filesystem to sync
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Reload current country to refresh notes (should now show topic note)
      await loadCountry($currentCountry.slug);

      if (onSuccess) onSuccess();
      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to promote note";
      console.error("Error promoting note:", err);
    } finally {
      isSubmitting = false;
    }
  }

  function toggleCountry(countrySlug: string) {
    if (additionalCountries.includes(countrySlug)) {
      additionalCountries = additionalCountries.filter(
        (c) => c !== countrySlug
      );
    } else {
      additionalCountries = [...additionalCountries, countrySlug];
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

<div
  class="modal-backdrop fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
>
  <div
    class="modal-content bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="p-6">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
          Promote to Topic
        </h2>
        <button
          onclick={onClose}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
        >
          <svg
            class="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      <!-- Note Preview -->
      <div class="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4 mb-6">
        <h3 class="font-semibold text-gray-900 dark:text-gray-100 mb-1">
          {note.title}
        </h3>
        <p class="text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
          {note.content}
        </p>
        {#if note.tags.length > 0}
          <div class="flex gap-1 mt-2">
            {#each note.tags as tag}
              <span
                class="text-xs px-2 py-1 bg-gray-200 dark:bg-gray-600 rounded"
              >
                {tag}
              </span>
            {/each}
          </div>
        {/if}
      </div>

      {#if error}
        <div
          class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 mb-4"
        >
          <p class="text-red-800 dark:text-red-200 text-sm">{error}</p>
        </div>
      {/if}

      <!-- Mode Selection -->
      <div class="mb-6">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
        >
          Promote to:
        </label>
        <div class="flex gap-3">
          <button
            type="button"
            onclick={() => (mode = "existing")}
            class="flex-1 px-4 py-3 rounded-lg border-2 transition {mode ===
            'existing'
              ? 'border-mapanote-blue-600 bg-mapanote-blue-50 dark:bg-mapanote-blue-900/20'
              : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'}"
          >
            <div class="font-medium text-gray-900 dark:text-gray-100">
              Existing Topic
            </div>
            <div class="text-xs text-gray-500 dark:text-gray-400">
              Add to a topic you already have
            </div>
          </button>
          <button
            type="button"
            onclick={() => (mode = "new")}
            class="flex-1 px-4 py-3 rounded-lg border-2 transition {mode ===
            'new'
              ? 'border-mapanote-blue-600 bg-mapanote-blue-50 dark:bg-mapanote-blue-900/20'
              : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500'}"
          >
            <div class="font-medium text-gray-900 dark:text-gray-100">
              New Topic
            </div>
            <div class="text-xs text-gray-500 dark:text-gray-400">
              Create a new cross-country topic
            </div>
          </button>
        </div>
      </div>

      {#if mode === "existing"}
        <!-- Select Existing Topic -->
        <div class="mb-6">
          <label
            class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
          >
            Select Topic
          </label>
          {#if $topics.length === 0}
            <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
              No topics yet. Create a new topic instead.
            </p>
          {:else}
            <div class="space-y-2">
              {#each $topics as topic}
                <button
                  type="button"
                  onclick={() => (selectedTopicId = topic.id)}
                  class="w-full text-left px-4 py-3 rounded-lg border-2 transition {selectedTopicId ===
                  topic.id
                    ? 'border-mapanote-blue-600 bg-mapanote-blue-50 dark:bg-mapanote-blue-900/20'
                    : 'border-gray-200 dark:border-gray-600 hover:border-gray-300'}"
                  style="border-left: 4px solid {topic.color || '#3B82F6'}"
                >
                  <div class="font-medium text-gray-900 dark:text-gray-100">
                    {topic.title}
                    {#if topic.pinned}
                      <span class="text-xs ml-1">📌</span>
                    {/if}
                  </div>
                  {#if topic.summary}
                    <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      {topic.summary}
                    </div>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {:else}
        <!-- Create New Topic -->
        <div class="space-y-4 mb-6">
          <div>
            <label
              class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
            >
              Topic Title *
            </label>
            <input
              type="text"
              bind:value={newTopicTitle}
              placeholder="e.g., USSR Architecture, Cold War Relations"
              class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                     bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100
                     focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent"
            />
          </div>

          <div>
            <label
              class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
            >
              Topic Summary (optional)
            </label>
            <textarea
              bind:value={newTopicSummary}
              placeholder="Describe what this topic is about..."
              rows="2"
              class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                     bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100
                     focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent resize-none"
            ></textarea>
          </div>

          <div>
            <label
              class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
            >
              Topic Color
            </label>
            <div class="flex gap-2 flex-wrap mb-3">
              {#each colorPresets as color}
                <button
                  type="button"
                  onclick={() => (newTopicColor = color)}
                  class="w-10 h-10 rounded-lg border-2 transition {newTopicColor ===
                  color
                    ? 'border-gray-900 dark:border-white scale-110'
                    : 'border-gray-300 dark:border-gray-600'}"
                  style="background-color: {color}"
                  title={color}
                ></button>
              {/each}
            </div>
            <!-- Custom color input -->
            <div class="flex gap-2 items-center">
              <input
                type="color"
                bind:value={newTopicColor}
                class="w-12 h-10 rounded border border-gray-300 dark:border-gray-600 cursor-pointer"
              />
              <input
                type="text"
                bind:value={newTopicColor}
                placeholder="#3B82F6"
                maxlength="7"
                class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                       bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm
                       focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent font-mono"
              />
            </div>
          </div>
        </div>
      {/if}

      <!-- Additional Countries (Optional) -->
      <div class="mb-6">
        <label
          class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
        >
          Additional Countries (optional)
        </label>
        <p class="text-xs text-gray-500 dark:text-gray-400 mb-3">
          This note will appear on {$currentCountry?.name} and any additional countries
          you select
        </p>
        <!-- You could add a country picker here, but for now just show a note -->
        <div class="text-xs text-gray-400 italic">
          Note: Additional countries can be added from the topic page after
          promotion
        </div>
      </div>

      <!-- Actions -->
      <div class="flex justify-end gap-3">
        <button
          type="button"
          onclick={onClose}
          class="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600
                 text-gray-700 dark:text-gray-300 rounded-lg transition"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handlePromote}
          disabled={isSubmitting ||
            (mode === "existing" && !selectedTopicId) ||
            (mode === "new" && !newTopicTitle.trim())}
          class="px-6 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
                 disabled:bg-gray-400 disabled:cursor-not-allowed
                 text-white rounded-lg font-medium transition"
        >
          {isSubmitting ? "Promoting..." : "Promote to Topic"}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
