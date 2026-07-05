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

    let res4k = 0;
    let res1080p = 0;
    let res720p = 0;
    let resUnknown = 0;

    let latencyTimeline: { time: string; latency: number }[] = [];
    const providerLatestHealth = new Map<string, { healthy: boolean; latency_ms: number | null }>();
    
    // New aggregations
    const leaderboardMap = new Map<string, number>();
    const activeTokens15m = new Set<string>();
    let apiErrors = 0;
    const nowMs = Date.now();
    const fifteenMinsMs = 15 * 60 * 1000;

    if (telemetryEvents) {
      // Sort chronologically for timelines
      const chronologicalEvents = [...telemetryEvents].reverse();

      for (const event of chronologicalEvents) {
        if (event.event_type === "playback_started") {
          playbackTotal++;
          if (event.event_data && event.event_data.success) {
            playbackSuccess++;
          } else {
            apiErrors++;
          }
        } else if (event.event_type === "streams_requested") {
          const data = event.event_data;
          if (data) {
            if (data.stremio_id) {
              leaderboardMap.set(data.stremio_id, (leaderboardMap.get(data.stremio_id) || 0) + 1);
            }
            if (data.install_token && (nowMs - new Date(event.created_at).getTime()) <= fifteenMinsMs) {
              activeTokens15m.add(data.install_token);
            }
            if (data.resolution_distribution) {
              res4k += data.resolution_distribution["4k"] || 0;
              res1080p += data.resolution_distribution["1080p"] || 0;
              res720p += data.resolution_distribution["720p"] || 0;
              resUnknown += data.resolution_distribution.unknown || 0;
            }
            if (typeof data.latency_ms === "number" && data.latency_ms > 0) {
              latencyTimeline.push({
                time: new Date(event.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
                latency: data.latency_ms
              });
            }
          }
        } else if (event.event_type === "provider_health") {
          const pName = event.event_data?.provider?.toLowerCase().replace(" ", "");
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

    // Simplify latency timeline to max 20 points
    if (latencyTimeline.length > 20) {
      const step = Math.ceil(latencyTimeline.length / 20);
      latencyTimeline = latencyTimeline.filter((_, i) => i % step === 0).slice(-20);
    }

    const leaderboard = Array.from(leaderboardMap.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 10)
      .map(([id, count]) => ({ id, count }));

    return {
      totalUsers: totalUsers || 0,
      streamsResolved: streamsResolved || 0,
      successRate,
      avgLatency,
      providers,
      activeUsers15m: activeTokens15m.size,
      apiErrors,
      leaderboard,
      analytics: {
        playback: { total: playbackTotal, success: playbackSuccess, failure: playbackTotal - playbackSuccess },
        resolution: {
          '4K UHD': res4k,
          '1080p HD': res1080p,
          '720p HD': res720p,
          'Unknown': resUnknown
        },
        latencyTimeline
      }
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
