// Fetch data from mledoze/countries and transform to our format
import https from 'https';
import fs from 'fs/promises';
import path from 'path';

const SOURCE_URL = 'https://raw.githubusercontent.com/mledoze/countries/master/countries.json';
const OUTPUT_PATH = path.join(process.cwd(), 'src/lib/data/countries.json');

function fetchJSON(url) {
  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch (e) {
          reject(e);
        }
      });
    }).on('error', reject);
  });
}

function generateSummary(country) {
  // Create a concise summary from available data
  const parts = [];
  
  if (country.subregion) parts.push(country.subregion);
  if (country.unMember) parts.push('UN member');
  if (country.region) parts.push(`in ${country.region}`);
  
  // Add capital if available
  if (country.capital && country.capital.length > 0) {
    parts.push(`capital ${country.capital[0]}`);
  }
  
  // Add currency if available
  if (country.currencies) {
    const currencyCode = Object.keys(country.currencies)[0];
    if (currencyCode) {
      parts.push(`${currencyCode} currency`);
    }
  }
  
  return parts.join('; ') + '.';
}

async function main() {
  console.log('Fetching country data from mledoze/countries...');
  const rawData = await fetchJSON(SOURCE_URL);
  
  console.log(`Fetched ${rawData.length} countries`);
  
  // Transform to our format
  const transformed = rawData
    .filter(c => c.cca2 && c.name?.common) // Must have ISO2 and name
    .map(country => ({
      iso2: country.cca2,
      slug: country.cca2.toLowerCase(),
      name: country.name.common,
      region: country.region || 'Unknown',
      subregion: country.subregion || country.region || 'Unknown',
      summary: generateSummary(country),
      aliases: [
        country.name.official,
        ...(country.altSpellings || [])
      ].filter((name, i, arr) => 
        name !== country.name.common && arr.indexOf(name) === i
      ).slice(0, 3) // Keep max 3 aliases
    }))
    .sort((a, b) => a.name.localeCompare(b.name)); // Sort alphabetically
  
  console.log(`Transformed ${transformed.length} countries`);
  
  // Ensure directory exists
  await fs.mkdir(path.dirname(OUTPUT_PATH), { recursive: true });
  
  // Write to file
  await fs.writeFile(
    OUTPUT_PATH,
    JSON.stringify(transformed, null, 2),
    'utf8'
  );
  
  console.log(`✅ Written to ${OUTPUT_PATH}`);
  console.log(`   Size: ${(JSON.stringify(transformed).length / 1024).toFixed(1)}KB`);
  console.log(`   Sample: ${transformed[0].name} (${transformed[0].iso2})`);
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});