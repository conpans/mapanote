// Generate starter vault with all country pages
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const COUNTRIES_PATH = path.join(__dirname, '../src/lib/data/countries.json');
const OUTPUT_DIR = path.join(__dirname, '../vault-template');

// Frontmatter template
function generateCountryMarkdown(country) {
  const today = new Date().toISOString().split('T')[0];
  
  // Build aliases array for YAML
  const aliasesYaml = country.aliases.length > 0
    ? `[${country.aliases.map(a => `"${a}"`).join(', ')}]`
    : '[]';
  
  return `---
title: ${country.name}
slug: ${country.slug}
region: ${country.subregion}
summary: ${country.summary}
aliases: ${aliasesYaml}
updated_at: ${today}
---

## Overview

${country.summary}

## Notes

<!-- Add your first note below -->
`;
}

// Vault config
function generateVaultConfig() {
  const now = new Date().toISOString();
  return JSON.stringify({
    schema_version: 1,
    created_at: now,
    app_version: "0.1.0"
  }, null, 2);
}

async function main() {
  console.log('🏗️  Generating Starter Vault...\n');
  
  // Read country data
  const countriesJson = await fs.readFile(COUNTRIES_PATH, 'utf8');
  const countries = JSON.parse(countriesJson);
  
  console.log(`📊 Found ${countries.length} countries in metadata`);
  
  // Clean output directory
  try {
    await fs.rm(OUTPUT_DIR, { recursive: true, force: true });
  } catch (e) {
    // Directory doesn't exist, that's fine
  }
  
  // Create base structure
  await fs.mkdir(OUTPUT_DIR, { recursive: true });
  await fs.mkdir(path.join(OUTPUT_DIR, 'countries'), { recursive: true });
  
  // Generate vault.json
  await fs.writeFile(
    path.join(OUTPUT_DIR, 'vault.json'),
    generateVaultConfig(),
    'utf8'
  );
  console.log('✅ Created vault.json');
  
  // Generate each country page
  let created = 0;
  for (const country of countries) {
    const countryDir = path.join(OUTPUT_DIR, 'countries', country.slug);
    const indexPath = path.join(countryDir, 'index.md');
    
    await fs.mkdir(countryDir, { recursive: true });
    await fs.writeFile(indexPath, generateCountryMarkdown(country), 'utf8');
    
    created++;
    if (created % 50 === 0) {
      console.log(`   📝 Created ${created}/${countries.length} pages...`);
    }
  }
  
  console.log(`✅ Created ${created} country pages\n`);
  
  // Calculate size
  const stats = await getDirectorySize(OUTPUT_DIR);
  console.log(`📦 Vault size: ${(stats / 1024).toFixed(1)}KB`);
  console.log(`📁 Location: ${OUTPUT_DIR}`);
  console.log(`\n✨ Starter Vault generated successfully!`);
}

async function getDirectorySize(dirPath) {
  let size = 0;
  const files = await fs.readdir(dirPath, { recursive: true, withFileTypes: true });
  
  for (const file of files) {
    if (file.isFile()) {
      const fullPath = path.join(file.path || dirPath, file.name);
      const stats = await fs.stat(fullPath);
      size += stats.size;
    }
  }
  
  return size;
}

main().catch(err => {
  console.error('❌ Error:', err);
  process.exit(1);
});