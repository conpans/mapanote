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
  import {
    countriesMetadata,
    loadCountriesMetadata,
  } from "$lib/stores/countriesMetadata";
  import { get } from "svelte/store";
  import { vaultOpened } from "$lib/stores/vault";

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

  const colorCache = new Map<string, { top: string; bottom: string }>();
  let arrowBackgroundColor = $state<string>("white");
  let tooltipArrowPosition = $state<"left" | "right" | "center">("center");

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
      // 1) Always load embedded country metadata (no disk churn, instant for first-time users)
      console.log("Loading embedded countries metadata...");
      await loadCountriesMetadata();

      // 2) Only load vault-backed stats if a vault is currently opened
      if (get(vaultOpened)) {
        // ← CHANGED: Use get() instead of $
        console.log("Loading map stats from vault...");
        await loadMapStats();
      } else {
        console.log(
          "No vault opened - showing map with embedded metadata only."
        );
      }

      // 3) Load and mount the SVG
      if (!mapContainer) throw new Error("mapContainer ref not bound");

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

        // Wrap content in viewport group for pan/zoom
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

      (path as SVGPathElement).style.cursor = "pointer";
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

      (circle as SVGCircleElement).style.cursor = "pointer";
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
    const offset = 15; // Distance from cursor
    const arrowOffset = 40; // Distance of arrow from edge
    const tooltipWidth = 360;
    const tooltipHeight = 320;
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    const mouseX = e.clientX;
    const mouseY = e.clientY;

    // Calculate which side has more space
    const spaceRight = viewportWidth - mouseX;
    const spaceLeft = mouseX;
    const spaceBelow = viewportHeight - mouseY;
    const spaceAbove = mouseY;

    // Determine horizontal position
    let x: number;
    let arrowXPosition: "left" | "right" | "center";

    // Prefer showing on the side with more space
    if (spaceLeft > spaceRight && spaceLeft > tooltipWidth + offset) {
      // Show on left side
      x = mouseX - tooltipWidth - offset;
      arrowXPosition = "right";
    } else if (spaceRight > tooltipWidth + offset) {
      // Show on right side
      x = mouseX + offset;
      arrowXPosition = "left";
    } else if (spaceLeft > tooltipWidth + offset) {
      // Fall back to left if right doesn't fit
      x = mouseX - tooltipWidth - offset;
      arrowXPosition = "right";
    } else {
      // Center it as last resort
      x = Math.max(
        10,
        Math.min(mouseX - tooltipWidth / 2, viewportWidth - tooltipWidth - 10)
      );
      arrowXPosition = "center";
    }

    // Determine vertical position
    let y: number;
    if (spaceBelow > tooltipHeight + offset) {
      // Show below cursor
      y = mouseY + offset;
      tooltipPosition = "bottom";
    } else if (spaceAbove > tooltipHeight + offset) {
      // Show above cursor
      y = mouseY - tooltipHeight - offset;
      tooltipPosition = "top";
    } else {
      // Default to below
      y = Math.max(10, mouseY + offset);
      tooltipPosition = "bottom";
    }

    tooltipX = x;
    tooltipY = y;
    tooltipArrowPosition = arrowXPosition;
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

  // Function to detect flag edge colors based on arrow position
  async function detectFlagEdgeColors(
    isoCode: string,
    arrowPos: "left" | "right" | "center"
  ): Promise<{ top: string; bottom: string } | null> {
    // Create cache key including arrow position
    const cacheKey = `${isoCode}-${arrowPos}`;
    if (colorCache.has(cacheKey)) {
      return colorCache.get(cacheKey)!;
    }

    try {
      const img = new Image();
      img.crossOrigin = "anonymous";
      img.src = `/flags/${isoCode.toLowerCase()}.svg`;

      await new Promise<void>((resolve, reject) => {
        img.onload = () => resolve();
        img.onerror = () => reject(new Error("Failed to load flag"));
        setTimeout(() => reject(new Error("Timeout")), 2000);
      });

      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d", { willReadFrequently: true });

      if (!ctx) return null;

      canvas.width = img.width || 100;
      canvas.height = img.height || 60;
      ctx.drawImage(img, 0, 0);

      // Determine X position based on where arrow is
      let sampleX: number;
      if (arrowPos === "left") {
        sampleX = canvas.width * 0.1; // Sample from left side (10% in)
      } else if (arrowPos === "right") {
        sampleX = canvas.width * 0.9; // Sample from right side (90% in)
      } else {
        sampleX = canvas.width / 2; // Sample from center
      }

      // Sample Y positions (top and bottom of flag)
      const topY = Math.min(10, canvas.height * 0.15);
      const bottomY = canvas.height - Math.min(10, canvas.height * 0.15);

      // Get color from top edge at arrow's X position
      const topPixel = ctx.getImageData(sampleX, topY, 1, 1).data;
      const topColor = `rgb(${topPixel[0]}, ${topPixel[1]}, ${topPixel[2]})`;

      // Get color from bottom edge at arrow's X position
      const bottomPixel = ctx.getImageData(sampleX, bottomY, 1, 1).data;
      const bottomColor = `rgb(${bottomPixel[0]}, ${bottomPixel[1]}, ${bottomPixel[2]})`;

      const colors = { top: topColor, bottom: bottomColor };

      // Cache the result
      colorCache.set(cacheKey, colors);

      return colors;
    } catch (error) {
      console.warn(`Failed to detect colors for ${isoCode}:`, error);
      return null;
    }
  }

  // Effect to update arrow color when country or position changes
  $effect(() => {
    if (hoveredIso && shouldShowTooltip) {
      // Only use flag color when tooltip is BELOW cursor (arrow at top points up into flag)
      if (tooltipPosition === "bottom") {
        detectFlagEdgeColors(hoveredIso, tooltipArrowPosition).then(
          (colors) => {
            if (colors) {
              arrowBackgroundColor = colors.top;
            } else {
              arrowBackgroundColor = "white";
            }
          }
        );
      } else {
        // Tooltip is ABOVE cursor - arrow at bottom should match background
        // Don't set inline style, let CSS handle dark mode
        arrowBackgroundColor = ""; // Empty string removes inline style
      }
    }
  });
</script>

<svelte:window on:mousemove={handlePanMove} on:mouseup={handlePanEnd} />

<div class="world-map-wrapper">
  <div
    class="map-container"
    bind:this={mapContainer}
    onwheel={handleWheel}
    onmousedown={handlePanStart}
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
      onclick={resetView}
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
  <!-- Enhanced Tooltip with Flag and Smooth Transitions -->
  <!-- Enhanced Tooltip with Flag and Smooth Transitions -->
  {#if hoveredIso && tooltipName && !isLoading && shouldShowTooltip}
    {#key hoveredIso}
      <div
        class="map-tooltip {tooltipPosition === 'top'
          ? 'tooltip-top'
          : 'tooltip-bottom'}"
        class:tooltip-fadeout={isTooltipFadingOut}
        style="left: {tooltipX}px; top: {tooltipY}px;"
        onmouseenter={handleTooltipMouseEnter}
        onmouseleave={handleTooltipMouseLeave}
        role="tooltip"
      >
        <!-- Arrow indicator -->
        <div
          class="tooltip-arrow arrow-{tooltipArrowPosition}"
          style:background-color={arrowBackgroundColor || undefined}
        ></div>

        <!-- Full-width Flag -->
        <div class="tooltip-flag-container">
          {#if hoveredIso}
            <img
              src="/flags/{hoveredIso.toLowerCase()}.svg"
              alt="{tooltipName} flag"
              class="flag-image-large"
              onerror={(e) => {
                (e.currentTarget as HTMLImageElement).style.display = "none";
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
    {/key}
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
    height: 100%;
    min-height: 400px;
  }

  .map-container {
    width: 100%;
    height: 100%;
    background: white;
    border-radius: 8px;
    border: 1px solid #e5e7eb;
    padding: 1rem;
    overflow: hidden;
    user-select: none;
  }

  :global(.dark) .map-container {
    background: #111827;
    border-color: #1f2937;
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
    border-color: #374151;
    color: #d1d5db;
  }

  :global(.dark) .reset-button:hover {
    background: #374151;
    border-color: #4b5563;
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
    background: rgba(17, 24, 39, 0.9);
  }

  /* SVG styling */
  :global(.map-container svg) {
    display: block;
  }

  :global(.map-container path) {
    vector-effect: non-scaling-stroke;
    transition: all 0.15s ease;
  }

  /* LIGHT MODE - Refined heat map colors */
  :global(.activity-none) {
    fill: #f9fafb !important;
    stroke: #e5e7eb !important;
    stroke-width: 0.5;
  }

  :global(.activity-low) {
    fill: #dbeafe !important;
    stroke: #93c5fd !important;
    stroke-width: 0.6;
  }

  :global(.activity-medium) {
    fill: #93c5fd !important;
    stroke: #3b82f6 !important;
    stroke-width: 0.7;
  }

  :global(.activity-high) {
    fill: #3b82f6 !important;
    stroke: #2563eb !important;
    stroke-width: 0.8;
  }

  :global(.activity-very-high) {
    fill: #2563eb !important;
    stroke: #1d4ed8 !important;
    stroke-width: 1;
  }

  /* DARK MODE - Better contrast and glow */
  :global(.dark .activity-none) {
    fill: #1f2937 !important;
    stroke: #374151 !important;
    stroke-width: 0.5;
  }

  :global(.dark .activity-low) {
    fill: #1e3a8a !important;
    stroke: #3b82f6 !important;
    stroke-width: 0.7;
    filter: drop-shadow(0 0 2px rgba(59, 130, 246, 0.3));
  }

  :global(.dark .activity-medium) {
    fill: #1e40af !important;
    stroke: #60a5fa !important;
    stroke-width: 0.8;
    filter: drop-shadow(0 0 3px rgba(96, 165, 250, 0.4));
  }

  :global(.dark .activity-high) {
    fill: #2563eb !important;
    stroke: #93c5fd !important;
    stroke-width: 1;
    filter: drop-shadow(0 0 4px rgba(147, 197, 253, 0.5));
  }

  :global(.dark .activity-very-high) {
    fill: #3b82f6 !important;
    stroke: #bfdbfe !important;
    stroke-width: 1.2;
    filter: drop-shadow(0 0 6px rgba(191, 219, 254, 0.6));
  }

  /* Fresh indicator (recent notes) */
  :global(.fresh) {
    stroke: #10b981 !important;
    stroke-width: 2 !important;
    animation: pulse-green 2s ease-in-out infinite;
  }

  :global(.dark .fresh) {
    stroke: #34d399 !important;
    filter: drop-shadow(0 0 8px rgba(52, 211, 153, 0.7));
  }

  @keyframes pulse-green {
    0%,
    100% {
      stroke-opacity: 1;
    }
    50% {
      stroke-opacity: 0.7;
    }
  }

  /* Hover states */
  :global(path[role="button"]:hover) {
    opacity: 0.85;
    stroke-width: 2 !important;
    filter: brightness(1.1);
  }

  :global(.dark path[role="button"]:hover) {
    filter: brightness(1.3) drop-shadow(0 0 8px currentColor);
  }

  :global(path[role="button"]:focus) {
    outline: 2px solid #3b82f6;
    outline-offset: 2px;
  }

  /* Microstate styles - LIGHT MODE */
  :global(#microstates circle) {
    vector-effect: non-scaling-stroke;
    transition: all 0.2s ease;
  }

  :global(.microstate-none) {
    fill: #d1d5db !important;
    stroke: #9ca3af !important;
    stroke-width: 0.8;
    r: 1.2;
  }

  :global(.microstate-low) {
    fill: #93c5fd !important;
    stroke: #60a5fa !important;
    stroke-width: 1;
    r: 1.5;
  }

  :global(.microstate-medium),
  :global(.microstate-high),
  :global(.microstate-very-high) {
    fill: #3b82f6 !important;
    stroke: #2563eb !important;
    stroke-width: 1.2;
    r: 1.8;
  }

  /* Microstate styles - DARK MODE */
  :global(.dark .microstate-none) {
    fill: #4b5563 !important;
    stroke: #6b7280 !important;
    stroke-width: 0.8;
    r: 1.3;
  }

  :global(.dark .microstate-low) {
    fill: #3b82f6 !important;
    stroke: #60a5fa !important;
    stroke-width: 1.2;
    r: 1.6;
    filter: drop-shadow(0 0 3px rgba(59, 130, 246, 0.6));
  }

  :global(.dark .microstate-medium),
  :global(.dark .microstate-high),
  :global(.dark .microstate-very-high) {
    fill: #60a5fa !important;
    stroke: #93c5fd !important;
    stroke-width: 1.4;
    r: 2;
    filter: drop-shadow(0 0 5px rgba(147, 197, 253, 0.8));
  }

  :global(#microstates circle[role="button"]:hover) {
    r: 2.5 !important;
    stroke-width: 2 !important;
    filter: drop-shadow(0 0 10px currentColor) !important;
  }

  /* Tooltip styles remain the same... */
  .map-tooltip {
    position: fixed;
    background: white;
    border: 1px solid #d1d5db;
    border-radius: 0;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    pointer-events: auto;
    z-index: 1000;
    width: 360px;
    overflow: visible; /* ← CHANGED */
    cursor: default;
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

  .tooltip-arrow {
    position: absolute;
    width: 20px;
    height: 20px;
    background: white;
    border: 1px solid rgba(0, 0, 0, 0.1);
    transform: rotate(45deg);
    z-index: 2;
    transition: background-color 0.2s ease;
  }

  :global(.dark) .tooltip-arrow {
    background: #1f2937;
    border-color: rgba(255, 255, 255, 0.1);
  }

  /* Vertical position */
  .tooltip-bottom .tooltip-arrow {
    top: -10px;
    border-bottom: none;
    border-right: none;
    box-shadow: -2px -2px 3px rgba(0, 0, 0, 0.1);
  }

  .tooltip-top .tooltip-arrow {
    bottom: -10px;
    border-top: none;
    border-left: none;
    box-shadow: 2px 2px 3px rgba(0, 0, 0, 0.1);
  }

  /* Horizontal position - closer to edges */
  .tooltip-arrow.arrow-left {
    left: 25px; /* Closer to left edge */
  }

  .tooltip-arrow.arrow-right {
    right: 25px; /* Closer to right edge */
  }

  .tooltip-arrow.arrow-center {
    left: 50%;
    margin-left: -10px;
  }

  .tooltip-flag-container {
    position: relative;
    width: 100%;
    height: 180px;
    overflow: hidden;
    background: #f3f4f6;
    z-index: 1;
  }

  :global(.dark) .tooltip-flag-container {
    background: #111827;
  }

  .flag-image-large {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    position: relative;
    z-index: 0;
  }

  .tooltip-content {
    padding: 12px 16px 16px;
    position: relative;
    z-index: 1; /* ← ADD THIS */
    background: white; /* ← ADD THIS */
  }

  :global(.dark) .tooltip-content {
    background: #1f2937; /* ← ADD THIS */
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
    background: #f9fafb;
  }
  .legend-swatch.activity-low {
    background: #dbeafe;
  }
  .legend-swatch.activity-medium {
    background: #93c5fd;
  }
  .legend-swatch.activity-high {
    background: #3b82f6;
  }
  .legend-swatch.activity-very-high {
    background: #2563eb;
  }
  .legend-swatch.fresh-indicator {
    background: #10b981;
    border-color: #10b981;
  }

  :global(.dark) .legend-swatch.activity-none {
    background: #1f2937;
  }
  :global(.dark) .legend-swatch.activity-low {
    background: #1e3a8a;
  }
  :global(.dark) .legend-swatch.activity-medium {
    background: #1e40af;
  }
  :global(.dark) .legend-swatch.activity-high {
    background: #2563eb;
  }
  :global(.dark) .legend-swatch.activity-very-high {
    background: #3b82f6;
  }
</style>
