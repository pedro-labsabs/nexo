import { describe, it, expect } from 'vitest';
import { validateSnapshot, validateAnnotations } from '../shared/validation.js';
import { createDemoSnapshot } from '../demo/generateDemoData.js';

describe('Zod Validation and Integrity', () => {
  it('validates the complete generated demo snapshot successfully', () => {
    const demo = createDemoSnapshot();
    const result = validateSnapshot(demo);
    expect(result.success).toBe(true);
  });

  it('rejects snapshots missing mandatory metadata fields', () => {
    const invalid = {
      metadata: { version: '1.0' },
      organizations: [],
      repositories: [],
      relationships: [],
    };
    const result = validateSnapshot(invalid);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.errors.length).toBeGreaterThan(0);
    }
  });

  it('validates user annotations structure correctly', () => {
    const annotations = {
      schemaVersion: '1.0',
      exportedAt: new Date().toISOString(),
      manualRelationships: [],
      potentialAssessments: {
        '1001': {
          repositoryId: 1001,
          strategicValue: 'high',
          commercialPotential: 'viable',
          updatedAt: new Date().toISOString(),
          tags: ['test'],
        },
      },
    };

    const result = validateAnnotations(annotations);
    expect(result.success).toBe(true);
  });
});
