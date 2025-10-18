<script lang="ts">
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  function insertMarkdown(markdown: string) {
    dispatch("insertMarkdown", { markdown });
  }

  const buttons = [
    {
      label: "Bold",
      icon: "B",
      markdown: "**text**",
      title: "Bold (Ctrl+B)",
    },
    {
      label: "Italic",
      icon: "I",
      markdown: "*text*",
      title: "Italic (Ctrl+I)",
    },
    {
      label: "Highlight",
      icon: "H",
      markdown: "==text==",
      title: "Highlight",
    },
    {
      label: "Code",
      icon: "</>",
      markdown: "`text`",
      title: "Inline code",
    },
    {
      label: "Link",
      icon: "🔗",
      markdown: "[text](url)",
      title: "Insert link",
    },
    {
      label: "List",
      icon: "•",
      markdown: "- text",
      title: "Bullet list",
    },
    {
      label: "Heading",
      icon: "H1",
      markdown: "# text",
      title: "Heading",
    },
  ];
</script>

<div
  class="markdown-toolbar flex items-center gap-1 p-2 bg-gray-50 dark:bg-gray-700/50 rounded-lg mb-3 flex-wrap"
>
  <span class="text-xs text-gray-500 dark:text-gray-400 mr-2">Format:</span>

  {#each buttons as button}
    <button
      type="button"
      onclick={() => insertMarkdown(button.markdown)}
      class="px-2.5 py-1.5 text-sm font-medium rounded hover:bg-gray-200 dark:hover:bg-gray-600
             text-gray-700 dark:text-gray-300 transition
             {button.label === 'Bold' ? 'font-bold' : ''}
             {button.label === 'Italic' ? 'italic' : ''}"
      title={button.title}
    >
      {button.icon}
    </button>
  {/each}

  <div class="ml-auto text-xs text-gray-400 dark:text-gray-500">
    Select text to format
  </div>
</div>

<style>
  .markdown-toolbar button:active {
    transform: scale(0.95);
  }
</style>
