import { describe, it, expect } from 'vitest';
import { inspectRepositoryTree } from '../cli/sync.js';

describe('CLI Sync Tree Inspection & Error Handling', () => {
  it('handles tree inspection network/auth errors gracefully returning unknown state', async () => {
    const mockOctokit = {
      rest: {
        git: {
          getTree: async () => {
            throw new Error('403 Forbidden: Rate limit exceeded or access denied');
          },
        },
      },
    } as any;

    const signals = await inspectRepositoryTree(mockOctokit, 'owner', 'repo');
    expect(signals.treeInspected).toBe(false);
    expect(signals.hasTests).toBe(false);
    expect(signals.inspectionError).toContain('403 Forbidden');
  });

  it('correctly inspects tree items and extracts test/CI/documentation evidence', async () => {
    const mockOctokit = {
      rest: {
        git: {
          getTree: async () => ({
            data: {
              truncated: false,
              tree: [
                { path: 'README.md', type: 'blob' },
                { path: 'LICENSE', type: 'blob' },
                { path: 'tests/unit.test.ts', type: 'blob' },
                { path: '.github/workflows/ci.yml', type: 'blob' },
                { path: 'docs/architecture.md', type: 'blob' },
              ],
            },
          }),
        },
      },
    } as any;

    const signals = await inspectRepositoryTree(mockOctokit, 'owner', 'repo');
    expect(signals.treeInspected).toBe(true);
    expect(signals.treeTruncated).toBe(false);
    expect(signals.hasTests).toBe(true);
    expect(signals.testEvidence).toContain('tests/unit.test.ts');
    expect(signals.hasCI).toBe(true);
    expect(signals.ciEvidence).toContain('.github/workflows/ci.yml');
    expect(signals.readmeFound).toBe(true);
    expect(signals.licenseFileFound).toBe(true);
    expect(signals.hasDocumentation).toBe(true);
  });
});

  it('detects empty repository correctly and flags incomplete sync on truncated tree', async () => {
    const mockEmptyOctokit = {
      rest: {
        git: {
          getTree: async () => {
            const err: any = new Error('Git Repository is empty');
            err.status = 409;
            throw err;
          },
        },
      },
    } as any;

    const signals = await inspectRepositoryTree(mockEmptyOctokit, 'owner', 'empty-repo');
    expect(signals.treeInspected).toBe(false);
    expect(signals.isEmptyRepo).toBe(true);
  });

  it('detects package.json manifest and marks tree truncated', async () => {
    const mockTruncatedOctokit = {
      rest: {
        git: {
          getTree: async () => ({
            data: {
              truncated: true,
              tree: [
                { path: 'package.json', type: 'blob' },
              ],
            },
          }),
        },
      },
    } as any;

    const signals = await inspectRepositoryTree(mockTruncatedOctokit, 'owner', 'truncated-repo');
    expect(signals.treeInspected).toBe(true);
    expect(signals.treeTruncated).toBe(true);
    expect(signals.packageJsonFound).toBe(true);
  });
