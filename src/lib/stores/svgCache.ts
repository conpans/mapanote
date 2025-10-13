import { writable } from 'svelte/store';

// Module-level cache (persists across component remounts)
let cachedSvg: string | null = null;

export async function loadWorldSvg(): Promise<string> {
  if (cachedSvg) {
    console.log('Using cached SVG');
    return cachedSvg;
  }

  console.log('Fetching SVG from server...');
  const response = await fetch('/maps/world.svg');
  
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  const svgText = await response.text();
  cachedSvg = svgText;
  
  console.log('SVG fetched and cached, size:', svgText.length);
  return svgText;
}