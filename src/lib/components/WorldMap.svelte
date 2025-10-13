<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    mapStats,
    mapStatsLoaded, // ← ADD THIS
    getActivityLevel,
    isFresh,
    loadMapStats,
  } from "$lib/stores/mapStats";
  import { isoToSlug, isoToName, hasCountry } from "$lib/data/isoMapping";
  import LoadingSkeleton from "./LoadingSkeleton.svelte";
  import { loadWorldSvg } from "$lib/stores/svgCache";

  let mapContainer: HTMLDivElement;
  let svgElement: SVGElement | null = null;
  let hoveredIso: string | null = null;
  let tooltipX = 0;
  let tooltipY = 0;
  let isLoading = true;
  let errorMessage = "";

  onMount(async () => {
    console.log("=== WorldMap mounted ===");

    try {
      // Only load stats if not already loaded
      if (!$mapStatsLoaded) {
        console.log("Loading map stats...");
        await loadMapStats();
      } else {
        console.log("Using cached map stats");
      }

      if (!mapContainer) {
        throw new Error("mapContainer ref not bound");
      }

      // Load SVG (cached after first load)
      console.log("Loading SVG...");
      const svgText = await loadWorldSvg();

      mapContainer.innerHTML = svgText;
      svgElement = mapContainer.querySelector("svg");

      if (svgElement) {
        svgElement.removeAttribute("width");
        svgElement.removeAttribute("height");
        svgElement.setAttribute("viewBox", "0 0 800 387");
        svgElement.style.width = "100%";
        svgElement.style.height = "auto";
        svgElement.style.display = "block";
      } else {
        throw new Error("SVG element not found in response");
      }

      // Enhance
      console.log("Enhancing map...");
      enhanceMap();
      console.log("Enhancement complete");

      isLoading = false;
    } catch (error) {
      console.error("Map initialization failed:", error);
      errorMessage = error instanceof Error ? error.message : String(error);
      isLoading = false;
    }
  });

  async function loadSVG() {
    // Use the module-level cache
    const svgText = await loadWorldSvg();

    mapContainer.innerHTML = svgText;

    svgElement = mapContainer.querySelector("svg");

    if (svgElement) {
      svgElement.removeAttribute("width");
      svgElement.removeAttribute("height");
      svgElement.setAttribute("viewBox", "0 0 800 387");
      svgElement.style.width = "100%";
      svgElement.style.height = "auto";
      svgElement.style.display = "block";
    } else {
      throw new Error("SVG element not found in response");
    }
  }

  function enhanceMap() {
    if (!svgElement) return;

    const countriesGroup = svgElement.querySelector(
      "#ne_110m_admin_0_countries"
    );
    if (!countriesGroup) return;

    const allPaths = countriesGroup.querySelectorAll("path[id]");
    console.log("Processing", allPaths.length, "paths");

    let enhanced = 0;

    allPaths.forEach((path) => {
      const isoCode = path.id;
      if (!isoCode) return;

      // Remove <a> wrapper first
      const parent = path.parentElement;
      if (parent && parent.tagName.toLowerCase() === "a") {
        parent.replaceWith(path);
      }

      // Check if this country exists in our metadata
      if (!hasCountry(isoCode)) {
        path.classList.add("activity-none");
        return;
      }

      const slug = isoToSlug[isoCode];
      const stats = slug ? $mapStats.get(slug) : null;

      // Apply activity color based on note count
      if (stats && stats.noteCount > 0) {
        const level = getActivityLevel(stats.noteCount);
        path.classList.add(`activity-${level}`);

        if (isFresh(stats.lastUpdated)) {
          path.classList.add("fresh");
        }
      } else {
        path.classList.add("activity-none");
      }

      // Make ALL countries with metadata clickable
      path.style.cursor = "pointer";
      path.setAttribute("role", "button");
      path.setAttribute("tabindex", "0");

      path.addEventListener("mouseenter", (e) => handleMouseEnter(e, isoCode));
      path.addEventListener("mousemove", handleMouseMove);
      path.addEventListener("mouseleave", handleMouseLeave);
      path.addEventListener("click", () => handleCountryClick(slug));

      enhanced++;
    });

    console.log(`Enhanced ${enhanced} countries (paths)`);

    // NEW: Enhance microstates
    enhanceMicrostates();
  }

  function enhanceMicrostates() {
    if (!svgElement) return;

    const microstatesGroup = svgElement.querySelector("#microstates");
    if (!microstatesGroup) {
      console.log("No microstates group found");
      return;
    }

    const circles = microstatesGroup.querySelectorAll("circle[data-code]");
    console.log("Processing", circles.length, "microstates");

    let enhanced = 0;

    circles.forEach((circle) => {
      const isoCode = circle.getAttribute("data-code");
      if (!isoCode) return;

      // Check if this microstate exists in our metadata
      if (!hasCountry(isoCode)) {
        return;
      }

      const slug = isoToSlug[isoCode];
      const stats = slug ? $mapStats.get(slug) : null;

      // Apply activity color
      if (stats && stats.noteCount > 0) {
        const level = getActivityLevel(stats.noteCount);
        circle.classList.add(`microstate-${level}`);
      } else {
        circle.classList.add("microstate-none");
      }

      // Make clickable
      circle.style.cursor = "pointer";
      circle.setAttribute("role", "button");
      circle.setAttribute("tabindex", "0");

      circle.addEventListener("mouseenter", (e) =>
        handleMouseEnter(e, isoCode)
      );
      circle.addEventListener("mousemove", handleMouseMove);
      circle.addEventListener("mouseleave", handleMouseLeave);
      circle.addEventListener("click", () => handleCountryClick(slug));

      enhanced++;
    });

    console.log(`Enhanced ${enhanced} microstates`);
  }

  function handleMouseEnter(e: Event, isoCode: string) {
    hoveredIso = isoCode;
    if (e instanceof MouseEvent) {
      updateTooltipPosition(e);
    }
  }

  function handleMouseMove(e: Event) {
    if (hoveredIso && e instanceof MouseEvent) {
      updateTooltipPosition(e);
    }
  }

  function handleMouseLeave() {
    hoveredIso = null;
  }

  function updateTooltipPosition(e: MouseEvent) {
    tooltipX = e.clientX + 15;
    tooltipY = e.clientY + 15;
  }

  function handleCountryClick(slug: string) {
    goto(`/country/${slug}`);
  }

  $: tooltipSlug = hoveredIso ? isoToSlug[hoveredIso] : null;
  $: tooltipName = hoveredIso ? isoToName[hoveredIso] : null;
  $: tooltipStats = tooltipSlug ? $mapStats.get(tooltipSlug) : null;
</script>

<div class="world-map-wrapper">
  <!-- Always render container so bind:this works -->
  <div class="map-container" bind:this={mapContainer}>
    <!-- SVG will be inserted here -->
  </div>

  <!-- Loading overlay -->
  {#if isLoading}
    <div class="loading-overlay">
      <LoadingSkeleton type="map" />
    </div>
  {/if}

  <!-- Error overlay -->
  {#if errorMessage}
    <div class="error-overlay">
      <div
        class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6 max-w-md"
      >
        <h3 class="text-red-800 dark:text-red-200 font-semibold mb-2">
          Failed to load map
        </h3>
        <p class="text-red-700 dark:text-red-300 text-sm">
          {errorMessage}
        </p>
      </div>
    </div>
  {/if}

  <!-- Tooltip -->
  {#if hoveredIso && tooltipName && !isLoading}
    <div class="map-tooltip" style="left: {tooltipX}px; top: {tooltipY}px;">
      <div class="font-semibold text-gray-900 dark:text-gray-100">
        {tooltipName}
      </div>
      {#if tooltipStats}
        <div class="mt-2 text-sm">
          <div class="text-gray-700 dark:text-gray-300">
            📝 {tooltipStats.noteCount}
            {tooltipStats.noteCount === 1 ? "note" : "notes"}
          </div>
          {#if tooltipStats.lastUpdated}
            <div class="text-gray-500 dark:text-gray-400 text-xs mt-1">
              Updated: {tooltipStats.lastUpdated}
            </div>
          {/if}
          {#if isFresh(tooltipStats.lastUpdated)}
            <div class="text-green-600 dark:text-green-400 text-xs mt-1">
              🔥 Active
            </div>
          {/if}
        </div>
        <div
          class="text-xs text-mapanote-blue-600 dark:text-mapanote-blue-400 mt-2"
        >
          Click to open →
        </div>
      {:else}
        <div class="text-sm text-gray-500 dark:text-gray-400 mt-2">
          No notes yet
        </div>
        <div
          class="text-xs text-mapanote-blue-600 dark:text-mapanote-blue-400 mt-2"
        >
          Click to start →
        </div>
      {/if}
    </div>
  {/if}

  <!-- Legend -->
  {#if !isLoading && !errorMessage}
    <div class="map-legend">
      <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
        Activity
      </h3>
      <div class="space-y-1">
        <div class="legend-item">
          <div class="legend-swatch activity-none"></div>
          <span class="text-xs">No notes</span>
        </div>
        <div class="legend-item">
          <div class="legend-swatch activity-low"></div>
          <span class="text-xs">1-5</span>
        </div>
        <div class="legend-item">
          <div class="legend-swatch activity-medium"></div>
          <span class="text-xs">6-20</span>
        </div>
        <div class="legend-item">
          <div class="legend-swatch activity-high"></div>
          <span class="text-xs">21-50</span>
        </div>
        <div class="legend-item">
          <div class="legend-swatch activity-very-high"></div>
          <span class="text-xs">50+</span>
        </div>
        <div
          class="legend-item mt-2 pt-2 border-t border-gray-200 dark:border-gray-700"
        >
          <div class="legend-swatch fresh-indicator"></div>
          <span class="text-xs">🔥 Recent</span>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .world-map-wrapper {
    position: relative;
    width: 100%;
    min-height: 600px;
  }

  .map-container {
    width: 100%;
    min-height: 600px;
    background: white;
    border-radius: 8px;
    border: 1px solid #e5e7eb;
    padding: 1rem;
  }

  :global(.dark) .map-container {
    background: #1f2937;
    border-color: #374151;
  }

  .loading-overlay,
  .error-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.9);
    border-radius: 8px;
    z-index: 100;
  }

  :global(.dark) .loading-overlay,
  :global(.dark) .error-overlay {
    background: rgba(31, 41, 55, 0.9);
  }

  /* SVG styling */
  :global(.map-container svg) {
    display: block;
  }

  :global(.map-container path) {
    vector-effect: non-scaling-stroke;
    transition:
      opacity 0.12s ease,
      stroke-width 0.12s ease;
  }

  /* Activity colors */
  :global(.activity-none) {
    fill: #f3f4f6 !important;
    stroke: #d1d5db !important;
    stroke-width: 0.5;
  }

  :global(.dark .activity-none) {
    fill: #374151 !important;
    stroke: #4b5563 !important;
  }

  :global(.activity-low) {
    fill: #dbeafe !important;
    stroke: #93c5fd !important;
    stroke-width: 0.5;
  }

  :global(.dark .activity-low) {
    fill: #1e3a8a !important;
    stroke: #3b82f6 !important;
  }

  :global(.activity-medium) {
    fill: #93c5fd !important;
    stroke: #60a5fa !important;
    stroke-width: 0.5;
  }

  :global(.dark .activity-medium) {
    fill: #1e40af !important;
    stroke: #3b82f6 !important;
  }

  :global(.activity-high) {
    fill: #60a5fa !important;
    stroke: #3b82f6 !important;
    stroke-width: 0.5;
  }

  :global(.dark .activity-high) {
    fill: #1d4ed8 !important;
    stroke: #2563eb !important;
  }

  :global(.activity-very-high) {
    fill: #3b82f6 !important;
    stroke: #2563eb !important;
    stroke-width: 0.5;
  }

  :global(.dark .activity-very-high) {
    fill: #1e40af !important;
    stroke: #1d4ed8 !important;
  }

  :global(.fresh) {
    stroke: #10b981 !important;
    stroke-width: 2 !important;
  }

  :global(path[role="button"]:hover) {
    opacity: 0.8;
    stroke-width: 2 !important;
  }

  :global(path[role="button"]:focus) {
    outline: 2px solid #3b82f6;
    outline-offset: 2px;
  }

  /* Tooltip */
  .map-tooltip {
    position: fixed;
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 12px;
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    pointer-events: none;
    z-index: 1000;
    min-width: 180px;
    max-width: 250px;
  }

  :global(.dark) .map-tooltip {
    background: #1f2937;
    border-color: #374151;
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.3);
  }

  /* Legend */
  .map-legend {
    position: absolute;
    bottom: 20px;
    right: 20px;
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 12px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    z-index: 10;
  }

  :global(.dark) .map-legend {
    background: #1f2937;
    border-color: #374151;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #6b7280;
  }

  :global(.dark) .legend-item {
    color: #9ca3af;
  }

  /* NEW: Microstate styles */
  :global(#microstates circle) {
    vector-effect: non-scaling-stroke;
    transition: all 0.2s ease;
  }

  /* Default microstate (no notes) */
  :global(.microstate-none) {
    fill: #9ca3af !important;
    stroke: #6b7280 !important;
    stroke-width: 0.8;
    r: 1.2;
  }

  :global(.dark .microstate-none) {
    fill: #60a5fa !important;
    stroke: #93c5fd !important;
    stroke-width: 1;
    r: 1.5;
    filter: drop-shadow(0 0 3px rgba(96, 165, 250, 0.6));
  }

  /* Microstate with notes */
  :global(.microstate-low) {
    fill: #93c5fd !important;
    stroke: #60a5fa !important;
    r: 1.5;
  }

  :global(.dark .microstate-low) {
    fill: #60a5fa !important;
    stroke: #3b82f6 !important;
    r: 1.8;
    filter: drop-shadow(0 0 4px rgba(59, 130, 246, 0.7));
  }

  :global(.microstate-medium),
  :global(.microstate-high),
  :global(.microstate-very-high) {
    fill: #3b82f6 !important;
    stroke: #2563eb !important;
    r: 1.8;
  }

  :global(.dark .microstate-medium),
  :global(.dark .microstate-high),
  :global(.dark .microstate-very-high) {
    fill: #3b82f6 !important;
    stroke: #2563eb !important;
    r: 2;
    filter: drop-shadow(0 0 5px rgba(37, 99, 235, 0.8));
  }

  /* Hover state for microstates */
  :global(#microstates circle[role="button"]:hover) {
    r: 2.5 !important;
    stroke-width: 1.5 !important;
    filter: drop-shadow(0 0 8px currentColor) !important;
  }

  .legend-swatch {
    width: 20px;
    height: 14px;
    border-radius: 2px;
    border: 1px solid #d1d5db;
    flex-shrink: 0;
  }

  :global(.dark) .legend-swatch {
    border-color: #4b5563;
  }

  .legend-swatch.activity-none {
    background: #f3f4f6;
  }
  .legend-swatch.activity-low {
    background: #dbeafe;
  }
  .legend-swatch.activity-medium {
    background: #93c5fd;
  }
  .legend-swatch.activity-high {
    background: #60a5fa;
  }
  .legend-swatch.activity-very-high {
    background: #3b82f6;
  }
  .legend-swatch.fresh-indicator {
    background: #10b981;
    border-color: #10b981;
  }
</style>
