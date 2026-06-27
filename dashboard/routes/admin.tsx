import { Handlers, PageProps } from "$fresh/server.ts";
import { getCookies } from "$std/http/cookie.ts";
import { createClient } from "@supabase/supabase-js";
import { getAdminSupabaseClient } from "../utils/admin_supabase.ts";
import AdminLogin from "../islands/AdminLogin.tsx";

interface AdminData {
  totalUsers: number;
  streamsResolved: number;
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
        error: "Supabase configuration missing in environment." 
      });
    }

    const cookies = getCookies(req.headers);
    const token = cookies["sb-admin-token"];

    if (!token) {
      return ctx.render({ totalUsers: 0, streamsResolved: 0, needsLogin: true, supabaseUrl, supabaseAnonKey });
    }

    // Verify token using Supabase Auth
    const authClient = createClient(supabaseUrl, supabaseAnonKey, { auth: { persistSession: false } });
    const { data: { user }, error: authError } = await authClient.auth.getUser(token);

    // If there is an auth error, or the user is not found, or they don't have an email (they are an anonymous user)
    // then they are not an admin.
    if (authError || !user || !user.email) {
      return ctx.render({ totalUsers: 0, streamsResolved: 0, needsLogin: true, supabaseUrl, supabaseAnonKey });
    }

    const supabase = getAdminSupabaseClient();
    if (!supabase) {
      return ctx.render({ 
        totalUsers: 0, 
        streamsResolved: 0, 
        error: "Supabase service role key not configured. Cannot load live stats." 
      });
    }

    try {
      // Fetch total users (count rows in preferences table)
      const { count: totalUsers, error: usersError } = await supabase
        .from("preferences")
        .select("*", { count: "exact", head: true });

      if (usersError) throw usersError;

      // Fetch streams resolved (count rows in telemetry table where event_type = 'stream_resolved')
      const { count: streamsResolved, error: streamsError } = await supabase
        .from("telemetry")
        .select("*", { count: "exact", head: true })
        .eq("event_type", "stream_resolved");

      if (streamsError) throw streamsError;

      return ctx.render({
        totalUsers: totalUsers || 0,
        streamsResolved: streamsResolved || 0,
      });
    } catch (err) {
      console.error("Failed to load admin stats:", err);
      return ctx.render({
        totalUsers: 0,
        streamsResolved: 0,
        error: "Failed to fetch live stats from the database.",
      });
    }
  },
};

export default function AdminDashboard({ data }: PageProps<AdminData>) {
  if (data.needsLogin && data.supabaseUrl && data.supabaseAnonKey) {
    return (
      <div class="relative z-10 px-4 py-16 mx-auto min-h-screen flex flex-col items-center justify-center">
        <div class="mb-12 text-center w-full max-w-md mx-auto animate-fade-in-up">
          <a href="/" class="inline-block text-indigo-400 hover:text-indigo-300 font-medium mb-6 transition-colors">
            &larr; Back to Home
          </a>
          <h1 class="text-3xl font-extrabold text-white tracking-tight drop-shadow-sm">Atlas Admin</h1>
        </div>
        <AdminLogin supabaseUrl={data.supabaseUrl} supabaseAnonKey={data.supabaseAnonKey} />
      </div>
    );
  }

  return (
    <div class="relative z-10 px-4 py-16 mx-auto max-w-screen-md min-h-screen animate-fade-in-up">
      <div class="mb-12 text-center">
        <a href="/" class="inline-block text-indigo-400 hover:text-indigo-300 font-medium mb-6 transition-colors">
          &larr; Back to Home
        </a>
        <h1 class="text-4xl md:text-5xl font-extrabold text-white tracking-tight drop-shadow-sm">Admin Dashboard</h1>
        <p class="text-gray-400 mt-4 text-lg">System metrics and overview.</p>
      </div>

      {data.error && (
        <div class="mb-8 p-4 bg-red-500/10 border border-red-500/20 rounded-xl text-red-200">
          <p class="font-medium">Configuration Error</p>
          <p class="text-sm mt-1">{data.error}</p>
        </div>
      )}
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div class="bg-white/5 border border-white/10 p-8 rounded-2xl shadow-lg backdrop-blur-md">
          <div class="flex items-center gap-3 mb-4">
            <div class="p-2 bg-indigo-500/20 rounded-lg">
              <svg class="w-6 h-6 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
            </div>
            <h2 class="text-xl font-medium text-gray-300">Total Users</h2>
          </div>
          <p class="text-5xl font-extrabold text-white">
            {data.totalUsers.toLocaleString()}
          </p>
        </div>
        
        <div class="bg-white/5 border border-white/10 p-8 rounded-2xl shadow-lg backdrop-blur-md">
          <div class="flex items-center gap-3 mb-4">
            <div class="p-2 bg-purple-500/20 rounded-lg">
              <svg class="w-6 h-6 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
            <h2 class="text-xl font-medium text-gray-300">Streams Resolved</h2>
          </div>
          <p class="text-5xl font-extrabold text-white">
            {data.streamsResolved.toLocaleString()}
          </p>
        </div>
      </div>
    </div>
  );
}
