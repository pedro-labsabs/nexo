import { Snapshot, SnapshotSchema, UserAnnotations, UserAnnotationsSchema } from './types.js';
import { isValidDate } from './annotations.js';

export function validateSnapshot(data: unknown): { success: true; data: Snapshot } | { success: false; errors: string[] } {
  const result = SnapshotSchema.safeParse(data);
  if (!result.success) {
    const errors = result.error.errors.map((err) => `${err.path.join('.')}: ${err.message}`);
    return { success: false, errors };
  }

  const errors: string[] = [];
  const snapshot = result.data;

  if (!isValidDate(snapshot.metadata.createdAt)) {
    errors.push('metadata.createdAt: data inválida (esperado formato ISO 8601 / RFC 3339)');
  }
  if (!isValidDate(snapshot.metadata.referenceDate)) {
    errors.push('metadata.referenceDate: data inválida (esperado formato ISO 8601 / RFC 3339)');
  }
  if (snapshot.metadata.totalRepositories !== snapshot.repositories.length) {
    errors.push(
      `metadata.totalRepositories: total de repositórios diverge do tamanho da lista de repositórios (declarado: ${snapshot.metadata.totalRepositories}, encontrado: ${snapshot.repositories.length})`
    );
  }

  const seenRepoIds = new Set<number>();
  for (let i = 0; i < snapshot.repositories.length; i++) {
    const repo = snapshot.repositories[i];
    if (seenRepoIds.has(repo.id)) {
      errors.push(`repositories[${i}].id: ID de repositório duplicado: ${repo.id}`);
    }
    seenRepoIds.add(repo.id);

    if (!isValidDate(repo.createdAt)) {
      errors.push(`repositories[${i}].createdAt: data inválida (esperado formato ISO 8601 / RFC 3339)`);
    }
    if (!isValidDate(repo.updatedAt)) {
      errors.push(`repositories[${i}].updatedAt: data inválida (esperado formato ISO 8601 / RFC 3339)`);
    }
    if (!isValidDate(repo.pushedAt)) {
      errors.push(`repositories[${i}].pushedAt: data inválida (esperado formato ISO 8601 / RFC 3339)`);
    }
  }

  if (errors.length > 0) {
    return { success: false, errors };
  }

  return { success: true, data: snapshot };
}

export function validateAnnotations(data: unknown): { success: true; data: UserAnnotations } | { success: false; errors: string[] } {
  // Normalize missing deletedRelationships or default collections if validating raw JSON from previous versions
  const normalized = typeof data === 'object' && data !== null
    ? {
        manualRelationships: [],
        deletedRelationships: [],
        potentialAssessments: {},
        ...data,
      }
    : data;

  const result = UserAnnotationsSchema.safeParse(normalized);
  if (result.success) {
    return { success: true, data: result.data };
  }

  const errors = result.error.errors.map((err) => `${err.path.join('.')}: ${err.message}`);
  return { success: false, errors };
}
