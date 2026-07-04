import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { env as publicEnv } from '$env/dynamic/public';
import { createClient } from '@supabase/supabase-js';

export const load: PageServerLoad = async ({ cookies, url }) => {
  const supabaseUrl = publicEnv.PUBLIC_SUPABASE_URL;
  const supabaseAnonKey = publicEnv.PUBLIC_SUPABASE_ANON_KEY;
  const serviceRoleKey = env.SUPABASE_SERVICE_ROLE_KEY;

  if (!supabaseUrl || !supabaseAnonKey) {
    return {
      error: "Supabase configuration missing in environment.",
      needsLogin: false
    };
  }

  const token = cookies.get('sb-admin-token');

  if (!token) {
    return {
      needsLogin: true
    };
  }

  // Verify token
  const authClient = createClient(supabaseUrl, supabaseAnonKey, {
    auth: { persistSession: false },
  });
  
  const { data: { user }, error: authError } = await authClient.auth.getUser(token);

  if (authError || !user || !user.email) {
    return {
      needsLogin: true
    };
  }

  if (!serviceRoleKey) {
    return {
      error: "Supabase service role key not configured. Cannot load live stats."
    };
  }

  const supabase = createClient(supabaseUrl, serviceRoleKey, {
    auth: { persistSession: false },
  });

  // Verify user is an admin
  const { data: profile, error: profileError } = await supabase
    .from("profiles")
    .select("role")
    .eq("id", user.id)
    .single();

  if (profileError || !profile || profile.role !== "admin") {
    return {
      error: "Access denied. Administrator privileges required.",
      needsLogin: true
    };
  }

  try {
    const { count: totalUsers } = await supabase
      .from("preferences")
      .select("*", { count: "exact", head: true });

    const { count: streamsResolved } = await supabase
      .from("telemetry")
      .select("*", { count: "exact", head: true })
      .eq("event_type", "playback_started");

    // Fetch last 1000 telemetry events
    const { data: telemetryEvents } = await supabase
      .from("telemetry")
      .select("event_type, event_data, created_at")
      .order("created_at", { ascending: false })
      .limit(1000);

    let playbackTotal = 0;
    let playbackSuccess = 0;
    let latencyTotal = 0;
    let latencyCount = 0;

    const providerLatestHealth = new Map<string, { healthy: boolean; latency_ms: number | null }>();

    if (telemetryEvents) {
      for (const event of telemetryEvents) {
        if (event.event_type === "playback_started") {
          playbackTotal++;
          if (event.event_data && event.event_data.success) {
            playbackSuccess++;
          }
        } else if (event.event_type === "provider_health") {
          const pName = event.event_data?.provider;
          const isHealthy = event.event_data?.healthy;
          const latencyMs = event.event_data?.latency_ms;

          if (pName && !providerLatestHealth.has(pName)) {
            providerLatestHealth.set(pName, {
              healthy: isHealthy,
              latency_ms: latencyMs || null,
            });
          }

          if (typeof latencyMs === "number" && latencyMs > 0) {
            latencyTotal += latencyMs;
            latencyCount++;
          }
        }
      }
    }

    const successRate = playbackTotal > 0
      ? `${((playbackSuccess / playbackTotal) * 100).toFixed(1)}%`
      : "—";

    const avgLatency = latencyCount > 0
      ? `${(latencyTotal / latencyCount).toFixed(0)}ms`
      : "—";

    const providers = ["TorBox"].map((name) => {
      const id = name.toLowerCase().replace(" ", "");
      const health = providerLatestHealth.get(id);

      if (!health) {
        return { name, status: "Unknown", color: "amber", latencyMs: null };
      }

      return {
        name,
        status: health.healthy ? "Operational" : "Degraded",
        color: health.healthy ? "emerald" : "amber",
        latencyMs: health.latency_ms,
      };
    });

    return {
      totalUsers: totalUsers || 0,
      streamsResolved: streamsResolved || 0,
      successRate,
      avgLatency,
      providers,
    };
  } catch (err) {
    console.error("Failed to load admin stats:", err);
    return {
      error: "Failed to fetch live stats from the database.",
    };
  }
};

export const actions: Actions = {
  logout: async ({ cookies }) => {
    cookies.delete('sb-admin-token', { path: '/' });
    return { success: true };
  }
};
