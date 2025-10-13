// Filter countries.json to only include those in the SVG
import fs from 'fs/promises';
import path from 'path';

const SVG_COUNTRIES_PATH = path.join(process.cwd(), 'scripts/svg-countries.json');
const COUNTRIES_JSON_PATH = path.join(process.cwd(), 'src/lib/data/countries.json');
const OUTPUT_PATH = path.join(process.cwd(), 'src/lib/data/countries-filtered.json');

async function main() {
  console.log('Loading SVG country list...');
  const svgCountries = JSON.parse(await fs.readFile(SVG_COUNTRIES_PATH, 'utf8'));
  const svgCodesSet = new Set(svgCountries);
  
  console.log('Loading full countries.json...');
  const allCountries = JSON.parse(await fs.readFile(COUNTRIES_JSON_PATH, 'utf8'));
  
  console.log(`Filtering ${allCountries.length} countries...`);
  const filtered = allCountries.filter(country => 
    svgCodesSet.has(country.iso2.toUpperCase())
  );
  
  console.log(`\n✅ Filtered to ${filtered.length} countries (in SVG)`);
  console.log(`   Excluded ${allCountries.length - filtered.length} countries (not in SVG)`);
  
  // Show some excluded countries
  const excluded = allCountries
    .filter(c => !svgCodesSet.has(c.iso2.toUpperCase()))
    .slice(0, 10)
    .map(c => `${c.name} (${c.iso2})`);
  
  console.log(`\n   Examples excluded: ${excluded.join(', ')}`);
  
  // Save filtered version
  await fs.writeFile(
    OUTPUT_PATH,
    JSON.stringify(filtered, null, 2),
    'utf8'
  );
  
  console.log(`\n✅ Saved filtered list to ${OUTPUT_PATH}`);
  console.log(`\nNext steps:`);
  console.log(`  1. Review the filtered list`);
  console.log(`  2. If looks good, replace countries.json:`);
  console.log(`     mv src/lib/data/countries-filtered.json src/lib/data/countries.json`);
  console.log(`  3. Regenerate vault:`);
  console.log(`     npm run generate-vault`);
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});