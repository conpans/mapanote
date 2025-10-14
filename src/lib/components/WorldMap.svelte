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
  import {
    isoToSlug,
    isoToName,
    hasCountry,
    getCountryByIso2,
  } from "$lib/data/isoMapping";
  import { truncate } from "$lib/data/countryHelpers";
  import LoadingSkeleton from "./LoadingSkeleton.svelte";
  import { loadWorldSvg } from "$lib/stores/svgCache";

  let mapContainer: HTMLDivElement;
  let svgElement: SVGElement | null = null;
  let hoveredIso = $state<string | null>(null);
  let tooltipX = $state(0);
  let tooltipY = $state(0);
  let isLoading = $state(true);
  let errorMessage = $state("");
  let isTooltipHovered = $state(false);
  let hideTooltipTimeout: number | null = null;
  let showTooltipTimeout: number | null = null; // ← NEW: Delay before showing
  let shouldShowTooltip = $state(false); // ← NEW: Controls visibility with delay
  let isTooltipFadingOut = $state(false);

  // Zoom/Pan state
  let transform = $state({ scale: 1, x: 0, y: 0 });
  let isPanning = $state(false);
  let panStart = { x: 0, y: 0, transformX: 0, transformY: 0 };
  let viewportGroup: SVGGElement | null = null;

  // Derived values
  let tooltipSlug = $derived(hoveredIso ? isoToSlug[hoveredIso] : null);
  let tooltipName = $derived(hoveredIso ? isoToName[hoveredIso] : null);
  let tooltipStats = $derived(tooltipSlug ? $mapStats.get(tooltipSlug) : null);
  let tooltipCountry = $derived(
    hoveredIso ? getCountryByIso2(hoveredIso) : null
  );
  let tooltipPosition = $state<"top" | "bottom">("bottom");

  onMount(async () => {
    console.log("=== WorldMap mounted ===");

    try {
      if (!$mapStatsLoaded) {
        console.log("Loading map stats...");
        await loadMapStats();
      } else {
        console.log("Using cached map stats");
      }

      if (!mapContainer) {
        throw new Error("mapContainer ref not bound");
      }

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

        // Wrap content in viewport group
        wrapInViewportGroup();
      } else {
        throw new Error("SVG element not found in response");
      }

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

      // ← NEW: Wrap content in viewport group for transforms
      wrapInViewportGroup();
    } else {
      throw new Error("SVG element not found in response");
    }
  }

  // ← NEW: Wrap SVG content for zoom/pan
  function wrapInViewportGroup() {
    if (!svgElement) return;

    let viewport = svgElement.querySelector(
      "#viewport-transform"
    ) as SVGGElement;

    if (!viewport) {
      viewport = document.createElementNS("http://www.w3.org/2000/svg", "g");
      viewport.id = "viewport-transform";

      // Move all children into viewport group
      const children = Array.from(svgElement.children);
      children.forEach((child) => {
        viewport.appendChild(child);
      });

      svgElement.appendChild(viewport);
    }

    viewportGroup = viewport;
    console.log("Viewport group created:", viewportGroup);
  }

  // ← NEW: Zoom handler (mouse wheel)
  function handleWheel(e: WheelEvent) {
    e.preventDefault();

    const delta = e.deltaY < 0 ? 1.15 : 1 / 1.15;
    const newScale = clamp(transform.scale * delta, 1, 10);

    if (svgElement && mapContainer) {
      const rect = mapContainer.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      // Convert to SVG coordinates
      const svgX = (mouseX / rect.width) * 800;
      const svgY = (mouseY / rect.height) * 387;

      const scaleChange = newScale / transform.scale;

      transform = {
        scale: newScale,
        x: svgX - (svgX - transform.x) * scaleChange,
        y: svgY - (svgY - transform.y) * scaleChange,
      };

      applyTransform();
    }
  }

  // Pan handlers
  function handlePanStart(e: MouseEvent) {
    if (e.button !== 0) return;

    e.preventDefault();
    isPanning = true;

    panStart = {
      x: e.clientX,
      y: e.clientY,
      transformX: transform.x,
      transformY: transform.y,
    };
  }

  function handlePanMove(e: MouseEvent) {
    if (!isPanning || !svgElement || !mapContainer) return;

    const rect = mapContainer.getBoundingClientRect();
    const dx = e.clientX - panStart.x;
    const dy = e.clientY - panStart.y;

    // Convert pixels to SVG units
    const svgDx = (dx / rect.width) * 800;
    const svgDy = (dy / rect.height) * 387;

    transform = {
      ...transform,
      x: panStart.transformX + svgDx,
      y: panStart.transformY + svgDy,
    };

    applyTransform();
  }

  function handlePanEnd() {
    isPanning = false;
  }

  // ← NEW: Pan start (mouse down)
  function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return; // Only left click

    isPanning = true;
    panStart = {
      x: e.clientX,
      y: e.clientY,
      transformX: transform.x,
      transformY: transform.y,
    };

    if (mapContainer) {
      mapContainer.style.cursor = "grabbing";
    }
  }

  // ← NEW: Pan move (mouse move while dragging)
  function handleMouseMoveGlobal(e: MouseEvent) {
    if (!isPanning) return;

    const dx = e.clientX - panStart.x;
    const dy = e.clientY - panStart.y;

    // Convert screen pixels to SVG coordinates
    const svgDx = (dx / (svgElement?.getBoundingClientRect().width || 1)) * 800;
    const svgDy =
      (dy / (svgElement?.getBoundingClientRect().height || 1)) * 387;

    transform = {
      ...transform,
      x: panStart.transformX + svgDx,
      y: panStart.transformY + svgDy,
    };

    applyTransform();
  }

  // ← NEW: Pan end (mouse up)
  function handleMouseUp() {
    isPanning = false;

    if (mapContainer) {
      mapContainer.style.cursor = transform.scale > 1 ? "grab" : "default";
    }
  }

  // ← NEW: Apply transform to viewport group
  function applyTransform() {
    if (viewportGroup) {
      viewportGroup.setAttribute(
        "transform",
        `translate(${transform.x}, ${transform.y}) scale(${transform.scale})`
      );
    }
  }

  function resetView() {
    transform = { scale: 1, x: 0, y: 0 };
    applyTransform();
  }

  function clamp(value: number, min: number, max: number): number {
    return Math.min(Math.max(value, min), max);
  }

  function enhanceMap() {
    if (!svgElement) return;

    const countriesGroup = svgElement.querySelector(
      "#ne_110m_admin_0_countries"
    );
    if (!countriesGroup) {
      console.error("Countries group not found");
      return;
    }

    const allPaths = countriesGroup.querySelectorAll("path[id]");
    console.log("Processing", allPaths.length, "paths");

    let enhanced = 0;

    allPaths.forEach((path) => {
      const isoCode = path.id;
      if (!isoCode) return;

      const parent = path.parentElement;
      if (parent && parent.tagName.toLowerCase() === "a") {
        parent.replaceWith(path);
      }

      if (!hasCountry(isoCode)) {
        path.classList.add("activity-none");
        return;
      }

      const slug = isoToSlug[isoCode];
      const stats = slug ? $mapStats.get(slug) : null;

      if (stats && stats.noteCount > 0) {
        const level = getActivityLevel(stats.noteCount);
        path.classList.add(`activity-${level}`);

        if (isFresh(stats.lastUpdated)) {
          path.classList.add("fresh");
        }
      } else {
        path.classList.add("activity-none");
      }

      path.style.cursor = "pointer";
      path.setAttribute("role", "button");
      path.setAttribute("tabindex", "0");

      path.addEventListener("mouseenter", (e) => handleMouseEnter(e, isoCode));
      path.addEventListener("mousemove", handleMouseMove);
      path.addEventListener("mouseleave", handleMouseLeave);
      path.addEventListener("click", (e) => {
        e.stopPropagation();
        handleCountryClick(slug);
      });

      enhanced++;
    });

    console.log(`Enhanced ${enhanced} countries (paths)`);
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

      if (!hasCountry(isoCode)) return;

      const slug = isoToSlug[isoCode];
      const stats = slug ? $mapStats.get(slug) : null;

      if (stats && stats.noteCount > 0) {
        const level = getActivityLevel(stats.noteCount);
        circle.classList.add(`microstate-${level}`);
      } else {
        circle.classList.add("microstate-none");
      }

      circle.style.cursor = "pointer";
      circle.setAttribute("role", "button");
      circle.setAttribute("tabindex", "0");

      circle.addEventListener("mouseenter", (e) =>
        handleMouseEnter(e, isoCode)
      );
      circle.addEventListener("mousemove", handleMouseMove);
      circle.addEventListener("mouseleave", handleMouseLeave);
      circle.addEventListener("click", (e) => {
        e.stopPropagation();
        handleCountryClick(slug);
      });

      enhanced++;
    });

    console.log(`Enhanced ${enhanced} microstates`);
  }

  function handleMouseEnter(e: Event, isoCode: string) {
    // Clear any pending hide timeout
    if (hideTooltipTimeout) {
      clearTimeout(hideTooltipTimeout);
      hideTooltipTimeout = null;
    }

    hoveredIso = isoCode;

    if (e instanceof MouseEvent) {
      // Set position ONCE when entering (stationary)
      setTooltipPosition(e);

      // Show tooltip after short delay
      showTooltipTimeout = setTimeout(() => {
        shouldShowTooltip = true;
      }, 200) as unknown as number; // 300ms delay
    }
  }

  function handleMouseMove(e: Event) {
    // Don't update position - keep it stationary!
    // This function can be removed or kept empty for future use
  }

  function handleMouseLeave() {
    if (showTooltipTimeout) {
      clearTimeout(showTooltipTimeout);
      showTooltipTimeout = null;
    }

    hideTooltipTimeout = setTimeout(() => {
      if (!isTooltipHovered) {
        // Start fade-out animation
        isTooltipFadingOut = true;

        // Wait for animation to complete before hiding
        setTimeout(() => {
          hoveredIso = null;
          shouldShowTooltip = false;
          isTooltipFadingOut = false;
        }, 150); // Match animation duration
      }
    }, 150) as unknown as number;
  }

  function setTooltipPosition(e: MouseEvent) {
    const offset = 20;
    const tooltipWidth = 360;
    const tooltipHeight = 320;
    const viewportHeight = window.innerHeight;

    // Center horizontally on cursor
    let x = e.clientX - tooltipWidth / 2;

    // Keep on screen horizontally
    if (x < 10) x = 10;
    if (x + tooltipWidth > window.innerWidth - 10) {
      x = window.innerWidth - tooltipWidth - 10;
    }

    // Position above or below cursor
    let y: number;
    if (e.clientY + offset + tooltipHeight < viewportHeight) {
      // Show below cursor
      y = e.clientY + offset;
      tooltipPosition = "bottom";
    } else {
      // Show above cursor
      y = e.clientY - tooltipHeight - offset;
      tooltipPosition = "top";
    }

    tooltipX = x;
    tooltipY = y;
  }

  function updateTooltipPosition(e: MouseEvent) {
    const offset = 20;
    const tooltipWidth = 340;
    const viewportWidth = window.innerWidth;

    if (e.clientX + offset + tooltipWidth < viewportWidth) {
      tooltipX = e.clientX + offset;
    } else {
      tooltipX = e.clientX - tooltipWidth - offset;
    }

    tooltipY = e.clientY + offset;
  }

  function handleTooltipMouseEnter() {
    isTooltipHovered = true;
    shouldShowTooltip = true; // Keep it visible

    if (hideTooltipTimeout) {
      clearTimeout(hideTooltipTimeout);
      hideTooltipTimeout = null;
    }
  }

  function handleTooltipMouseLeave() {
    isTooltipHovered = false;

    hideTooltipTimeout = setTimeout(() => {
      isTooltipFadingOut = true;

      setTimeout(() => {
        hoveredIso = null;
        shouldShowTooltip = false;
        isTooltipFadingOut = false;
      }, 150);
    }, 150) as unknown as number;
  }

  function handleCountryClick(slug: string) {
    goto(`/country/${slug}`);
  }
</script>

<svelte:window on:mousemove={handlePanMove} on:mouseup={handlePanEnd} />

<div class="world-map-wrapper">
  <div
    class="map-container"
    bind:this={mapContainer}
    on:wheel={handleWheel}
    on:mousedown={handlePanStart}
    style:cursor={isPanning
      ? "grabbing"
      : transform.scale > 1
        ? "grab"
        : "default"}
  >
    <!-- SVG will be inserted here -->
  </div>

  <!-- Reset button (shows when zoomed) -->
  {#if transform.scale > 1}
    <button
      class="reset-button"
      on:click={resetView}
      title="Reset view (zoom to 1x)"
    >
      <svg
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
        <path d="M3 3v5h5" />
      </svg>
      Reset View
    </button>
  {/if}

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

  <!-- Enhanced Tooltip with Flag -->
  <!-- Stationary Tooltip (only show when shouldShowTooltip is true) -->
  {#if hoveredIso && tooltipName && !isLoading && shouldShowTooltip}
    <div
      class="map-tooltip"
      class:tooltip-top={tooltipPosition === "top"}
      class:tooltip-bottom={tooltipPosition === "bottom"}
      class:tooltip-fadeout={isTooltipFadingOut}
      style="left: {tooltipX}px; top: {tooltipY}px;"
      on:mouseenter={handleTooltipMouseEnter}
      on:mouseleave={handleTooltipMouseLeave}
      role="tooltip"
    >
      <!-- Arrow indicator -->
      <div class="tooltip-arrow"></div>

      <!-- Full-width Flag (no overlay) -->
      <div class="tooltip-flag-container">
        {#if hoveredIso}
          <img
            src="/flags/{hoveredIso.toLowerCase()}.svg"
            alt="{tooltipName} flag"
            class="flag-image-large"
            on:error={(e) => {
              e.currentTarget.style.display = "none";
            }}
          />
        {/if}
      </div>

      <!-- Compact Info Section -->
      <div class="tooltip-content">
        <!-- Stats -->
        {#if tooltipStats}
          <div class="stat-row">
            <span class="stat-icon">📝</span>
            <span class="stat-text">
              <strong>{tooltipStats.noteCount}</strong>
              {tooltipStats.noteCount === 1 ? "note" : "notes"}
            </span>
            {#if isFresh(tooltipStats.lastUpdated)}
              <span class="fresh-indicator">🔥</span>
            {/if}
          </div>
          {#if tooltipStats.lastUpdated}
            <div class="stat-row-small">
              Updated {tooltipStats.lastUpdated}
            </div>
          {/if}
        {:else}
          <div class="stat-row">
            <span class="stat-icon">📝</span>
            <span class="stat-text text-muted">No notes yet</span>
          </div>
        {/if}

        <!-- Summary with country name -->
        {#if tooltipCountry?.summary}
          <div class="tooltip-summary">
            <strong>{tooltipName}</strong>, {tooltipCountry.summary
              .toLowerCase()
              .startsWith(tooltipName.toLowerCase())
              ? tooltipCountry.summary
                  .substring(tooltipName.length)
                  .replace(/^,?\s*/, "")
              : tooltipCountry.summary}
          </div>
        {:else if tooltipName}
          <div class="tooltip-summary">
            <strong>{tooltipName}</strong>
          </div>
        {/if}
      </div>
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
    height: calc(100vh - 300px); /* ← Larger map area */
    min-height: 600px;
  }

  .map-container {
    width: 100%;
    height: 100%; /* ← Fill wrapper */
    background: white;
    border-radius: 8px;
    border: 1px solid #e5e7eb;
    padding: 1rem;
    overflow: hidden; /* ← Prevent scrollbars during pan */
    user-select: none; /* ← Prevent text selection during drag */
  }

  :global(.dark) .map-container {
    background: #1f2937;
    border-color: #374151;
  }

  /* Reset button */
  .reset-button {
    position: absolute;
    top: 20px;
    left: 20px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: white;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: #374151;
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    transition: all 0.2s;
    z-index: 20;
  }

  .reset-button:hover {
    background: #f3f4f6;
    border-color: #9ca3af;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.15);
  }

  :global(.dark) .reset-button {
    background: #1f2937;
    border-color: #4b5563;
    color: #d1d5db;
  }

  :global(.dark) .reset-button:hover {
    background: #374151;
    border-color: #6b7280;
  }

  /* SVG path interaction during zoom */
  :global(#viewport-transform path[role="button"]),
  :global(#viewport-transform circle[role="button"]) {
    pointer-events: all; /* Ensure clickable when zoomed */
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

  .map-tooltip {
    position: fixed;
    background: white;
    border: 1px solid #d1d5db;
    border-radius: 0; /* ← REMOVED: Was 8px */
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15); /* ← Simpler shadow */
    pointer-events: auto;
    z-index: 1000;
    width: 360px;
    overflow: hidden;
    cursor: default;
    animation: tooltipFadeIn 0.2s ease-out;
    animation: tooltipFadeIn 0.15s ease-out;
  }

  @keyframes tooltipFadeIn {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes tooltipFadeOut {
    from {
      opacity: 1;
      transform: translateY(0);
    }
    to {
      opacity: 0;
      transform: translateY(-8px);
    }
  }

  .map-tooltip.tooltip-fadeout {
    animation: tooltipFadeOut 0.15s ease-in;
  }

  :global(.dark) .map-tooltip {
    background: #1f2937;
    border-color: #374151;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  }

  /* Arrow Indicator */
  .tooltip-arrow {
    position: absolute;
    width: 16px;
    height: 16px;
    background: white;
    border: 1px solid #d1d5db;
    transform: rotate(45deg);
    left: 50%;
    margin-left: -8px;
    z-index: 1;
  }

  :global(.dark) .tooltip-arrow {
    background: #1f2937;
    border-color: #374151;
  }

  /* Arrow position */
  .tooltip-bottom .tooltip-arrow {
    top: -9px;
    border-bottom: none;
    border-right: none;
    box-shadow: -2px -2px 4px rgba(0, 0, 0, 0.05); /* ← NEW: subtle shadow */
  }

  .tooltip-top .tooltip-arrow {
    bottom: -9px;
    border-top: none;
    border-left: none;
    box-shadow: 2px 2px 4px rgba(0, 0, 0, 0.05); /* ← NEW: subtle shadow */
  }
  /* Full-width Flag Container (no overlay) */
  .tooltip-flag-container {
    position: relative;
    width: 100%;
    height: 180px;
    overflow: hidden;
    background: #f3f4f6;
  }

  :global(.dark) .tooltip-flag-container {
    background: #111827;
  }

  .flag-image-large {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  /* Compact Content Section */
  .tooltip-content {
    padding: 12px 16px 16px;
  }

  .stat-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .stat-icon {
    font-size: 16px;
    flex-shrink: 0;
  }

  .stat-text {
    font-size: 15px;
    color: #1f2937;
  }

  :global(.dark) .stat-text {
    color: #e5e7eb;
  }

  .stat-text strong {
    font-weight: 600;
    color: #111827;
  }

  :global(.dark) .stat-text strong {
    color: #f9fafb;
  }

  .fresh-indicator {
    margin-left: auto;
    font-size: 14px;
  }

  .stat-row-small {
    font-size: 12px;
    color: #6b7280;
    margin-bottom: 8px;
    padding-left: 24px;
  }

  :global(.dark) .stat-row-small {
    color: #9ca3af;
  }

  .text-muted {
    color: #9ca3af;
    font-style: italic;
  }

  :global(.dark) .text-muted {
    color: #6b7280;
  }

  /* Summary Section with Country Name */
  .tooltip-summary {
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid #e5e7eb;
    font-size: 13px;
    line-height: 1.5;
    color: #4b5563;
  }

  .tooltip-summary strong {
    font-weight: 600;
    color: #111827;
  }

  :global(.dark) .tooltip-summary {
    border-top-color: #374151;
    color: #d1d5db;
  }

  :global(.dark) .tooltip-summary strong {
    color: #f3f4f6;
  }

  /* Header with Flag */
  .tooltip-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-bottom: 1px solid #e5e7eb;
    background: #f9fafb;
  }

  :global(.dark) .tooltip-header {
    background: #111827;
    border-bottom-color: #374151;
  }

  .flag-image {
    width: 56px; /* ← Slightly larger */
    height: 37px;
    object-fit: cover;
    border-radius: 0; /* ← CHANGED: Sharp corners */
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
    flex-shrink: 0;
    border: 1px solid rgba(0, 0, 0, 0.1); /* ← NEW: Subtle border */
  }

  :global(.dark) .flag-image {
    border-color: rgba(255, 255, 255, 0.1);
  }

  .tooltip-title {
    font-weight: 600;
    font-size: 17px; /* ← Slightly larger */
    color: #111827;
    line-height: 1.3;
  }

  :global(.dark) .tooltip-title {
    color: #f9fafb;
  }

  /* Stats Section */
  .tooltip-stats {
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .stat-item {
    font-size: 14px;
    color: #4b5563;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  :global(.dark) .stat-item {
    color: #9ca3af;
  }

  .stat-value {
    font-weight: 600;
    color: #1f2937;
  }

  :global(.dark) .stat-value {
    color: #e5e7eb;
  }

  .text-xs {
    font-size: 12px;
  }

  .fresh-badge {
    color: #059669;
    font-weight: 500;
    font-size: 13px;
  }

  :global(.dark) .fresh-badge {
    color: #34d399;
  }

  /* Summary Section (more space now) */
  .tooltip-summary {
    padding: 12px; /* ← Increased padding */
    border-top: 1px solid #e5e7eb;
    font-size: 13px;
    line-height: 1.6; /* ← Better readability */
    color: #374151;
  }

  :global(.dark) .tooltip-summary {
    border-top-color: #374151;
    color: #d1d5db;
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
