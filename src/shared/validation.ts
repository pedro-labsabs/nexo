import { Snapshot, SnapshotSchema, UserAnnotations, UserAnnotationsSchema } from './types.js';

export function validateSnapshot(data: unknown): { success: true; data: Snapshot } | { success: false; errors: string[] } {
  const result = SnapshotSchema.safeParse(data);
  if (result.success) {
    return { success: true, data: result.data };
  }

  const errors = result.error.errors.map((err) => `${err.path.join('.')}: ${err.message}`);
  return { success: false, errors };
}

export function validateAnnotations(data: unknown): { success: true; data: UserAnnotations } | { success: false; errors: string[] } {
  const result = UserAnnotationsSchema.safeParse(data);
  if (result.success) {
    return { success: true, data: result.data };
  }

  const errors = result.error.errors.map((err) => `${err.path.join('.')}: ${err.message}`);
  return { success: false, errors };
}
