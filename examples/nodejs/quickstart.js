// EdgeVec Node.js Quick Start Example
// Demonstrates basic insert/search operations with the EdgeVecClient wrapper

import { EdgeVecClient } from '@edgevec/core';

async function main() {
    console.log('EdgeVec Node.js Quick Start\n');

    // 1. Create EdgeVec client (auto-initializes WASM)
    console.log('Creating EdgeVec client...');
    const client = await EdgeVecClient.create({
        dimensions: 128,
        metric: 'cosine' // Options: 'l2', 'cosine', 'dot'
    });
    console.log(`✓ Client created (${client.dimensions}d, cosine metric)\n`);

    // 2. Insert vectors (synchronous)
    console.log('Inserting vectors...');
    const vector1 = new Float32Array(128).fill(0.1);
    const vector2 = new Float32Array(128).fill(0.2);
    const vector3 = new Float32Array(128).fill(0.3);

    const id1 = client.insert(vector1);
    const id2 = client.insert(vector2);
    const id3 = client.insert(vector3);

    console.log(`✓ Inserted 3 vectors with IDs: ${id1}, ${id2}, ${id3}`);
    console.log(`  Total vectors: ${client.length}\n`);

    // 3. Search for nearest neighbors (synchronous)
    console.log('Searching for nearest neighbors...');
    const query = new Float32Array(128).fill(0.15); // Query closer to vector1
    const results = client.search(query, 2); // Find top 2

    console.log(`✓ Found ${results.length} results:`);
    results.forEach((result, i) => {
        console.log(`  ${i + 1}. ID: ${result.id}, Distance: ${result.distance.toFixed(6)}`);
    });
    console.log();

    // 4. Save to IndexedDB (in browser) or file (Node.js)
    console.log('Saving database...');
    await client.save('quickstart-db');
    console.log('✓ Database saved\n');

    // 5. Load from storage
    console.log('Loading database...');
    const loadedClient = await EdgeVecClient.load('quickstart-db', {
        dimensions: 128,
        metric: 'cosine'
    });
    console.log(`✓ Database loaded`);
    console.log(`  Vectors in loaded client: ${loadedClient.length}`);
    console.log('  Note: Vector count shows 0 due to WASM API limitation.');
    console.log('  Actual vectors are loaded and searchable.\n');

    // 6. Verify loaded database works
    console.log('Verifying loaded database...');
    const verifyQuery = new Float32Array(128).fill(0.25); // Query closer to vector2
    const verifyResults = loadedClient.search(verifyQuery, 2);
    console.log(`✓ Search on loaded database returned ${verifyResults.length} results:`);
    verifyResults.forEach((result, i) => {
        console.log(`  ${i + 1}. ID: ${result.id}, Distance: ${result.distance.toFixed(6)}`);
    });

    console.log('\n✓ Quick start complete!');
}

main().catch(err => {
    console.error('✗ Error:', err.message);
    console.error(err.stack);
    process.exit(1);
});
