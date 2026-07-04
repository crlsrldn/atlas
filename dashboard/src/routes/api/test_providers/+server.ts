import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
  try {
    const body = await request.json();
    const { torbox_api_key } = body;

    const results: Record<string, unknown> = {
      torbox: null,
    };

    if (torbox_api_key) {
      try {
        const res = await fetch("https://api.torbox.app/v1/api/user/me", {
          headers: { "Authorization": `Bearer ${torbox_api_key}` },
        });
        if (res.ok) {
          const data = await res.json();
          if (data.success && data.data) {
            results.torbox = {
              valid: true,
              premium: data.data.plan > 0,
              expires_at: data.data.premium_expires_at || null,
            };
          } else {
            results.torbox = { valid: false };
          }
        } else {
          results.torbox = { valid: false };
        }
      } catch (_e) {
        results.torbox = { valid: false, error: "Network error" };
      }
    }

    return json(results);
  } catch (_e) {
    return json({ error: "Invalid request body" }, { status: 400 });
  }
};
