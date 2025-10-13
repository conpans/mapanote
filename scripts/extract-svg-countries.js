// Extract ISO2 codes from the world SVG
import fs from 'fs/promises';
import path from 'path';

const SVG_PATH = path.join(process.cwd(), 'static/maps/world.svg');
const OUTPUT_PATH = path.join(process.cwd(), 'scripts/svg-countries.json');

async function main() {
  console.log('Reading SVG file...');
  const svgContent = await fs.readFile(SVG_PATH, 'utf8');
  
  const isoCodes = new Set();
  
  // Extract from path elements: <path id="US" ...>
  const pathRegex = /<path[^>]+id="([A-Z]{2})"[^>]*>/g;
  let match;
  while ((match = pathRegex.exec(svgContent)) !== null) {
    isoCodes.add(match[1]);
  }
  
  // Extract from microstate circles: <circle data-code="VA" ...>
  const circleRegex = /<circle[^>]+data-code="([A-Z]{2})"[^>]*>/g;
  while ((match = circleRegex.exec(svgContent)) !== null) {
    isoCodes.add(match[1]);
  }
  
  const sortedCodes = Array.from(isoCodes).sort();
  
  console.log(`Found ${sortedCodes.length} countries in SVG:`);
  console.log(sortedCodes.join(', '));
  
  // Save to file
  await fs.writeFile(
    OUTPUT_PATH,
    JSON.stringify(sortedCodes, null, 2),
    'utf8'
  );
  
  console.log(`\n✅ Saved to ${OUTPUT_PATH}`);
  console.log(`   Use this to filter countries.json`);
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});