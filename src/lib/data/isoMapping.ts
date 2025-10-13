/**
 * ISO2 ↔ Slug mappings auto-generated from countries.json
 * This file is generated - do not edit manually
 */

import countriesData from './countries.json';
import type { CountryMetadata } from './countryHelpers';

const COUNTRIES: CountryMetadata[] = countriesData as CountryMetadata[];

/**
 * Maps SVG path IDs (uppercase ISO2) to vault slugs (lowercase)
 */
export const isoToSlug: Record<string, string> = {};

/**
 * Reverse mapping: slug → ISO2
 */
export const slugToIso: Record<string, string> = {};

/**
 * Maps ISO2 codes to readable country names
 */
export const isoToName: Record<string, string> = {};

// Build all mappings from countries.json
COUNTRIES.forEach(country => {
  const isoUpper = country.iso2.toUpperCase();
  const isoLower = country.iso2.toLowerCase();
  
  // SVG uses uppercase IDs, slugs are lowercase
  isoToSlug[isoUpper] = country.slug;
  slugToIso[country.slug] = isoUpper;
  isoToName[isoUpper] = country.name;
  
  // Also support lowercase lookups
  isoToSlug[isoLower] = country.slug;
  isoToName[isoLower] = country.name;
});

/**
 * Get country metadata by ISO2 code
 */
export function getCountryByIso2(iso2: string): CountryMetadata | undefined {
  return COUNTRIES.find(c => 
    c.iso2.toUpperCase() === iso2.toUpperCase()
  );
}

/**
 * Get country metadata by slug
 */
export function getCountryBySlug(slug: string): CountryMetadata | undefined {
  return COUNTRIES.find(c => c.slug === slug);
}

/**
 * Check if a country exists in our metadata
 */
export function hasCountry(iso2OrSlug: string): boolean {
  return COUNTRIES.some(c => 
    c.iso2.toUpperCase() === iso2OrSlug.toUpperCase() ||
    c.slug === iso2OrSlug.toLowerCase()
  );
}

// Export the full list for convenience
export const ALL_COUNTRIES = COUNTRIES;