import { Handlers } from "$fresh/server.ts";
import { getCookies } from "$std/http/cookie.ts";
import { getAdminSupabaseClient } from "../../utils/admin_supabase.ts";

export const handler: Handlers = {
  async GET(req) {
    const kv = await Deno.openKv();
    const config = await kv.get(["global_config"]);
    
    // For GET, we only return monetization_enabled, not the secure Stripe keys
    // UNLESS the request is authenticated as an admin.
    
    const cookies = getCookies(req.headers);
    const token = cookies["sb-admin-token"];
    let isAdmin = false;
    
    if (token) {
      const supabase = getAdminSupabaseClient();
      if (supabase) {
        const { data: { user }, error: authError } = await supabase.auth.getUser(token);
        if (!authError && user) {
          const { data: profile } = await supabase.from("profiles").select("role").eq("id", user.id).single();
          if (profile?.role === "admin") {
            isAdmin = true;
          }
        }
      }
    }
    
    const value = config.value as Record<string, unknown> || {
      monetization_enabled: false,
    };
    
    if (!isAdmin) {
      return new Response(JSON.stringify({
        monetization_enabled: value.monetization_enabled === true,
      }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    return new Response(JSON.stringify(value), {
      headers: { "Content-Type": "application/json" }
    });
  },
  
  async POST(req) {
    // Only admins can update the global config
    const cookies = getCookies(req.headers);
    const token = cookies["sb-admin-token"];
    
    if (!token) {
      return new Response("Unauthorized", { status: 401 });
    }
    
    const supabase = getAdminSupabaseClient();
    if (!supabase) {
      return new Response("Supabase missing", { status: 500 });
    }
    
    const { data: { user }, error: authError } = await supabase.auth.getUser(token);
    if (authError || !user) {
      return new Response("Unauthorized", { status: 401 });
    }
    
    const { data: profile } = await supabase.from("profiles").select("role").eq("id", user.id).single();
    if (profile?.role !== "admin") {
      return new Response("Forbidden", { status: 403 });
    }
    
    try {
      const body = await req.json();
      const kv = await Deno.openKv();
      
      const currentConfig = (await kv.get(["global_config"])).value as Record<string, unknown> || {};
      
      const newConfig = {
        ...currentConfig,
        ...body,
      };
      
      await kv.set(["global_config"], newConfig);
      
      return new Response(JSON.stringify({ success: true }), {
        headers: { "Content-Type": "application/json" }
      });
    } catch (_e) {
      return new Response("Bad Request", { status: 400 });
    }
  }
};
