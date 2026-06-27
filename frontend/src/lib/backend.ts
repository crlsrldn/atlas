export const BACKEND_BASE_URL = import.meta.env.VITE_ATLAS_BACKEND_URL ?? 'http://127.0.0.1:3000';

export class BackendUnavailableError extends Error {
  constructor() {
    super('Atlas backend is not reachable.');
    this.name = 'BackendUnavailableError';
  }
}

export async function backendFetch(path: string, init?: RequestInit) {
  try {
    return await fetch(`${BACKEND_BASE_URL}${path}`, init);
  } catch {
    throw new BackendUnavailableError();
  }
}

export async function checkBackendHealth() {
  const response = await backendFetch('/health');
  return response.ok;
}
