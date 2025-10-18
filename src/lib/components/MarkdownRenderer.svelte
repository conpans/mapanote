<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface Props {
    content: string;
    countrySlug?: string;
    topicId?: string;
  }

  let { content, countrySlug, topicId }: Props = $props();

  let renderedHTML = $state("");

  onMount(async () => {
    await parseMarkdown();
  });

  $effect(() => {
    parseMarkdown();
  });

  async function parseMarkdown() {
    let html = content;

    // Process images first (async operation)
    const imageRegex = /!\[([^\]]*)\]\(([^)]+)\)/g;
    const imagePromises: Promise<void>[] = [];
    const imageReplacements: { original: string; replacement: string }[] = [];

    let match;
    while ((match = imageRegex.exec(content)) !== null) {
      const altText = match[1];
      const imagePath = match[2];
      const originalMatch = match[0];

      if (imagePath.startsWith("assets/")) {
        const promise = (async () => {
          try {
            const filename = imagePath.replace("assets/", "");
            let base64Data: string;

            if (countrySlug) {
              base64Data = await invoke<string>("get_note_image", {
                countrySlug,
                imageFilename: filename,
              });
            } else if (topicId) {
              base64Data = await invoke<string>("get_topic_image", {
                topicId,
                imageFilename: filename,
              });
            } else {
              console.error("No country or topic specified for image");
              return;
            }

            const extension = filename.split(".").pop()?.toLowerCase();
            const mimeType =
              extension === "jpg" || extension === "jpeg"
                ? "image/jpeg"
                : extension === "gif"
                  ? "image/gif"
                  : "image/png";

            const imgTag = `<div class="my-4"><img src="data:${mimeType};base64,${base64Data}" alt="${escapeHtml(altText)}" class="max-w-full h-auto rounded-lg border border-gray-200 dark:border-gray-700" loading="lazy" />${altText ? `<p class="text-sm text-gray-500 dark:text-gray-400 mt-1 italic">${escapeHtml(altText)}</p>` : ""}</div>`;

            imageReplacements.push({
              original: originalMatch,
              replacement: imgTag,
            });
          } catch (error) {
            console.error("Failed to load image:", error);
            imageReplacements.push({
              original: originalMatch,
              replacement: `<span class="text-red-500">[Image not found: ${escapeHtml(imagePath)}]</span>`,
            });
          }
        })();

        imagePromises.push(promise);
      }
    }

    // Wait for all images to load
    await Promise.all(imagePromises);

    // Replace images
    imageReplacements.forEach(({ original, replacement }) => {
      html = html.replace(original, replacement);
    });

    // Bold: **text** or __text__
    html = html.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
    html = html.replace(/__([^_]+)__/g, "<strong>$1</strong>");

    // Italic: *text* or _text_
    html = html.replace(/\*([^*]+)\*/g, "<em>$1</em>");
    html = html.replace(/_([^_]+)_/g, "<em>$1</em>");

    // Highlight: ==text==
    html = html.replace(
      /==([^=]+)==/g,
      '<mark class="bg-yellow-200 dark:bg-yellow-800 px-1 rounded">$1</mark>'
    );

    // Inline code: `text`
    html = html.replace(
      /`([^`]+)`/g,
      '<code class="bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded text-sm font-mono text-red-600 dark:text-red-400">$1</code>'
    );

    // Links: [text](url)
    html = html.replace(
      /\[([^\]]+)\]\(([^)]+)\)/g,
      '<a href="$2" target="_blank" rel="noopener noreferrer" class="text-mapanote-blue-600 dark:text-mapanote-blue-400 hover:underline">$1</a>'
    );

    // Headings: # text, ## text, ### text
    html = html.replace(
      /^### (.+)$/gm,
      '<h3 class="text-lg font-bold text-gray-900 dark:text-gray-100 mt-4 mb-2">$1</h3>'
    );
    html = html.replace(
      /^## (.+)$/gm,
      '<h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 mt-4 mb-2">$1</h2>'
    );
    html = html.replace(
      /^# (.+)$/gm,
      '<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mt-4 mb-2">$1</h1>'
    );

    // Bullet lists: - item or * item
    html = html.replace(/^[*-] (.+)$/gm, '<li class="ml-4">$1</li>');
    html = html.replace(
      /(<li class="ml-4">.*<\/li>)/s,
      '<ul class="list-disc list-inside text-gray-700 dark:text-gray-300 my-2">$1</ul>'
    );

    // Numbered lists: 1. item
    html = html.replace(/^\d+\. (.+)$/gm, '<li class="ml-4">$1</li>');
    html = html.replace(
      /(<li class="ml-4">.*<\/li>)/s,
      '<ol class="list-decimal list-inside text-gray-700 dark:text-gray-300 my-2">$1</ol>'
    );

    // Line breaks: preserve newlines
    html = html.replace(
      /\n\n/g,
      '</p><p class="text-gray-700 dark:text-gray-300 leading-relaxed my-2">'
    );
    html = html.replace(/\n/g, "<br>");

    // Wrap in paragraph if not already wrapped
    if (!html.startsWith("<")) {
      html = `<p class="text-gray-700 dark:text-gray-300 leading-relaxed">${html}</p>`;
    }

    renderedHTML = html;
  }

  function escapeHtml(text: string): string {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }
</script>

<div class="markdown-content">
  {@html renderedHTML}
</div>

<style>
  .markdown-content {
    max-width: 100%;
  }

  .markdown-content :global(img) {
    max-height: 500px;
    object-fit: contain;
  }

  .markdown-content :global(strong) {
    font-weight: 600;
    color: inherit;
  }

  .markdown-content :global(em) {
    font-style: italic;
  }

  .markdown-content :global(mark) {
    padding: 0.1em 0.3em;
  }
</style>
