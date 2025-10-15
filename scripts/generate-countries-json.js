// Generate a single countries.json with all metadata for embedding
import fs from 'fs/promises';
import path from 'path';

const COUNTRIES_JSON = path.join(process.cwd(), 'src/lib/data/countries.json');
const OUTPUT_PATH = path.join(process.cwd(), 'src-tauri/src/data/countries.json');

async function main() {
  console.log('Reading countries.json...');
  const countriesData = await fs.readFile(COUNTRIES_JSON, 'utf8');
  const countries = JSON.parse(countriesData);
  
  console.log(`Processing ${countries.length} countries...`);
  
  // Simplified metadata for embedding with defaults for missing fields
  const metadata = countries.map(country => ({
    slug: country.slug,
    name: country.name,
    iso2: country.iso2 || country.slug.toUpperCase(),
    iso3: country.iso3 || country.iso2?.toUpperCase() || country.slug.toUpperCase().slice(0, 3),
    summary: country.summary || `Country: ${country.name}`,
    region: country.region || 'Unknown',
    subregion: country.subregion || 'Unknown',
  }));
  
  console.log('Sample entries:');
  metadata.slice(0, 3).forEach(c => {
    console.log(`  ${c.name}: iso2=${c.iso2}, iso3=${c.iso3}`);
  });
  
  // Ensure output directory exists
  const outputDir = path.dirname(OUTPUT_PATH);
  await fs.mkdir(outputDir, { recursive: true });
  
  // Write to Rust src directory
  await fs.writeFile(
    OUTPUT_PATH,
    JSON.stringify(metadata, null, 2),
    'utf8'
  );
  
  console.log(`\n✅ Generated ${OUTPUT_PATH}`);
  console.log(`   ${metadata.length} countries, ${(await fs.stat(OUTPUT_PATH)).size} bytes`);
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});