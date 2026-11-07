import fs from 'node:fs';
import path from 'node:path';
import { createDemoSnapshot } from './generateDemoData.js';
import { validateSnapshot } from '../shared/validation.js';

const snapshot = createDemoSnapshot();
const validation = validateSnapshot(snapshot);

if (!validation.success) {
  console.error('Demo snapshot validation failed:', validation.errors);
  process.exit(1);
}

const json = JSON.stringify(snapshot, null, 2);
fs.writeFileSync(path.resolve('public/demo-snapshot.json'), json, 'utf-8');
fs.writeFileSync(path.resolve('src/demo/demo-snapshot.json'), json, 'utf-8');
console.log('✅ Demo snapshot created and validated successfully at public/demo-snapshot.json and src/demo/demo-snapshot.json');
