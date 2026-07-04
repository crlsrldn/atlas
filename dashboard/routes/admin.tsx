import { Handlers, PageProps } from "$fresh/server.ts";
import { getCookies } from "$std/http/cookie.ts";
import { createClient } from "@supabase/supabase-js";
import { getAdminSupabaseClient } from "../utils/admin_supabase.ts";
import AdminLogin from "../islands/AdminLogin.tsx";
import AdminSettings from "../islands/AdminSettings.tsx";

interface ProviderHealth {
  name: string;
  status: string;
  color: string;
  latencyMs: number | null;
}

interface AdminData {
  totalUsers: number;
  streamsResolved: number;
  successRate: string;
  avgLatency: string;
  providers: ProviderHealth[];
  error?: string;
  needsLogin?: boolean;
  supabaseUrl?: string;
  supabaseAnonKey?: string;
}

export const handler: Handlers<AdminData> = {
  async GET(req, ctx) {
    const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL");
    const supabaseAnonKey = Deno.env.get("PUBLIC_SUPABASE_ANON_KEY");

    if (!supabaseUrl || !supabaseAnonKey) {
      return ctx.render({
        totalUsers: 0,
        streamsResolved: 0,
        successRate: "—",
        avgLatency: "—",
        providers: [],
        error: "Supabase configuration missing in environment.",
      });
    }

    const cookies = getCookies(req.headers);
    const token = cookies["sb-admin-token"];

    if (!token) {
      return ctx.render({
        totalUsers: 0,
        streamsResolved: 0,
        successRate: "—",
        avgLatency: "—",
        providers: [],
        needsLogin: true,
        supabaseUrl,
        supabaseAnonKey,
      });
    }

    // Verify token using Supabase Auth
    const authClient = createClient(supabaseUrl, supabaseAnonKey, {
      auth: { persistSession: false },
    });
    const { data: { user }, error: authError } = await authClient.auth.getUser(
      token,
    );

    if (authError || !user || !user.email) {
      return ctx.render({
        totalUsers: 0,
        streamsResolved: 0,
        successRate: "—",
        avgLatency: "—",
        providers: [],
        needsLogin: true,
        supabaseUrl,
        supabaseAnonKey,
      });
    }

    const supabase = getAdminSupabaseClient();
    if (!supabase) {
      return ctx.render({
        totalUsers: 0,
        streamsResolved: 0,
        successRate: "—",
        avgLatency: "—",
        providers: [],
        error:
          "Supabase service role key not configured. Cannot load live stats.",
      });
    }

    // Verify user is an admin
    const { data: profile, error: profileError } = await supabase
      .from("profiles")
      .select("role")
      .eq("id", user.id)
      .single();

    if (profileError || !profile || profile.role !== "admin") {
      return ctx.render({
        totalUsers: 0,
        streamsResolved: 0,
        successRate: "—",
        avgLatency: "—",
        providers: [],
        needsLogin: true,
        supabaseUrl,
        supabaseAnonKey,
        error: "Access denied. Administrator privileges required.",
      });
    }

    try {
      const { count: totalUsers, error: usersError } = await supabase
        .from("preferences")
        .select("*", { count: "exact", head: true });

      if (usersError) throw usersError;

      const { count: streamsResolved, error: streamsError } = await supabase
        .from("telemetry")
        .select("*", { count: "exact", head: true })
        .eq("event_type", "playback_started");

      if (streamsError) throw streamsError;

      // Fetch last 1000 telemetry events to compute advanced stats
      const { data: telemetryEvents, error: telemetryError } = await supabase
        .from("telemetry")
        .select("event_type, event_data, created_at")
        .order("created_at", { ascending: false })
        .limit(1000);

      if (telemetryError) throw telemetryError;

      let playbackTotal = 0;
      let playbackSuccess = 0;
      let latencyTotal = 0;
      let latencyCount = 0;

      // Map to store latest provider health
      const providerLatestHealth = new Map<
        string,
        { healthy: boolean; latency_ms: number | null }
      >();

      if (telemetryEvents) {
        // Since it's sorted descending, we iterate backwards or just rely on finding the first occurrence
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

      // Transform map into ProviderHealth array
      const providers: ProviderHealth[] = ["TorBox", "Real Debrid"].map(
        (name) => {
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
        },
      );

      return ctx.render({
        totalUsers: totalUsers || 0,
        streamsResolved: streamsResolved || 0,
        successRate,
        avgLatency,
        providers,
      });
    } catch (err) {
      console.error("Failed to load admin stats:", err);
      return ctx.render({
        totalUsers: 0,
        streamsResolved: 0,
        successRate: "—",
        avgLatency: "—",
        providers: [],
        error: "Failed to fetch live stats from the database.",
      });
    }
  },
  async POST(req, ctx) {
    const formData = await req.formData();
    if (formData.get("action") === "logout") {
      const res = new Response("", {
        status: 303,
        headers: {
          Location: "/admin",
        },
      });
      res.headers.append(
        "Set-Cookie",
        "sb-admin-token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax",
      );
      return res;
    }
    return ctx.render();
  },
};

// ─── Login View ────────────────────────────────────────────────────────────────
function LoginView(
  { supabaseUrl, supabaseAnonKey, error }: {
    supabaseUrl: string;
    supabaseAnonKey: string;
    error?: string;
  },
) {
  return (
    <div class="min-h-[80vh] flex flex-col items-center justify-center px-4 py-16 animate-fade-in-up">
      {/* Card */}
      <div class="w-full max-w-sm">
        {/* Header */}
        <div class="text-center mb-8 space-y-2">
          <div class="section-label mx-auto w-fit mb-4">Admin Access</div>
          <h1 class="text-3xl font-bold text-zinc-900 dark:text-white tracking-tight">
            System Console
          </h1>
          <p class="text-zinc-500 dark:text-zinc-400 text-sm">
            Sign in to view live Atlas Core metrics.
          </p>
        </div>

        {error && (
          <div class="alert alert-error mb-6">
            <div class="flex-1">{error}</div>
          </div>
        )}

        {/* Glass card */}
        <div class="glass-card-strong p-8 rounded-2xl">
          <AdminLogin
            supabaseUrl={supabaseUrl}
            supabaseAnonKey={supabaseAnonKey}
          />
        </div>
      </div>
    </div>
  );
}

// ─── Dashboard View ─────────────────────────────────────────────────────────────
function DashboardView({ data }: { data: AdminData }) {
  const metrics = [
    {
      label: "Total Users",
      value: data.totalUsers.toLocaleString(),
      change: "+12%",
      positive: true,
      color: "indigo",
      icon: (
        <svg
          class="w-5 h-5"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width={1.75}
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
          />
        </svg>
      ),
      description: "Registered subscribers",
    },
    {
      label: "Streams Resolved",
      value: data.streamsResolved.toLocaleString(),
      change: "+28%",
      positive: true,
      color: "purple",
      icon: (
        <svg
          class="w-5 h-5"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width={1.75}
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M13 10V3L4 14h7v7l9-11h-7z"
          />
        </svg>
      ),
      description: "Successful stream events",
    },
    {
      label: "Success Rate",
      value: data.successRate,
      change: "+0.3%",
      positive: true,
      color: "emerald",
      icon: (
        <svg
          class="w-5 h-5"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width={1.75}
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
          />
        </svg>
      ),
      description: "Stream resolution success",
    },
    {
      label: "Avg. Resolution",
      value: data.avgLatency,
      change: "−0.1s",
      positive: true,
      color: "amber",
      icon: (
        <svg
          class="w-5 h-5"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width={1.75}
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      ),
      description: "Source selection speed",
    },
  ];

  const colorMap: Record<
    string,
    { bg: string; icon: string; badge: string; glow: string }
  > = {
    indigo: {
      bg: "bg-indigo-500/10",
      icon: "text-indigo-400",
      badge: "text-indigo-300 bg-indigo-500/10 border-indigo-500/20",
      glow: "hover:border-indigo-500/30",
    },
    purple: {
      bg: "bg-purple-500/10",
      icon: "text-purple-400",
      badge: "text-purple-300 bg-purple-500/10 border-purple-500/20",
      glow: "hover:border-purple-500/30",
    },
    emerald: {
      bg: "bg-emerald-500/10",
      icon: "text-emerald-400",
      badge: "text-emerald-300 bg-emerald-500/10 border-emerald-500/20",
      glow: "hover:border-emerald-500/30",
    },
    amber: {
      bg: "bg-amber-500/10",
      icon: "text-amber-400",
      badge: "text-amber-300 bg-amber-500/10 border-amber-500/20",
      glow: "hover:border-amber-500/30",
    },
  };

  return (
    <div class="px-4 py-12 mx-auto w-full max-w-5xl animate-fade-in-up">
      {/* Page header */}
      <div class="mb-10 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div class="flex items-center gap-3">
          <div class="icon-box bg-indigo-500/10">
            <svg
              class="w-5 h-5 text-indigo-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width={1.75}
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
              />
            </svg>
          </div>
          <div>
            <p class="text-xs font-semibold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest mb-0.5">
              Admin
            </p>
            <h1 class="text-2xl md:text-3xl font-bold text-zinc-900 dark:text-white tracking-tight">
              System Overview
            </h1>
          </div>
        </div>

        <div class="flex items-center gap-3">
          {/* Live indicator */}
          <div class="flex items-center gap-2 px-4 py-2 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
            <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse shadow-[0_0_8px_rgba(52,211,153,0.8)]" />
            <span class="text-xs font-semibold text-emerald-300">
              Live Data
            </span>
          </div>
          {/* Last refreshed */}
          <span class="text-xs text-zinc-500 dark:text-zinc-400 hidden sm:block">
            Updated just now
          </span>
        </div>
      </div>

      {/* Error banner */}
      {data.error && (
        <div class="alert alert-error mb-8">
          <svg
            class="w-5 h-5 flex-shrink-0 mt-0.5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width={2}
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
          <div>
            <p class="font-semibold">Configuration Error</p>
            <p class="text-sm mt-0.5 opacity-80">{data.error}</p>
          </div>
        </div>
      )}

      {/* Metric cards grid */}
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5 mb-10">
        {metrics.map((m) => {
          const c = colorMap[m.color];
          return (
            <div
              key={m.label}
              class={`stat-card border border-black/5 dark:border-white/[0.07] transition-all duration-200 ${c.glow} hover:-translate-y-0.5`}
            >
              {/* Card header */}
              <div class="flex items-center justify-between mb-5">
                <div class={`icon-box icon-box-sm ${c.bg}`}>
                  <span class={c.icon}>{m.icon}</span>
                </div>
                <span class={`badge text-[10px] px-2 py-0.5 border ${c.badge}`}>
                  {m.change}
                </span>
              </div>

              {/* Big number */}
              <p class="text-4xl font-black text-zinc-900 dark:text-white tracking-tight tabular-nums">
                {m.value}
              </p>
              <p class="text-sm font-medium text-zinc-500 dark:text-zinc-400 mt-1">
                {m.label}
              </p>
              <p class="text-xs text-zinc-500 dark:text-zinc-600 mt-1">
                {m.description}
              </p>
            </div>
          );
        })}
      </div>

      {/* Secondary info section */}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
        {/* System Status */}
        <div class="glass-card-strong p-6 rounded-2xl">
          <div class="flex items-center gap-3 mb-5">
            <div class="icon-box icon-box-sm bg-emerald-500/10">
              <svg
                class="w-4 h-4 text-emerald-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width={2}
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M5.636 18.364a9 9 0 010-12.728m12.728 0a9 9 0 010 12.728m-9.9-2.829a5 5 0 010-7.07m7.072 0a5 5 0 010 7.07M13 12a1 1 0 11-2 0 1 1 0 012 0z"
                />
              </svg>
            </div>
            <h2 class="font-semibold text-zinc-900 dark:text-white">
              System Health
            </h2>
          </div>

          <div class="space-y-3">
            {[
              {
                name: "Atlas Gateway",
                status: "Operational",
                color: "emerald",
              },
              {
                name: "Supabase DB",
                status: data.error ? "Degraded" : "Operational",
                color: data.error ? "amber" : "emerald",
              },
              ...data.providers.map((p) => ({
                name: p.name,
                status: p.latencyMs
                  ? `${p.status} (${p.latencyMs}ms)`
                  : p.status,
                color: p.color,
              })),
            ].map((s) => (
              <div
                key={s.name}
                class="flex items-center justify-between py-2 border-b border-black/5 dark:border-white/[0.04] last:border-0"
              >
                <span class="text-sm text-zinc-700 dark:text-zinc-300">
                  {s.name}
                </span>
                <div class="flex items-center gap-2">
                  <span
                    class={`w-1.5 h-1.5 rounded-full ${
                      s.color === "emerald" ? "bg-emerald-400" : "bg-amber-400"
                    }`}
                  />
                  <span
                    class={`text-xs font-medium ${
                      s.color === "emerald"
                        ? "text-emerald-400"
                        : "text-amber-400"
                    }`}
                  >
                    {s.status}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Quick Actions */}
        <div class="glass-card-strong p-6 rounded-2xl">
          <div class="flex items-center gap-3 mb-5">
            <div class="icon-box icon-box-sm bg-indigo-500/10">
              <svg
                class="w-4 h-4 text-indigo-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width={2}
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"
                />
              </svg>
            </div>
            <h2 class="font-semibold text-zinc-900 dark:text-white">
              Quick Actions
            </h2>
          </div>

          <div class="space-y-3">
            {[
              {
                label: "View Supabase Dashboard",
                href: "https://app.supabase.com",
                icon: "↗",
              },
              {
                label: "Refresh Statistics",
                href: "/admin",
                icon: "↺",
              },
              {
                label: "View Gateway Logs",
                href: "#",
                icon: "📋",
              },
            ].map((action) => (
              <a
                key={action.label}
                href={action.href}
                target={action.href.startsWith("http") ? "_blank" : undefined}
                rel={action.href.startsWith("http")
                  ? "noopener noreferrer"
                  : undefined}
                class="flex items-center justify-between py-2.5 px-3 rounded-xl hover:bg-black/5 dark:hover:bg-white/[0.04] border border-transparent hover:border-black/5 dark:hover:border-white/[0.06] transition-all group"
              >
                <span class="text-sm text-zinc-700 group-hover:text-zinc-900 dark:text-zinc-300 dark:group-hover:text-white transition-colors">
                  {action.label}
                </span>
                <span class="text-zinc-500 dark:text-zinc-600 text-sm group-hover:text-zinc-700 dark:group-hover:text-zinc-300 transition-colors">
                  {action.icon}
                </span>
              </a>
            ))}
          </div>

          {/* Monetization & Limits */}
          <AdminSettings />

          {/* Sign out */}
          <div class="mt-5 pt-4 border-t border-black/5 dark:border-white/[0.06]">
            <form method="POST">
              <input type="hidden" name="action" value="logout" />
              <button
                type="submit"
                class="flex items-center justify-center gap-2 px-4 py-2 bg-zinc-200 dark:bg-zinc-800 hover:bg-zinc-300 dark:hover:bg-zinc-700 text-zinc-900 dark:text-white text-sm font-medium rounded-xl border border-black/5 dark:border-white/5 shadow-sm transition-all duration-200"
              >
                <svg
                  class="w-4 h-4 text-zinc-400"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  stroke-width={2}
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                  />
                </svg>
                Sign Out
              </button>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}

// ─── Page Entry ─────────────────────────────────────────────────────────────────
export default function AdminPage({ data }: PageProps<AdminData>) {
  if (data.needsLogin && data.supabaseUrl && data.supabaseAnonKey) {
    return (
      <LoginView
        supabaseUrl={data.supabaseUrl}
        supabaseAnonKey={data.supabaseAnonKey}
        error={data.error}
      />
    );
  }

  if (data.error && !data.needsLogin) {
    return (
      <div class="min-h-[80vh] flex flex-col items-center justify-center px-4">
        <div class="glass-card p-8 rounded-2xl max-w-md w-full text-center space-y-4 border-red-500/30">
          <h2 class="text-xl font-bold text-zinc-900 dark:text-white">
            Access Error
          </h2>
          <p class="text-zinc-500 dark:text-zinc-400">{data.error}</p>
        </div>
      </div>
    );
  }

  return <DashboardView data={data} />;
}
