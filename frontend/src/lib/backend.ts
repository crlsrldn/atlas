export const BACKEND_BASE_URL = import.meta.env.VITE_ATLAS_BACKEND_URL ?? 'http://127.0.0.1:3000';
export const ATLAS_USER_ID = import.meta.env.VITE_ATLAS_USER_ID ?? 'demo-user';

export class BackendUnavailableError extends Error {
  constructor() {
    super('Atlas backend is not reachable.');
    this.name = 'BackendUnavailableError';
  }
}

export async function backendFetch(path: string, init?: RequestInit) {
  const headers = new Headers(init?.headers);
  headers.set('x-atlas-user-id', ATLAS_USER_ID);

  try {
    return await fetch(`${BACKEND_BASE_URL}${path}`, {
      ...init,
      headers
    });
  } catch {
    throw new BackendUnavailableError();
  }
}

export async function checkBackendHealth() {
  const response = await backendFetch('/health');
  return response.ok;
}
