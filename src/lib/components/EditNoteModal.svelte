<script lang="ts">
  import { updateNote, deleteNote, currentCountry } from "$lib/stores/vault";
  import { deleteTopicNote } from "$lib/stores/topics";
  import type { NoteWithSource } from "$lib/types";
  import ImageUploader from "./ImageUploader.svelte";
  import MarkdownToolbar from "./MarkdownToolbar.svelte";

  interface Props {
    noteWithSource: NoteWithSource; // ← Changed prop name
    onClose: () => void;
  }

  let { noteWithSource, onClose }: Props = $props();

  let title = $state(noteWithSource.title); // ← Direct access, not .noteWithSource.title
  let content = $state(noteWithSource.content);
  let tagsInput = $state(noteWithSource.tags.join(", "));
  let isSubmitting = $state(false);
  let isDeleting = $state(false);
  let showDeleteConfirm = $state(false);
  let error = $state("");
  let contentTextarea: HTMLTextAreaElement;

  // Determine if this is a topic note
  let isTopicNote = $derived(noteWithSource.source_type === "topic");

  async function handleSave() {
    if (!content.trim() || !title.trim() || !$currentCountry) return;

    // For topic notes, show a message that they should edit from the topic page
    if (isTopicNote) {
      error = "Topic notes can only be edited from the topic page.";
      return;
    }

    isSubmitting = true;
    error = "";

    try {
      const tags = tagsInput
        .split(",")
        .map((t) => t.trim())
        .filter((t) => t.length > 0);

      await updateNote(
        $currentCountry.slug,
        noteWithSource.id,
        title.trim(),
        content.trim(),
        tags
      );

      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to update note";
      console.error("Error updating note:", err);
    } finally {
      isSubmitting = false;
    }
  }

  async function handleDelete() {
    isDeleting = true;
    error = "";

    try {
      if (isTopicNote && noteWithSource.topic_id) {
        // Delete topic note
        await deleteTopicNote(noteWithSource.topic_id, noteWithSource.id);
      } else if ($currentCountry) {
        // Delete country note
        await deleteNote($currentCountry.slug, noteWithSource.id);
      }

      onClose();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to delete note";
      console.error("Error deleting note:", err);
      isDeleting = false;
    }
  }

  function handleImageInserted(event: CustomEvent<{ markdown: string }>) {
    const { markdown } = event.detail;

    if (contentTextarea) {
      const start = contentTextarea.selectionStart;
      const end = contentTextarea.selectionEnd;
      const text = content;

      content =
        text.substring(0, start) + "\n" + markdown + "\n" + text.substring(end);

      setTimeout(() => {
        contentTextarea.selectionStart = contentTextarea.selectionEnd =
          start + markdown.length + 2;
        contentTextarea.focus();
      }, 0);
    } else {
      content = content + "\n" + markdown;
    }
  }

  function handleMarkdownInsert(event: CustomEvent<{ markdown: string }>) {
    const { markdown } = event.detail;

    if (contentTextarea) {
      const start = contentTextarea.selectionStart;
      const end = contentTextarea.selectionEnd;
      const selectedText = content.substring(start, end);
      const text = content;

      let insertText = markdown;

      if (selectedText && markdown.includes("text")) {
        insertText = markdown.replace("text", selectedText);
      }

      content = text.substring(0, start) + insertText + text.substring(end);

      setTimeout(() => {
        if (selectedText) {
          contentTextarea.selectionStart = start;
          contentTextarea.selectionEnd = start + insertText.length;
        } else {
          contentTextarea.selectionStart = contentTextarea.selectionEnd =
            start + insertText.length;
        }
        contentTextarea.focus();
      }, 0);
    } else {
      content = content + markdown;
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="modal-backdrop fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
>
  <div
    class="modal-content bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-3xl w-full max-h-[90vh] overflow-y-auto"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="p-6">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {isTopicNote ? "View Topic Note" : "Edit Note"}
          </h2>
          {#if isTopicNote}
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
              From topic: {noteWithSource.source_name}
            </p>
          {/if}
        </div>
        <button
          onclick={onClose}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          aria-label="Close"
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

      {#if error}
        <div
          class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 mb-4"
        >
          <p class="text-red-800 dark:text-red-200 text-sm">{error}</p>
        </div>
      {/if}

      {#if isTopicNote}
        <div
          class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-3 mb-4"
        >
          <p class="text-blue-800 dark:text-blue-200 text-sm">
            💡 This is a topic noteWithSource. To edit it, go to the topic page.
            You can only delete it from here.
          </p>
        </div>
      {/if}

      <form
        onsubmit={(e) => {
          e.preventDefault();
          handleSave();
        }}
      >
        <!-- Title -->
        <input
          type="text"
          bind:value={title}
          placeholder="Note title..."
          disabled={isTopicNote}
          class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600
                 rounded-lg bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100
                 placeholder-gray-400 dark:placeholder-gray-500
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
                 disabled:opacity-50 disabled:cursor-not-allowed
                 mb-4"
        />

        {#if !isTopicNote}
          <!-- Markdown Toolbar -->
          <MarkdownToolbar on:insertMarkdown={handleMarkdownInsert} />
        {/if}

        <!-- Content -->
        <textarea
          bind:this={contentTextarea}
          bind:value={content}
          placeholder="Write your noteWithSource..."
          rows="10"
          disabled={isTopicNote}
          class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600
                 rounded-lg bg-white dark:bg-gray-700
                 text-gray-900 dark:text-gray-100
                 placeholder-gray-400 dark:placeholder-gray-500
                 focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
                 disabled:opacity-50 disabled:cursor-not-allowed
                 resize-y mb-3 font-mono text-sm"
        ></textarea>

        {#if !isTopicNote}
          <!-- Image Uploader -->
          <div class="mb-4">
            <ImageUploader
              countrySlug={$currentCountry?.slug}
              on:imageInserted={handleImageInserted}
            />
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              💡 Tip: Paste images with Ctrl+V
            </p>
          </div>

          <!-- Tags -->
          <input
            type="text"
            bind:value={tagsInput}
            placeholder="Tags (comma-separated): politics, sports, history"
            class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600
                   rounded-lg bg-white dark:bg-gray-700
                   text-gray-900 dark:text-gray-100 text-sm
                   placeholder-gray-400 dark:placeholder-gray-500
                   focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
                   mb-4"
          />
        {:else}
          <!-- Show tags as read-only for topic notes -->
          {#if noteWithSource.tags.length > 0}
            <div class="mb-4">
              <label
                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
              >
                Tags
              </label>
              <div class="flex flex-wrap gap-2">
                {#each noteWithSource.tags as tag}
                  <span
                    class="px-2 py-1 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded text-sm"
                  >
                    {tag}
                  </span>
                {/each}
              </div>
            </div>
          {/if}
        {/if}

        <!-- Date (read-only) -->
        <div class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          Created: {noteWithSource.date}
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-between">
          <div>
            {#if !showDeleteConfirm}
              <button
                type="button"
                onclick={() => (showDeleteConfirm = true)}
                class="px-4 py-2 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20
                       rounded-lg transition"
              >
                Delete Note
              </button>
            {:else}
              <div class="flex items-center gap-2">
                <span class="text-sm text-gray-600 dark:text-gray-400">
                  Are you sure?
                </span>
                <button
                  type="button"
                  onclick={handleDelete}
                  disabled={isDeleting}
                  class="px-3 py-1 bg-red-600 hover:bg-red-700 text-white rounded
                         disabled:opacity-50 disabled:cursor-not-allowed text-sm"
                >
                  {isDeleting ? "Deleting..." : "Yes, delete"}
                </button>
                <button
                  type="button"
                  onclick={() => (showDeleteConfirm = false)}
                  class="px-3 py-1 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600
                         text-gray-700 dark:text-gray-300 rounded text-sm"
                >
                  Cancel
                </button>
              </div>
            {/if}
          </div>

          <div class="flex gap-2">
            <button
              type="button"
              onclick={onClose}
              class="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600
                     text-gray-700 dark:text-gray-300 rounded-lg transition"
            >
              {isTopicNote ? "Close" : "Cancel"}
            </button>
            {#if !isTopicNote}
              <button
                type="submit"
                disabled={!content.trim() || !title.trim() || isSubmitting}
                class="px-6 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
                       disabled:bg-gray-400 disabled:cursor-not-allowed
                       text-white rounded-lg font-medium transition"
              >
                {isSubmitting ? "Saving..." : "Save Changes"}
              </button>
            {/if}
          </div>
        </div>
      </form>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    animation: fadeIn 0.15s ease-out;
  }

  .modal-content {
    animation: slideUp 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  textarea:focus,
  input:focus {
    outline: none;
  }
</style>
