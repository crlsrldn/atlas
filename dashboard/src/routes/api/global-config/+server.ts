import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async () => {
  // Simplifed global config since monetization is disabled for self-hosted TorBox service.
  return json({
    monetization_enabled: false,
  });
};
