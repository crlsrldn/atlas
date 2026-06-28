import { Handlers } from "$fresh/server.ts";

export const handler: Handlers = {
  async POST(req) {
    try {
      const body = await req.json();
      const { torbox_api_key, real_debrid_api_key } = body;

      const results: any = {
        torbox: null,
        real_debrid: null
      };

      if (torbox_api_key) {
        try {
          const res = await fetch("https://api.torbox.app/v1/api/user/me", {
            headers: { "Authorization": `Bearer ${torbox_api_key}` }
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
        } catch (e) {
          results.torbox = { valid: false, error: "Network error" };
        }
      }

      if (real_debrid_api_key) {
        try {
          const res = await fetch("https://api.real-debrid.com/rest/1.0/user", {
            headers: { "Authorization": `Bearer ${real_debrid_api_key}` }
          });
          if (res.ok) {
            const data = await res.json();
            results.real_debrid = {
              valid: true,
              premium: data.type === "premium",
              expires_at: data.expiration || null,
            };
          } else {
            results.real_debrid = { valid: false };
          }
        } catch (e) {
          results.real_debrid = { valid: false, error: "Network error" };
        }
      }

      return new Response(JSON.stringify(results), {
        headers: { "Content-Type": "application/json" }
      });
    } catch (e) {
      return new Response(JSON.stringify({ error: "Invalid request body" }), {
        status: 400,
        headers: { "Content-Type": "application/json" }
      });
    }
  }
};
