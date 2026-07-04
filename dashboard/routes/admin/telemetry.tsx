import { Handlers, PageProps } from "$fresh/server.ts";
import { getCookies } from "$std/http/cookie.ts";
import { createClient } from "@supabase/supabase-js";
import { getAdminSupabaseClient } from "../../utils/admin_supabase.ts";
import Navbar from "../../islands/Navbar.tsx";

interface UserStats {
  userId: string;
  totalRequests: number;
  successfulStreams: number;
  failedStreams: number;
  lastActive: string;
}

interface TelemetryData {
  stats: UserStats[];
  error?: string;
  needsLogin?: boolean;
  supabaseUrl: string;
  supabaseAnonKey: string;
  url: string;
}

export const handler: Handlers<TelemetryData> = {
  async GET(req, ctx) {
    const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL");
    const supabaseAnonKey = Deno.env.get("PUBLIC_SUPABASE_ANON_KEY");

    if (!supabaseUrl || !supabaseAnonKey) {
      return ctx.render({
        stats: [],
        error: "Supabase configuration missing.",
        supabaseUrl: "",
        supabaseAnonKey: "",
        url: req.url,
      });
    }

    const cookies = getCookies(req.headers);
    const token = cookies["sb-admin-token"];

    if (!token) {
      return new Response("", {
        status: 302,
        headers: { Location: "/admin" },
      });
    }

    const authClient = createClient(supabaseUrl, supabaseAnonKey, {
      auth: { persistSession: false },
    });
    const { data: { user }, error: authError } = await authClient.auth.getUser(
      token,
    );

    if (authError || !user) {
      return new Response("", {
        status: 302,
        headers: { Location: "/admin" },
      });
    }

    const supabase = getAdminSupabaseClient();
    if (!supabase) {
      return ctx.render({
        stats: [],
        error: "Admin client missing.",
        supabaseUrl,
        supabaseAnonKey,
        url: req.url,
      });
    }

    const { data: profile } = await supabase
      .from("profiles")
      .select("role")
      .eq("id", user.id)
      .single();

    if (!profile || profile.role !== "admin") {
      return new Response("", {
        status: 302,
        headers: { Location: "/admin" },
      });
    }

    // Fetch telemetry data
    const { data: telemetryEvents, error: telemetryError } = await supabase
      .from("telemetry")
      .select("event_type, event_data, created_at")
      .eq("event_type", "playback_started")
      .order("created_at", { ascending: false })
      .limit(5000);

    if (telemetryError) {
      return ctx.render({
        stats: [],
        error: "Failed to fetch telemetry data.",
        supabaseUrl,
        supabaseAnonKey,
        url: req.url,
      });
    }

    const userStatsMap = new Map<string, UserStats>();

    if (telemetryEvents) {
      for (const event of telemetryEvents) {
        const userId = event.event_data?.user_id || "Anonymous";
        const success = event.event_data?.success === true;
        const createdAt = event.created_at;

        if (!userStatsMap.has(userId)) {
          userStatsMap.set(userId, {
            userId,
            totalRequests: 0,
            successfulStreams: 0,
            failedStreams: 0,
            lastActive: createdAt,
          });
        }

        const stats = userStatsMap.get(userId)!;
        stats.totalRequests++;
        if (success) {
          stats.successfulStreams++;
        } else {
          stats.failedStreams++;
        }

        // Update lastActive if this event is more recent
        if (new Date(createdAt) > new Date(stats.lastActive)) {
          stats.lastActive = createdAt;
        }
      }
    }

    const sortedStats = Array.from(userStatsMap.values()).sort((a, b) =>
      b.totalRequests - a.totalRequests
    );

    return ctx.render({
      stats: sortedStats,
      supabaseUrl,
      supabaseAnonKey,
      url: req.url,
    });
  },
};

export default function TelemetryDashboard({ data }: PageProps<TelemetryData>) {
  const url = new URL(data.url || "http://localhost");
  return (
    <div class="min-h-screen bg-zinc-50 dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100 font-sans selection:bg-indigo-500/30">
      <Navbar
        pathname={url.pathname}
        supabaseUrl={data.supabaseUrl}
        supabaseAnonKey={data.supabaseAnonKey}
      />

      <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div class="mb-10 flex flex-col md:flex-row md:items-end justify-between gap-4">
          <div>
            <h1 class="text-4xl font-extrabold tracking-tight text-zinc-900 dark:text-white mb-2">
              Advanced Telemetry
            </h1>
            <p class="text-lg text-zinc-500 dark:text-zinc-400 max-w-2xl">
              Deep dive into platform usage metrics and user streaming behavior.
            </p>
          </div>
          <a
            href="/admin"
            class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium transition-all duration-200 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800 shadow-sm"
          >
            &larr; Back to Admin Console
          </a>
        </div>

        {data.error && (
          <div class="mb-8 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800/30 rounded-2xl text-red-600 dark:text-red-400">
            {data.error}
          </div>
        )}

        <div class="glass-card rounded-3xl overflow-hidden">
          <div class="p-6 border-b border-black/5 dark:border-white/[0.04]">
            <h2 class="text-xl font-bold text-zinc-900 dark:text-white flex items-center gap-2">
              <svg
                class="w-5 h-5 text-indigo-500"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
                />
              </svg>
              Streams Resolved per User
            </h2>
          </div>

          <div class="overflow-x-auto">
            <table class="w-full text-left text-sm whitespace-nowrap">
              <thead class="bg-zinc-100/50 dark:bg-zinc-800/50 text-zinc-500 dark:text-zinc-400">
                <tr>
                  <th class="px-6 py-4 font-semibold">User ID</th>
                  <th class="px-6 py-4 font-semibold text-right">
                    Total Requests
                  </th>
                  <th class="px-6 py-4 font-semibold text-right">
                    Successful Streams
                  </th>
                  <th class="px-6 py-4 font-semibold text-right">
                    Failed Streams
                  </th>
                  <th class="px-6 py-4 font-semibold text-right">
                    Success Rate
                  </th>
                  <th class="px-6 py-4 font-semibold text-right">
                    Last Active
                  </th>
                </tr>
              </thead>
              <tbody class="divide-y divide-black/5 dark:divide-white/[0.04]">
                {data.stats.length === 0
                  ? (
                    <tr>
                      <td
                        colSpan={6}
                        class="px-6 py-8 text-center text-zinc-500 dark:text-zinc-400"
                      >
                        No telemetry data found.
                      </td>
                    </tr>
                  )
                  : (
                    data.stats.map((stat) => {
                      const successRate = stat.totalRequests > 0
                        ? ((stat.successfulStreams / stat.totalRequests) * 100)
                          .toFixed(1)
                        : "0.0";

                      return (
                        <tr
                          key={stat.userId}
                          class="hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors"
                        >
                          <td class="px-6 py-4 font-mono text-xs text-zinc-600 dark:text-zinc-300">
                            {stat.userId.length > 20
                              ? stat.userId.substring(0, 20) + "..."
                              : stat.userId}
                          </td>
                          <td class="px-6 py-4 text-right font-medium text-zinc-900 dark:text-white">
                            {stat.totalRequests.toLocaleString()}
                          </td>
                          <td class="px-6 py-4 text-right text-emerald-600 dark:text-emerald-400">
                            {stat.successfulStreams.toLocaleString()}
                          </td>
                          <td class="px-6 py-4 text-right text-red-600 dark:text-red-400">
                            {stat.failedStreams.toLocaleString()}
                          </td>
                          <td class="px-6 py-4 text-right">
                            <span
                              class={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                                parseFloat(successRate) >= 90
                                  ? "bg-emerald-100 text-emerald-800 dark:bg-emerald-400/10 dark:text-emerald-400"
                                  : parseFloat(successRate) >= 70
                                  ? "bg-amber-100 text-amber-800 dark:bg-amber-400/10 dark:text-amber-400"
                                  : "bg-red-100 text-red-800 dark:bg-red-400/10 dark:text-red-400"
                              }`}
                            >
                              {successRate}%
                            </span>
                          </td>
                          <td class="px-6 py-4 text-right text-zinc-500 dark:text-zinc-400">
                            {new Date(stat.lastActive).toLocaleDateString()}
                            {" "}
                            {new Date(stat.lastActive).toLocaleTimeString([], {
                              hour: "2-digit",
                              minute: "2-digit",
                            })}
                          </td>
                        </tr>
                      );
                    })
                  )}
              </tbody>
            </table>
          </div>
        </div>
      </main>
    </div>
  );
}
