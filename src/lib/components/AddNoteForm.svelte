<script lang="ts">
  import { addNote, currentCountry } from "$lib/stores/vault";
  import ImageUploader from "./ImageUploader.svelte";
  import MarkdownToolbar from "./MarkdownToolbar.svelte";

  export let onSuccess: () => void = () => {};

  let title = "";
  let content = "";
  let tagsInput = "";
  let isSubmitting = false;
  let error = "";
  let contentTextarea: HTMLTextAreaElement;

  async function handleSubmit() {
    if (!content.trim() || !title.trim() || !$currentCountry) return;

    isSubmitting = true;
    error = "";

    try {
      const tags = tagsInput
        .split(",")
        .map((t) => t.trim())
        .filter((t) => t.length > 0);

      await addNote($currentCountry.slug, title.trim(), content.trim(), tags);

      // Clear form
      title = "";
      content = "";
      tagsInput = "";

      onSuccess();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to add note";
      console.error("Error adding note:", err);
    } finally {
      isSubmitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Ctrl+Enter or Cmd+Enter to submit
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      handleSubmit();
    }
  }

  function handleImageInserted(event: CustomEvent<{ markdown: string }>) {
    const { markdown } = event.detail;

    // Insert at cursor position
    if (contentTextarea) {
      const start = contentTextarea.selectionStart;
      const end = contentTextarea.selectionEnd;
      const text = content;

      content =
        text.substring(0, start) + "\n" + markdown + "\n" + text.substring(end);

      // Move cursor after inserted image
      setTimeout(() => {
        contentTextarea.selectionStart = contentTextarea.selectionEnd =
          start + markdown.length + 2;
        contentTextarea.focus();
      }, 0);
    } else {
      // Append to end if no cursor position
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

      // If markdown has placeholder text and there's a selection, replace it
      if (selectedText && markdown.includes("text")) {
        insertText = markdown.replace("text", selectedText);
      }

      content = text.substring(0, start) + insertText + text.substring(end);

      setTimeout(() => {
        // Position cursor appropriately
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
</script>

<div
  class="add-note-form bg-white dark:bg-gray-800 rounded-lg border-2 border-mapanote-blue-200 dark:border-mapanote-blue-800 p-4 mb-6"
>
  <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
    Add Note
  </h3>

  {#if error}
    <div
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 mb-3"
    >
      <p class="text-red-800 dark:text-red-200 text-sm">{error}</p>
    </div>
  {/if}

  <form on:submit|preventDefault={handleSubmit}>
    <!-- Title input -->
    <input
      type="text"
      bind:value={title}
      placeholder="Note title..."
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
             rounded-lg bg-white dark:bg-gray-700
             text-gray-900 dark:text-gray-100
             placeholder-gray-400 dark:placeholder-gray-500
             focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
             mb-3"
    />

    <!-- Markdown Toolbar -->
    <MarkdownToolbar on:insertMarkdown={handleMarkdownInsert} />

    <!-- Note content -->
    <textarea
      bind:this={contentTextarea}
      bind:value={content}
      on:keydown={handleKeydown}
      placeholder="Write your note... (Ctrl+Enter to save)"
      rows="4"
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
             rounded-lg bg-white dark:bg-gray-700
             text-gray-900 dark:text-gray-100
             placeholder-gray-400 dark:placeholder-gray-500
             focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
             resize-y mb-3 font-mono text-sm"
    ></textarea>

    <!-- Image Uploader -->
    <div class="mb-3">
      <ImageUploader
        countrySlug={$currentCountry?.slug}
        on:imageInserted={handleImageInserted}
      />
      <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
        💡 Tip: Paste images directly with Ctrl+V or click to upload
      </p>
    </div>

    <!-- Tags input -->
    <input
      type="text"
      bind:value={tagsInput}
      placeholder="Tags (comma-separated): politics, sports, history"
      class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
             rounded-lg bg-white dark:bg-gray-700
             text-gray-900 dark:text-gray-100 text-sm
             placeholder-gray-400 dark:placeholder-gray-500
             focus:ring-2 focus:ring-mapanote-blue-500 focus:border-transparent
             mb-3"
    />

    <!-- Submit button -->
    <div class="flex justify-end">
      <button
        type="submit"
        disabled={!content.trim() || !title.trim() || isSubmitting}
        class="px-6 py-2 bg-mapanote-blue-600 hover:bg-mapanote-blue-700
               disabled:bg-gray-400 disabled:cursor-not-allowed
               text-white rounded-lg font-medium transition"
      >
        {#if isSubmitting}
          Adding...
        {:else}
          Add Note
        {/if}
      </button>
    </div>
  </form>

  <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
    ⌨️ Tip: Press <kbd
      class="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs"
      >Ctrl+Enter</kbd
    > to save quickly
  </p>
</div>

<style>
  textarea:focus,
  input:focus {
    outline: none;
  }

  kbd {
    font-family: monospace;
  }
</style>
