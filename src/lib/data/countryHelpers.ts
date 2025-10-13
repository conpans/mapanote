import countriesData from './countries.json';

export interface CountryMetadata {
  iso2: string;
  slug: string;
  name: string;
  region: string;
  subregion: string;
  summary: string;
  aliases: string[];
}

// Type the imported JSON
const COUNTRIES: CountryMetadata[] = countriesData as CountryMetadata[];

// Create lookup maps for fast access
const bySlug = new Map<string, CountryMetadata>();
const byIso2 = new Map<string, CountryMetadata>();

COUNTRIES.forEach(country => {
  bySlug.set(country.slug, country);
  byIso2.set(country.iso2, country);
  byIso2.set(country.iso2.toLowerCase(), country); // Support lowercase lookups
});

export function getCountryBySlug(slug: string): CountryMetadata | undefined {
  return bySlug.get(slug);
}

export function getCountryByIso2(iso2: string): CountryMetadata | undefined {
  return byIso2.get(iso2) || byIso2.get(iso2.toUpperCase());
}

export function getAllCountries(): CountryMetadata[] {
  return COUNTRIES;
}

export function getCountryCount(): number {
  return COUNTRIES.length;
}

// Flag helpers
export function getFlagSrc(iso2: string): string {
  return `/flags/${iso2.toLowerCase()}.svg`;
}

export function getFlagEmoji(iso2: string): string {
  // Convert ISO2 to regional indicator emoji (🇫🇮)
  const codePoints = [...iso2.toUpperCase()].map(char => 
    0x1F1E6 - 65 + char.charCodeAt(0)
  );
  return String.fromCodePoint(...codePoints);
}

// Truncate helper for summaries
export function truncate(text: string, maxLength: number = 100): string {
  if (text.length <= maxLength) return text;
  const truncated = text.slice(0, maxLength).trim();
  const lastSpace = truncated.lastIndexOf(' ');
  return (lastSpace > 0 ? truncated.slice(0, lastSpace) : truncated) + '...';
}