import { describe, it, expect } from 'vitest';
import { extractExplicitRelationships } from '../shared/relations.js';
import { Repository, Organization } from '../shared/types.js';

describe('Relationship Extraction Engine', () => {
  const orgs: Organization[] = [
    {
      id: 50,
      login: 'nexo-org',
      name: 'Nexo Org',
      description: null,
      repositoryCount: 2,
    },
  ];

  const repos: Repository[] = [
    {
      id: 1,
      nodeId: 'n1',
      name: 'core',
      fullName: 'nexo-org/core',
      owner: { login: 'nexo-org', id: 50, type: 'Organization' },
      isPrivate: false,
      htmlUrl: 'https://github.com/nexo-org/core',
      description: null,
      fork: false,
      createdAt: '2025-01-01T00:00:00Z',
      updatedAt: '2026-01-01T00:00:00Z',
      pushedAt: '2026-01-01T00:00:00Z',
      sizeKb: 100,
      stargazersCount: 0,
      watchersCount: 0,
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
        hasDocumentation: true,
        documentationEvidence: [],
        readmeFound: true,
        contributingFound: false,
        licenseFileFound: false,
        treeTruncated: false,
        treeInspected: true,
      },
    },
    {
      id: 2,
      nodeId: 'n2',
      name: 'extension',
      fullName: 'nexo-org/extension',
      owner: { login: 'nexo-org', id: 50, type: 'Organization' },
      isPrivate: false,
      htmlUrl: 'https://github.com/nexo-org/extension',
      description: null,
      fork: false,
      createdAt: '2025-01-01T00:00:00Z',
      updatedAt: '2026-01-01T00:00:00Z',
      pushedAt: '2026-01-01T00:00:00Z',
      sizeKb: 100,
      stargazersCount: 0,
      watchersCount: 0,
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
        hasDocumentation: true,
        documentationEvidence: [],
        readmeFound: true,
        contributingFound: false,
        licenseFileFound: false,
        treeTruncated: false,
        treeInspected: true,
      },
    },
  ];

  it('extracts belongs_to relationships for repositories belonging to organizations', () => {
    const rels = extractExplicitRelationships(repos, orgs, {});
    const belongsTo = rels.filter((r) => r.type === 'belongs_to');
    expect(belongsTo.length).toBe(2);
    expect(belongsTo[0].source).toBe('1');
    expect(belongsTo[0].target).toBe('50');
  });

  it('extracts explicit documentation references when mentioned in README', () => {
    const readmeMap = {
      2: 'Veja o projeto principal em https://github.com/nexo-org/core para maiores detalhes.',
    };

    const rels = extractExplicitRelationships(repos, orgs, readmeMap);
    const references = rels.filter((r) => r.type === 'references');
    expect(references.length).toBe(1);
    expect(references[0].source).toBe('2');
    expect(references[0].target).toBe('1');
    expect(references[0].verified).toBe(true);
  });

  it('does NOT fabricate relationships when repos merely share prefix or keywords', () => {
    const unrelatedRepo: Repository = {
      ...repos[0],
      id: 3,
      name: 'core-fake',
      fullName: 'other/core-fake',
    };
    const rels = extractExplicitRelationships([unrelatedRepo], [], {});
    expect(rels.length).toBe(0);
  });
});
