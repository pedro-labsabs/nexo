import { UserAnnotations } from './types.js';
import { validateAnnotations } from './validation.js';

const STORAGE_KEY_PREFIX = 'nexo_annotations_';

/**
 * Derives a storage key scoped to user/portfolio identity.
 */
export function getStorageKey(userLogin?: string): string {
  const sanitized = (userLogin || 'default').trim().toLowerCase().replace(/[^a-z0-9_-]/g, '_');
  return `${STORAGE_KEY_PREFIX}${sanitized}`;
}

function getLocalStorage(): Storage | null {
  if (typeof globalThis !== 'undefined' && 'localStorage' in globalThis) {
    try {
      return (globalThis as unknown as { localStorage: Storage }).localStorage;
    } catch {
      return null;
    }
  }
  return null;
}

/**
 * Loads annotations scoped to the specified user from localStorage.
 */
export function loadScopedAnnotations(userLogin?: string): UserAnnotations | null {
  const storage = getLocalStorage();
  if (!storage) return null;

  try {
    const key = getStorageKey(userLogin);
    const raw = storage.getItem(key);
    if (!raw) return null;

    const parsed = JSON.parse(raw);
    const valid = validateAnnotations(parsed);
    if (valid.success) {
      return valid.data;
    }
    console.warn(`[NEXO Storage] Anotações locais corrompidas para chave ${key}:`, valid.errors);
    return null;
  } catch (err) {
    console.error(`[NEXO Storage] Erro ao carregar anotações locais:`, err);
    return null;
  }
}

/**
 * Saves annotations scoped to user in localStorage.
 * Returns { success: true } or { success: false, error: string } so caller can notify user.
 */
export function saveScopedAnnotations(
  annotations: UserAnnotations,
  userLogin?: string
): { success: boolean; error?: string } {
  const storage = getLocalStorage();
  if (!storage) {
    return { success: false, error: 'Armazenamento local indisponível no ambiente atual.' };
  }

  try {
    const key = getStorageKey(userLogin || annotations.authenticatedUser);
    const json = JSON.stringify(annotations);
    storage.setItem(key, json);
    return { success: true };
  } catch (err: any) {
    console.error('[NEXO Storage] Falha ao salvar no localStorage:', err);
    return {
      success: false,
      error: `Falha ao gravar no localStorage: ${err?.message || 'Limite de armazenamento excedido ou bloqueado'}. Faça backup manual via 'Exportar Anotações'.`,
    };
  }
}

/**
 * Removes scoped annotations explicitly from localStorage.
 */
export function clearScopedAnnotations(userLogin?: string): void {
  const storage = getLocalStorage();
  if (!storage) return;

  try {
    const key = getStorageKey(userLogin);
    storage.removeItem(key);
  } catch (err) {
    console.error('[NEXO Storage] Erro ao remover do localStorage:', err);
  }
}
