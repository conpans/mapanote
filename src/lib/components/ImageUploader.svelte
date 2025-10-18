<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher } from "svelte";

  interface Props {
    countrySlug?: string;
    topicId?: string;
    onImageInserted?: (markdown: string) => void;
  }

  let { countrySlug, topicId, onImageInserted }: Props = $props();

  const dispatch = createEventDispatcher();

  let isUploading = $state(false);
  let uploadError = $state<string | null>(null);
  let fileInput: HTMLInputElement;

  async function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];

    if (!file) return;

    // Check if it's an image
    if (!file.type.startsWith("image/")) {
      uploadError = "Please select an image file";
      return;
    }

    await uploadImage(file);
  }

  async function handlePaste(event: ClipboardEvent) {
    const items = event.clipboardData?.items;
    if (!items) return;

    for (const item of items) {
      if (item.type.startsWith("image/")) {
        event.preventDefault();
        const file = item.getAsFile();
        if (file) {
          await uploadImage(file);
        }
        break;
      }
    }
  }

  async function uploadImage(file: File) {
    isUploading = true;
    uploadError = null;

    try {
      // Convert file to base64
      const base64Data = await fileToBase64(file);

      // Remove the data URL prefix (e.g., "data:image/png;base64,")
      const base64String = base64Data.split(",")[1];

      // Generate filename
      const extension = file.name.split(".").pop() || "png";
      const filename = `${Date.now()}.${extension}`;

      // Save to vault
      let imagePath: string;
      if (countrySlug) {
        imagePath = await invoke<string>("save_note_image", {
          countrySlug,
          imageData: base64String,
          filename,
        });
      } else if (topicId) {
        imagePath = await invoke<string>("save_topic_image", {
          topicId,
          imageData: base64String,
          filename,
        });
      } else {
        throw new Error("No country or topic specified");
      }

      // Create markdown syntax
      const markdown = `![Image](${imagePath})`;

      // Dispatch event or call callback
      if (onImageInserted) {
        onImageInserted(markdown);
      }
      dispatch("imageInserted", { markdown, path: imagePath });

      // Reset file input
      if (fileInput) {
        fileInput.value = "";
      }
    } catch (error) {
      console.error("Failed to upload image:", error);
      uploadError = error instanceof Error ? error.message : String(error);
    } finally {
      isUploading = false;
    }
  }

  function fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result as string);
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }

  function openFilePicker() {
    fileInput?.click();
  }
</script>

<div class="image-uploader">
  <!-- Hidden file input -->
  <input
    bind:this={fileInput}
    type="file"
    accept="image/*"
    onchange={handleFileSelect}
    class="hidden"
  />

  <!-- Upload button -->
  <button
    type="button"
    onclick={openFilePicker}
    disabled={isUploading}
    class="inline-flex items-center gap-2 px-3 py-1.5 text-sm
           bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600
           text-gray-700 dark:text-gray-300 rounded transition
           disabled:opacity-50 disabled:cursor-not-allowed"
    title="Upload image or paste from clipboard (Ctrl+V)"
  >
    {#if isUploading}
      <svg
        class="w-4 h-4 animate-spin"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
        />
      </svg>
      Uploading...
    {:else}
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
          d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
        />
      </svg>
      Add Image
    {/if}
  </button>

  {#if uploadError}
    <p class="text-xs text-red-600 dark:text-red-400 mt-1">{uploadError}</p>
  {/if}
</div>

<svelte:window onpaste={handlePaste} />

<style>
  .hidden {
    display: none;
  }
</style>
