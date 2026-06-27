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
    <div class="w-full max-w-md bg-white/5 border border-white/10 p-8 rounded-3xl shadow-2xl backdrop-blur-md animate-fade-in-up mx-auto">
      <div class="text-center mb-8">
        <h2 class="text-2xl font-bold text-white mb-2">Admin Login</h2>
        <p class="text-gray-400 text-sm">Enter your Supabase administrator credentials</p>
      </div>

      <form onSubmit={handleLogin} class="space-y-6">
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Email Address</label>
          <input
            type="email"
            required
            value={email}
            onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
            class="w-full px-4 py-3 bg-black/20 border border-white/10 rounded-xl text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 transition-all"
            placeholder="admin@example.com"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Password</label>
          <input
            type="password"
            required
            value={password}
            onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
            class="w-full px-4 py-3 bg-black/20 border border-white/10 rounded-xl text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 transition-all"
            placeholder="••••••••"
          />
        </div>

        {error && (
          <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-200 text-sm">
            {error}
          </div>
        )}

        <button
          type="submit"
          disabled={loading}
          class="w-full py-4 px-4 bg-indigo-600 hover:bg-indigo-700 text-white font-bold rounded-xl shadow-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex justify-center items-center gap-2"
        >
          {loading ? (
            <>
              <span class="animate-spin h-5 w-5 border-2 border-white/30 border-t-white rounded-full"></span>
              Authenticating...
            </>
          ) : (
            "Log In"
          )}
        </button>
      </form>
    </div>
  );
}
