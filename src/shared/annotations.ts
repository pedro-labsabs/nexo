import {
  UserAnnotations,
  PotentialAssessment,
  Relationship,
  RelationshipTombstone,
} from './types.js';

/**
 * Validates whether a date string is a valid ISO timestamp.
 */
export function isValidDate(dateStr?: string | null): boolean {
  if (!dateStr || typeof dateStr !== 'string') return false;
  const time = Date.parse(dateStr);
  return !isNaN(time);
}

/**
 * Extracts a comparable timestamp from an ISO date string, falling back to 0 if invalid.
 */
function getTimestamp(dateStr?: string | null): number {
  if (!isValidDate(dateStr)) return 0;
  return new Date(dateStr!).getTime();
}

/**
 * Generates the stable directed logical key for a manual relationship: (source, target, type)
 */
export function getRelationshipLogicalKey(rel: Pick<Relationship, 'source' | 'target' | 'type'>): string {
  return `${rel.source}::${rel.target}::${rel.type}`;
}

/**
 * Merges two PotentialAssessment records deterministically using Last-Write-Wins (LWW).
 * In case of timestamp tie or invalid timestamp, the local assessment takes precedence.
 */
export function mergePotentialAssessments(
  localAssessments: Record<string, PotentialAssessment> = {},
  incomingAssessments: Record<string, PotentialAssessment> = {}
): Record<string, PotentialAssessment> {
  const merged: Record<string, PotentialAssessment> = { ...localAssessments };

  for (const [key, incoming] of Object.entries(incomingAssessments)) {
    const local = localAssessments[key];
    if (!local) {
      merged[key] = incoming;
      continue;
    }

    const localTime = getTimestamp(local.updatedAt);
    const incomingTime = getTimestamp(incoming.updatedAt);

    // If incoming is strictly newer than local, incoming wins. Otherwise local wins (tie goes to local).
    if (incomingTime > localTime) {
      merged[key] = incoming;
    } else {
      merged[key] = local;
    }
  }

  return merged;
}

/**
 * Merges manual relationships deterministically using:
 * 1. Stable identifier `id` and logical directed tuple `(source, target, type)`
 * 2. Tombstones (`deletedRelationships`) to prevent resurrection of deleted relations
 * 3. `updatedAt` (or `createdAt` if updatedAt is missing) using Last-Write-Wins, with ties favoring local
 */
export function mergeManualRelationships(
  localRelationships: Relationship[] = [],
  incomingRelationships: Relationship[] = [],
  localTombstones: RelationshipTombstone[] = [],
  incomingTombstones: RelationshipTombstone[] = []
): {
  mergedRelationships: Relationship[];
  mergedTombstones: RelationshipTombstone[];
} {
  // 1. Merge tombstones deterministically by logical key and id (LWW)
  const tombstoneMap = new Map<string, RelationshipTombstone>();

  const processTombstone = (t: RelationshipTombstone) => {
    const key = getRelationshipLogicalKey(t);
    const existing = tombstoneMap.get(key);
    if (!existing) {
      tombstoneMap.set(key, t);
      return;
    }
    const existingTime = getTimestamp(existing.deletedAt);
    const incomingTime = getTimestamp(t.deletedAt);
    if (incomingTime > existingTime) {
      tombstoneMap.set(key, t);
    }
  };

  for (const t of localTombstones) processTombstone(t);
  for (const t of incomingTombstones) processTombstone(t);

  // 2. Index active relationships by logical key (and stable id)
  const relationMap = new Map<string, Relationship>();

  // Helper to determine last modified time
  const getModTime = (r: Relationship): number => {
    return getTimestamp(r.updatedAt || r.createdAt);
  };

  // Add local relations first
  for (const r of localRelationships) {
    const key = getRelationshipLogicalKey(r);
    const tombstone = tombstoneMap.get(key);
    if (tombstone) {
      const deletedTime = getTimestamp(tombstone.deletedAt);
      const modTime = getModTime(r);
      if (deletedTime >= modTime) {
        // Relation was deleted
        continue;
      }
    }
    relationMap.set(key, r);
  }

  // Merge incoming relations
  for (const r of incomingRelationships) {
    const key = getRelationshipLogicalKey(r);
    const tombstone = tombstoneMap.get(key);
    const incomingModTime = getModTime(r);

    if (tombstone) {
      const deletedTime = getTimestamp(tombstone.deletedAt);
      if (deletedTime >= incomingModTime) {
        // Relation was deleted by tombstone
        continue;
      }
    }

    const localRel = relationMap.get(key);
    if (!localRel) {
      relationMap.set(key, r);
    } else {
      const localModTime = getModTime(localRel);
      // Strictly newer incoming wins; tie or older keeps local
      if (incomingModTime > localModTime) {
        relationMap.set(key, r);
      }
    }
  }

  // Deduplicate by ID as well (if an ID was used across different tuples, preserve latest)
  const idMap = new Map<string, Relationship>();
  for (const rel of relationMap.values()) {
    const existing = idMap.get(rel.id);
    if (!existing) {
      idMap.set(rel.id, rel);
    } else {
      if (getModTime(rel) > getModTime(existing)) {
        idMap.set(rel.id, rel);
      }
    }
  }

  return {
    mergedRelationships: Array.from(idMap.values()),
    mergedTombstones: Array.from(tombstoneMap.values()),
  };
}

/**
 * Deterministically merges two UserAnnotations objects.
 */
export function mergeUserAnnotations(
  local: UserAnnotations,
  incoming: UserAnnotations,
  exportedAtOverride?: string
): UserAnnotations {
  const mergedAssessments = mergePotentialAssessments(
    local.potentialAssessments,
    incoming.potentialAssessments
  );

  const { mergedRelationships, mergedTombstones } = mergeManualRelationships(
    local.manualRelationships || [],
    incoming.manualRelationships || [],
    local.deletedRelationships || [],
    incoming.deletedRelationships || []
  );

  return {
    schemaVersion: '1.0',
    exportedAt: exportedAtOverride || new Date().toISOString(),
    authenticatedUser: local.authenticatedUser || incoming.authenticatedUser,
    manualRelationships: mergedRelationships,
    deletedRelationships: mergedTombstones,
    potentialAssessments: mergedAssessments,
  };
}
