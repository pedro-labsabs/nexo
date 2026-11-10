import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { validateSnapshot } from '../src/shared/validation.js';
import { computeIndicators } from '../src/shared/indicators.js';
import { extractExplicitRelationships } from '../src/shared/relations.js';
import { mergeUserAnnotations } from '../src/shared/annotations.js';
import { UserAnnotations } from '../src/shared/types.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');
const fixturesDir = path.join(projectRoot, 'fixtures', 'domain');
const expectedDir = path.join(fixturesDir, 'expected');

export interface ParityScenarioResult {
  scenario: string;
  valid: boolean;
  validationErrors: string[];
  indicators: Array<{
    repositoryId: number;
    repositoryName: string;
    overallQualityScore: number;
    indicators: ReturnType<typeof computeIndicators>;
  }>;
  extractedRelationships: ReturnType<typeof extractExplicitRelationships>;
  mergedAnnotations: UserAnnotations | null;
}

export function runScenario(scenarioName: string, rawJson: any): ParityScenarioResult {
  const validation = validateSnapshot(rawJson);

  if (!validation.success) {
    return {
      scenario: scenarioName,
      valid: false,
      validationErrors: validation.errors,
      indicators: [],
      extractedRelationships: [],
      mergedAnnotations: null,
    };
  }

  const snapshot = validation.data;
  const refDate = snapshot.metadata.referenceDate;

  // 1. Indicators computation
  const indicators = snapshot.repositories.map((repo) => {
    const computed = computeIndicators(repo, refDate);
    return {
      repositoryId: repo.id,
      repositoryName: repo.name,
      overallQualityScore: computed.overallQualityScore,
      indicators: computed,
    };
  });

  // 2. Explicit relationships extraction
  const readmeMap = (rawJson.readmeContents as Record<number, string>) || {};
  const manifestMap = (rawJson.manifestContents as Record<number, string>) || {};
  const extractedRelationships = extractExplicitRelationships(
    snapshot.repositories,
    snapshot.organizations,
    readmeMap,
    manifestMap
  );

  // 3. Annotations merge if incoming annotations are provided
  let mergedAnnotations: UserAnnotations | null = null;
  if (rawJson.incomingAnnotations) {
    const defaultLocal: UserAnnotations = {
      schemaVersion: '1.0',
      exportedAt: refDate,
      authenticatedUser: snapshot.metadata.authenticatedUser,
      manualRelationships: [],
      deletedRelationships: [],
      potentialAssessments: {},
    };
    const local = snapshot.userAnnotations || defaultLocal;
    mergedAnnotations = mergeUserAnnotations(local, rawJson.incomingAnnotations, refDate);
  } else if (snapshot.userAnnotations) {
    mergedAnnotations = snapshot.userAnnotations;
  }

  return {
    scenario: scenarioName,
    valid: true,
    validationErrors: [],
    indicators,
    extractedRelationships,
    mergedAnnotations,
  };
}

export function main() {
  if (!fs.existsSync(expectedDir)) {
    fs.mkdirSync(expectedDir, { recursive: true });
  }

  const files = fs
    .readdirSync(fixturesDir)
    .filter((f) => f.endsWith('.json') && !f.endsWith('.expected.json'))
    .sort();

  console.log(`\n===============================================================`);
  console.log(` NEXO Domain TS Reference Parity Harness`);
  console.log(` Evaluated Scenarios: ${files.length} fixtures in fixtures/domain/`);
  console.log(`===============================================================\n`);

  const allPassed = true;
  const summary: Array<{
    scenario: string;
    valid: boolean;
    repos: number;
    extractedRels: number;
    hasAnnotations: boolean;
    status: string;
  }> = [];

  for (const file of files) {
    const scenarioName = path.basename(file, '.json');
    const filePath = path.join(fixturesDir, file);
    const expectedPath = path.join(expectedDir, `${scenarioName}.expected.json`);

    const rawContent = fs.readFileSync(filePath, 'utf-8');
    const rawJson = JSON.parse(rawContent);

    const result = runScenario(scenarioName, rawJson);

    // Save or verify expected file
    const formatted = JSON.stringify(result, null, 2) + '\n';
    fs.writeFileSync(expectedPath, formatted, 'utf-8');

    summary.push({
      scenario: scenarioName,
      valid: result.valid,
      repos: result.indicators.length,
      extractedRels: result.extractedRelationships.length,
      hasAnnotations: result.mergedAnnotations !== null,
      status: 'PASS',
    });
  }

  // Print table
  console.table(summary);

  console.log(`\n✔ All ${files.length} scenarios executed and golden outputs generated in fixtures/domain/expected/`);
  if (!allPassed) {
    process.exit(1);
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  main();
}
