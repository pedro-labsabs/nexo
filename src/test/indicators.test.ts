import { describe, it, expect } from 'vitest';
import { computeIndicators } from '../shared/indicators.js';
import { Repository } from '../shared/types.js';

describe('Deterministic Indicators Calculation', () => {
  const baseRepo: Repository = {
    id: 101,
    nodeId: 'node_101',
    name: 'sample-project',
    fullName: 'testorg/sample-project',
    owner: {
      login: 'testorg',
      id: 99,
      type: 'Organization',
    },
    isPrivate: false,
    htmlUrl: 'https://github.com/testorg/sample-project',
    description: 'Sample description for testing indicators',
    fork: false,
    createdAt: '2025-01-01T00:00:00Z',
    updatedAt: '2026-10-15T00:00:00Z',
    pushedAt: '2026-10-15T00:00:00Z',
    sizeKb: 5000,
    stargazersCount: 50,
    watchersCount: 10,
    openIssuesCount: 2,
    forksCount: 5,
    primaryLanguage: 'TypeScript',
    languages: { TypeScript: 5000 },
    archived: false,
    disabled: false,
    defaultBranch: 'main',
    affiliation: 'owner',
    treeSignals: {
      hasTests: true,
      testEvidence: ['tests/index.test.ts', 'vitest.config.ts'],
      hasCI: true,
      ciEvidence: ['.github/workflows/ci.yml'],
      hasDocumentation: true,
      documentationEvidence: ['docs/guide.md'],
      readmeFound: true,
      readmePath: 'README.md',
      contributingFound: true,
      licenseFileFound: true,
      packageJsonFound: false,
    isEmptyRepo: false,
    treeTruncated: false,
      treeInspected: true,
    },
  };

  const referenceDate = '2026-11-01T00:00:00Z'; // 17 days after last push

  it('computes passing scores for a repo with comprehensive signals', () => {
    const indicators = computeIndicators(baseRepo, referenceDate);

    expect(indicators.testInfrastructure.status).toBe('passed');
    expect(indicators.testInfrastructure.score).toBe(100);
    expect(indicators.ciConfiguration.status).toBe('passed');
    expect(indicators.documentation.status).toBe('passed');
    expect(indicators.activityRecency.status).toBe('passed');
    expect(indicators.overallQualityScore).toBeGreaterThanOrEqual(90);
  });

  it('marks indicators as unknown when the file tree was not inspected', () => {
    const repoUninspected: Repository = {
      ...baseRepo,
      treeSignals: {
        hasTests: false,
        testEvidence: [],
        hasCI: false,
        ciEvidence: [],
        hasDocumentation: false,
        documentationEvidence: [],
        readmeFound: false,
        contributingFound: false,
        licenseFileFound: false,
        packageJsonFound: false,
    isEmptyRepo: false,
    treeTruncated: false,
        treeInspected: false,
        inspectionError: '403 Forbidden',
      },
    };

    const indicators = computeIndicators(repoUninspected, referenceDate);
    expect(indicators.testInfrastructure.status).toBe('unknown');
    expect(indicators.ciConfiguration.status).toBe('unknown');
    expect(indicators.testInfrastructure.limitations).toContain(
      'Não foi presumido resultado negativo.'
    );
  });

  it('penalizes archived projects in maintenance signals deterministically', () => {
    const archivedRepo: Repository = {
      ...baseRepo,
      archived: true,
    };

    const indicators = computeIndicators(archivedRepo, referenceDate);
    expect(indicators.maintenanceSignals.status).toBe('failed');
    expect(indicators.maintenanceSignals.score).toBe(10);
  });
});

  it('marks indicators as unknown rather than failed when tree is truncated and signals are unobserved', () => {
    const truncatedRepo: Repository = {
      id: 999,
      nodeId: 'r999',
      name: 'large-repo',
      fullName: 'org/large-repo',
      owner: { login: 'org', id: 1, type: 'Organization' },
      isPrivate: false,
      htmlUrl: 'https://github.com/org/large-repo',
      description: 'Large repo',
      fork: false,
      createdAt: '2025-01-01T00:00:00Z',
      updatedAt: '2026-01-01T00:00:00Z',
      pushedAt: '2026-01-01T00:00:00Z',
      sizeKb: 50000,
      stargazersCount: 10,
      watchersCount: 1,
      openIssuesCount: 0,
      forksCount: 0,
      primaryLanguage: 'TypeScript',
      languages: {},
      archived: false,
      disabled: false,
      defaultBranch: 'main',
      affiliation: 'owner',
      treeSignals: {
        hasTests: false,
        testEvidence: [],
        hasCI: false,
        ciEvidence: [],
        hasDocumentation: false,
        documentationEvidence: [],
        readmeFound: false,
        contributingFound: false,
        licenseFileFound: false,
        packageJsonFound: false,
        isEmptyRepo: false,
        treeTruncated: true, // Truncated tree!
        treeInspected: true,
      },
    };

    const indicators = computeIndicators(truncatedRepo, '2026-01-02T00:00:00Z');
    expect(indicators.testInfrastructure.status).toBe('unknown');
    expect(indicators.testInfrastructure.summary).toContain('Inspeção parcial');
    expect(indicators.ciConfiguration.status).toBe('unknown');
    expect(indicators.ciConfiguration.summary).toContain('Inspeção parcial');
  });
