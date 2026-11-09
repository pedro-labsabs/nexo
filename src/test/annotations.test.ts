import { describe, it, expect } from 'vitest';
import {
  mergePotentialAssessments,
  mergeManualRelationships,
  mergeUserAnnotations,
  getRelationshipLogicalKey,
} from '../shared/annotations.js';
import { UserAnnotations, Relationship, PotentialAssessment } from '../shared/types.js';

describe('Deterministic Annotations Merging & Conflict Resolution', () => {
  it('correctly builds stable directed logical keys (source::target::type)', () => {
    const key = getRelationshipLogicalKey({
      source: '1001',
      target: '1002',
      type: 'custom',
    });
    expect(key).toBe('1001::1002::custom');

    const inverseKey = getRelationshipLogicalKey({
      source: '1002',
      target: '1001',
      type: 'custom',
    });
    expect(inverseKey).not.toBe(key);
  });

  describe('mergePotentialAssessments (LWW)', () => {
    it('incoming wins when updatedAt is strictly newer', () => {
      const local: Record<string, PotentialAssessment> = {
        '1001': {
          repositoryId: 1001,
          strategicValue: 'medium',
          commercialPotential: 'experimental',
          updatedAt: '2026-01-01T10:00:00Z',
          tags: ['local'],
        },
      };
      const incoming: Record<string, PotentialAssessment> = {
        '1001': {
          repositoryId: 1001,
          strategicValue: 'critical',
          commercialPotential: 'production_ready',
          updatedAt: '2026-01-02T10:00:00Z',
          tags: ['incoming'],
        },
      };

      const merged = mergePotentialAssessments(local, incoming);
      expect(merged['1001'].strategicValue).toBe('critical');
      expect(merged['1001'].tags).toContain('incoming');
    });

    it('local wins on timestamp tie or when local is strictly newer', () => {
      const local: Record<string, PotentialAssessment> = {
        '1001': {
          repositoryId: 1001,
          strategicValue: 'high',
          commercialPotential: 'viable',
          updatedAt: '2026-01-02T10:00:00Z',
          notes: 'local version',
          tags: [],
        },
      };
      const incomingTie: Record<string, PotentialAssessment> = {
        '1001': {
          repositoryId: 1001,
          strategicValue: 'low',
          commercialPotential: 'none',
          updatedAt: '2026-01-02T10:00:00Z',
          notes: 'incoming tie',
          tags: [],
        },
      };

      const mergedTie = mergePotentialAssessments(local, incomingTie);
      expect(mergedTie['1001'].strategicValue).toBe('high');
      expect(mergedTie['1001'].notes).toBe('local version');
    });

    it('handles invalid timestamps deterministically', () => {
      const local: Record<string, PotentialAssessment> = {
        '1001': {
          repositoryId: 1001,
          strategicValue: 'high',
          commercialPotential: 'viable',
          updatedAt: 'invalid-date',
          tags: [],
        },
      };
      const incoming: Record<string, PotentialAssessment> = {
        '1001': {
          repositoryId: 1001,
          strategicValue: 'critical',
          commercialPotential: 'production_ready',
          updatedAt: '2026-01-01T00:00:00Z',
          tags: [],
        },
      };

      const merged = mergePotentialAssessments(local, incoming);
      expect(merged['1001'].strategicValue).toBe('critical');
    });
  });

  describe('mergeManualRelationships & Tombstones', () => {
    it('prevents duplicate relationships for identical logical keys', () => {
      const rel1: Relationship = {
        id: 'rel_1',
        source: '1',
        target: '2',
        sourceType: 'repository',
        targetType: 'repository',
        type: 'custom',
        provenance: 'manual',
        verified: false,
        createdAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-01T00:00:00Z',
        description: 'First version',
      };
      const rel2: Relationship = {
        id: 'rel_1',
        source: '1',
        target: '2',
        sourceType: 'repository',
        targetType: 'repository',
        type: 'custom',
        provenance: 'manual',
        verified: false,
        createdAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-02T00:00:00Z',
        description: 'Updated description',
      };

      const { mergedRelationships } = mergeManualRelationships([rel1], [rel2], [], []);
      expect(mergedRelationships.length).toBe(1);
      expect(mergedRelationships[0].description).toBe('Updated description');
    });

    it('respects tombstones and does not resurrect deleted relationships upon import', () => {
      const rel1: Relationship = {
        id: 'rel_1',
        source: '1',
        target: '2',
        sourceType: 'repository',
        targetType: 'repository',
        type: 'custom',
        provenance: 'manual',
        verified: false,
        createdAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-01T00:00:00Z',
      };

      const tombstone = {
        id: 'rel_1',
        source: '1',
        target: '2',
        type: 'custom' as const,
        deletedAt: '2026-01-02T00:00:00Z',
      };

      // Local has tombstone, incoming has older rel1
      const { mergedRelationships, mergedTombstones } = mergeManualRelationships(
        [],
        [rel1],
        [tombstone],
        []
      );

      expect(mergedRelationships.length).toBe(0);
      expect(mergedTombstones.length).toBe(1);
    });

    it('re-creation strictly after deletion tombstone is allowed', () => {
      const tombstone = {
        id: 'rel_1',
        source: '1',
        target: '2',
        type: 'custom' as const,
        deletedAt: '2026-01-02T00:00:00Z',
      };

      const reCreatedRel: Relationship = {
        id: 'rel_recreated',
        source: '1',
        target: '2',
        sourceType: 'repository',
        targetType: 'repository',
        type: 'custom',
        provenance: 'manual',
        verified: false,
        createdAt: '2026-01-03T00:00:00Z',
        updatedAt: '2026-01-03T00:00:00Z',
        description: 'Recreated later',
      };

      const { mergedRelationships } = mergeManualRelationships(
        [reCreatedRel],
        [],
        [tombstone],
        []
      );

      expect(mergedRelationships.length).toBe(1);
      expect(mergedRelationships[0].description).toBe('Recreated later');
    });
  });

  describe('mergeUserAnnotations full merge', () => {
    it('produces deterministic output independent of argument order when one is superset', () => {
      const annoA: UserAnnotations = {
        schemaVersion: '1.0',
        exportedAt: '2026-01-01T00:00:00Z',
        authenticatedUser: 'alexdev',
        manualRelationships: [],
        deletedRelationships: [],
        potentialAssessments: {
          '1': {
            repositoryId: 1,
            strategicValue: 'medium',
            commercialPotential: 'experimental',
            updatedAt: '2026-01-01T00:00:00Z',
            tags: [],
          },
        },
      };

      const annoB: UserAnnotations = {
        schemaVersion: '1.0',
        exportedAt: '2026-01-02T00:00:00Z',
        authenticatedUser: 'alexdev',
        manualRelationships: [],
        deletedRelationships: [],
        potentialAssessments: {
          '1': {
            repositoryId: 1,
            strategicValue: 'critical',
            commercialPotential: 'production_ready',
            updatedAt: '2026-01-02T00:00:00Z',
            tags: [],
          },
        },
      };

      const res1 = mergeUserAnnotations(annoA, annoB);
      expect(res1.potentialAssessments['1'].strategicValue).toBe('critical');
    });
  });
});
