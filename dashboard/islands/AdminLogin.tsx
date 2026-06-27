import { useState } from "preact/hooks";
import { createClient } from "@supabase/supabase-js";

interface AdminLoginProps {
  supabaseUrl: string;
  supabaseAnonKey: string;
}

export default function AdminLogin({ supabaseUrl, supabaseAnonKey }: AdminLoginProps) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleLogin = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const supabase = createClient(supabaseUrl, supabaseAnonKey, {
        auth: {
          persistSession: false,
        },
      });

      const { data, error } = await supabase.auth.signInWithPassword({
        email,
        password,
      });

      if (error) {
        throw error;
      }

      if (data.session) {
        // Set the session token in a cookie that the server can read
        document.cookie = `sb-admin-token=${data.session.access_token}; path=/; max-age=3600; samesite=lax`;
        
        // Reload the page so the SSR handler picks up the cookie
        window.location.reload();
      }
    } catch (err: any) {
      setError(err.message || "Failed to log in");
      setLoading(false);
    }
  };

  return (
    <div class="w-full max-w-md bg-[#09090b]/50 border border-white/10 p-8 rounded-2xl shadow-xl backdrop-blur-xl mx-auto">
      <div class="text-center mb-8">
        <h2 class="text-2xl font-semibold text-zinc-100 mb-2">Admin Login</h2>
        <p class="text-zinc-400 text-sm">Enter your Supabase administrator credentials</p>
      </div>

      <form onSubmit={handleLogin} class="space-y-6">
        <div class="space-y-5">
          <div>
            <label class="block text-sm font-medium text-zinc-300 mb-1.5">Email Address</label>
            <input
              type="email"
              required
              value={email}
              onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
              class="w-full px-4 py-2.5 bg-black/40 border border-white/10 rounded-xl text-white placeholder-zinc-600 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/50 transition-all text-sm"
              placeholder="admin@example.com"
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-zinc-300 mb-1.5">Password</label>
            <input
              type="password"
              required
              value={password}
              onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
              class="w-full px-4 py-2.5 bg-black/40 border border-white/10 rounded-xl text-white placeholder-zinc-600 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/50 transition-all text-sm"
              placeholder="••••••••"
            />
          </div>
        </div>

        {error && (
          <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-200 text-sm flex items-start gap-3">
            <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <div>
              <p class="font-semibold">Authentication Failed</p>
              <p class="mt-1 opacity-90">{error}</p>
            </div>
          </div>
        )}

        <button
          type="submit"
          disabled={loading}
          class="w-full relative group inline-flex items-center justify-center px-6 py-3 font-medium text-white bg-indigo-600 rounded-xl overflow-hidden transition-all duration-300 hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span class="relative flex items-center gap-2">
            {loading ? (
              <>
                <svg class="animate-spin h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Authenticating...
              </>
            ) : (
              "Log In"
            )}
          </span>
        </button>
      </form>
    </div>
  );
}
