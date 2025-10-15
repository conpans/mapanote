<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { createEventDispatcher } from "svelte";
  import { openVault } from "$lib/stores/vault";

  const dispatch = createEventDispatcher<{ vaultOpened: string }>();

  type WizardStep = "welcome" | "create-name" | "create-location";

  let currentStep = $state<WizardStep>("welcome");
  let vaultName = $state("My World Notes");
  let vaultLocation = $state("");
  let createSubfolder = $state(true);
  let recentVaults = $state<string[]>([]);
  let isCreating = $state(false);
  let error = $state("");

  $effect(() => {
    loadRecentVaults();
    setDefaultLocation();
  });

  async function loadRecentVaults() {
    try {
      const stored = localStorage.getItem("mapanote_recent_vaults");
      if (stored) {
        recentVaults = JSON.parse(stored).slice(0, 5);
      }
    } catch (e) {
      console.error("Failed to load recent vaults:", e);
    }
  }

  async function setDefaultLocation() {
    const isWindows = navigator.platform.toLowerCase().includes("win");
    const home = isWindows ? "C:\\Users\\Public\\Documents" : "~/Documents";
    vaultLocation = `${home}/Mapanote`;
  }

  async function handleBrowseLocation() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select vault location",
    });

    if (selected) {
      vaultLocation = selected;
    }
  }

  async function handleOpenExisting() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select your Mapanote vault folder",
    });

    if (selected) {
      try {
        await openVault(selected);
        addToRecentVaults(selected);
        dispatch("vaultOpened", selected);
      } catch (e) {
        error = e instanceof Error ? e.message : "Failed to open vault";
      }
    }
  }

  async function handleCreateVault() {
    isCreating = true;
    error = "";

    try {
      const fullPath = createSubfolder
        ? `${vaultLocation}/${vaultName}`
        : vaultLocation;

      console.log("Creating minimal vault at:", fullPath);

      // Use the new create_minimal_vault command
      await invoke("create_minimal_vault", {
        destination: fullPath,
        vaultName: vaultName,
      });

      // Open the newly created vault
      await openVault(fullPath);

      // Add to recent vaults
      addToRecentVaults(fullPath);

      // Notify parent
      dispatch("vaultOpened", fullPath);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create vault";
      console.error("Vault creation failed:", e);
    } finally {
      isCreating = false;
    }
  }

  async function handleOpenRecent(path: string) {
    try {
      await openVault(path);
      addToRecentVaults(path);
      dispatch("vaultOpened", path);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to open vault";
    }
  }

  function addToRecentVaults(path: string) {
    const updated = [path, ...recentVaults.filter((v) => v !== path)].slice(
      0,
      5
    );
    recentVaults = updated;
    localStorage.setItem("mapanote_recent_vaults", JSON.stringify(updated));
  }

  function nextStep() {
    if (currentStep === "welcome") currentStep = "create-name";
    else if (currentStep === "create-name") currentStep = "create-location";
  }

  function prevStep() {
    if (currentStep === "create-location") currentStep = "create-name";
    else if (currentStep === "create-name") currentStep = "welcome";
  }
</script>

<div class="welcome-container">
  <div class="welcome-card">
    {#if currentStep === "welcome"}
      <!-- Welcome Screen -->
      <div class="welcome-content">
        <h1 class="welcome-title">Welcome to Mapanote</h1>
        <p class="welcome-subtitle">
          A <strong>vault</strong> is your folder of notes, maps, and settings. Create
          a new one or open an existing folder.
        </p>

        <div class="action-buttons">
          <button class="btn btn-primary" on:click={nextStep}>
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M12 5v14M5 12h14" />
            </svg>
            Create New Vault
          </button>

          <button class="btn btn-secondary" on:click={handleOpenExisting}>
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
              />
            </svg>
            Open Existing Vault
          </button>
        </div>

        {#if error}
          <div class="error-message">{error}</div>
        {/if}

        {#if recentVaults.length > 0}
          <div class="recent-vaults">
            <h3>Recent Vaults</h3>
            {#each recentVaults as vaultPath}
              <button
                class="recent-vault-item"
                on:click={() => handleOpenRecent(vaultPath)}
              >
                <span class="vault-path">{vaultPath}</span>
                <span class="vault-action">Open →</span>
              </button>
            {/each}
          </div>
        {/if}

        <div class="welcome-footer">
          <a href="#" class="footer-link">Documentation</a>
          <a href="#" class="footer-link">GitHub</a>
        </div>
      </div>
    {:else if currentStep === "create-name"}
      <!-- Step 1: Name your vault -->
      <div class="wizard-content">
        <div class="wizard-header">
          <h2>Name your vault</h2>
          <p>Choose a name that describes your notes</p>
        </div>

        <div class="form-group">
          <label for="vault-name">Vault Name</label>
          <input
            id="vault-name"
            type="text"
            bind:value={vaultName}
            placeholder="My World Notes"
            class="input-large"
            autofocus
          />
          <p class="help-text">You can change this later.</p>
        </div>

        {#if error}
          <div class="error-message">{error}</div>
        {/if}

        <div class="wizard-actions">
          <button class="btn btn-secondary" on:click={prevStep}>Back</button>
          <button
            class="btn btn-primary"
            on:click={nextStep}
            disabled={!vaultName.trim()}
          >
            Next
          </button>
        </div>
      </div>
    {:else if currentStep === "create-location"}
      <!-- Step 2: Choose location -->
      <div class="wizard-content">
        <div class="wizard-header">
          <h2>Choose where to save</h2>
          <p>
            Your vault will be lightweight - just a few files to start. Country
            data is loaded from the app.
          </p>
        </div>

        <div class="form-group">
          <label for="vault-location">Save Location</label>
          <div class="input-with-button">
            <input
              id="vault-location"
              type="text"
              bind:value={vaultLocation}
              class="input-large"
              readonly
            />
            <button class="btn btn-secondary" on:click={handleBrowseLocation}>
              Browse
            </button>
          </div>

          <label class="checkbox-label">
            <input type="checkbox" bind:checked={createSubfolder} />
            <span>Create inside a new folder named "{vaultName}"</span>
          </label>

          <p class="help-text">
            Full path: <strong
              >{createSubfolder
                ? `${vaultLocation}/${vaultName}`
                : vaultLocation}</strong
            >
          </p>
        </div>

        {#if error}
          <div class="error-message">{error}</div>
        {/if}

        <div class="wizard-actions">
          <button
            class="btn btn-secondary"
            on:click={prevStep}
            disabled={isCreating}
          >
            Back
          </button>
          <button
            class="btn btn-primary"
            on:click={handleCreateVault}
            disabled={isCreating}
          >
            {isCreating ? "Creating Vault..." : "Create Vault"}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .welcome-container {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    padding: 2rem;
  }

  .welcome-card {
    background: white;
    max-width: 600px;
    width: 100%;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    border-radius: 12px;
    overflow: hidden;
  }

  :global(.dark) .welcome-card {
    background: #1f2937;
  }

  .welcome-content,
  .wizard-content {
    padding: 3rem;
  }

  .welcome-title {
    font-size: 2.5rem;
    font-weight: 700;
    color: #111827;
    margin-bottom: 1rem;
    text-align: center;
  }

  :global(.dark) .welcome-title {
    color: #f9fafb;
  }

  .welcome-subtitle {
    font-size: 1.125rem;
    color: #4b5563;
    text-align: center;
    margin-bottom: 2.5rem;
    line-height: 1.6;
  }

  :global(.dark) .welcome-subtitle {
    color: #d1d5db;
  }

  .action-buttons {
    display: flex;
    gap: 1rem;
    margin-bottom: 2.5rem;
  }

  .btn {
    flex: 1;
    padding: 1rem 1.5rem;
    border: none;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    border-radius: 6px;
  }

  .btn-primary {
    background: #2563eb;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #1d4ed8;
  }

  .btn-primary:disabled {
    background: #9ca3af;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: white;
    border: 2px solid #d1d5db;
    color: #374151;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #f3f4f6;
    border-color: #9ca3af;
  }

  :global(.dark) .btn-secondary {
    background: #374151;
    border-color: #4b5563;
    color: #d1d5db;
  }

  .recent-vaults {
    margin-top: 2rem;
    padding-top: 2rem;
    border-top: 1px solid #e5e7eb;
  }

  :global(.dark) .recent-vaults {
    border-top-color: #374151;
  }

  .recent-vaults h3 {
    font-size: 0.875rem;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.75rem;
  }

  .recent-vault-item {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    margin-bottom: 0.5rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .recent-vault-item:hover {
    background: #f3f4f6;
    border-color: #2563eb;
  }

  :global(.dark) .recent-vault-item {
    background: #374151;
    border-color: #4b5563;
  }

  .vault-path {
    font-size: 0.875rem;
    color: #374151;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.dark) .vault-path {
    color: #d1d5db;
  }

  .vault-action {
    color: #2563eb;
    font-weight: 500;
    font-size: 0.875rem;
  }

  .welcome-footer {
    display: flex;
    justify-content: center;
    gap: 1.5rem;
    margin-top: 2rem;
  }

  .footer-link {
    font-size: 0.875rem;
    color: #6b7280;
    text-decoration: none;
  }

  .footer-link:hover {
    color: #2563eb;
  }

  /* Wizard Styles */
  .wizard-header {
    margin-bottom: 2rem;
    text-align: center;
  }

  .wizard-header h2 {
    font-size: 1.875rem;
    font-weight: 700;
    color: #111827;
    margin-bottom: 0.5rem;
  }

  :global(.dark) .wizard-header h2 {
    color: #f9fafb;
  }

  .wizard-header p {
    color: #6b7280;
    font-size: 0.9375rem;
  }

  :global(.dark) .wizard-header p {
    color: #9ca3af;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-group label {
    display: block;
    font-weight: 500;
    color: #374151;
    margin-bottom: 0.5rem;
    font-size: 0.9375rem;
  }

  :global(.dark) .form-group label {
    color: #d1d5db;
  }

  .input-large {
    width: 100%;
    padding: 0.75rem 1rem;
    font-size: 1rem;
    border: 2px solid #d1d5db;
    border-radius: 6px;
    background: white;
    color: #111827;
    transition: border-color 0.2s;
  }

  .input-large:focus {
    outline: none;
    border-color: #2563eb;
  }

  :global(.dark) .input-large {
    background: #374151;
    border-color: #4b5563;
    color: #f9fafb;
  }

  .input-with-button {
    display: flex;
    gap: 0.5rem;
  }

  .input-with-button input {
    flex: 1;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1rem;
    cursor: pointer;
    font-size: 0.9375rem;
    color: #374151;
  }

  :global(.dark) .checkbox-label {
    color: #d1d5db;
  }

  .help-text {
    font-size: 0.875rem;
    color: #6b7280;
    margin-top: 0.5rem;
    line-height: 1.5;
  }

  :global(.dark) .help-text {
    color: #9ca3af;
  }

  .wizard-actions {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    margin-top: 2rem;
  }

  .error-message {
    padding: 1rem;
    background: #fee2e2;
    border: 1px solid #ef4444;
    border-radius: 6px;
    color: #991b1b;
    margin-bottom: 1rem;
    font-size: 0.875rem;
  }

  :global(.dark) .error-message {
    background: #7f1d1d;
    border-color: #991b1b;
    color: #fecaca;
  }
</style>
