import { useState } from "preact/hooks";
import { createClient } from "@supabase/supabase-js";

interface AdminLoginProps {
  supabaseUrl: string;
  supabaseAnonKey: string;
}

export default function AdminLogin(
  { supabaseUrl, supabaseAnonKey }: AdminLoginProps,
) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showPassword, setShowPassword] = useState(false);

  const handleLogin = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const supabase = createClient(supabaseUrl, supabaseAnonKey, {
        auth: { persistSession: false },
      });

      const { data, error } = await supabase.auth.signInWithPassword({
        email,
        password,
      });

      if (error) throw error;

      if (data.session) {
        document.cookie =
          `sb-admin-token=${data.session.access_token}; path=/; max-age=3600; samesite=lax`;
        window.location.reload();
      }
    } catch (err: any) {
      setError(err.message || "Failed to log in. Check your credentials.");
      setLoading(false);
    }
  };

  return (
    <div class="w-full">
      {/* Lock icon */}
      <div class="flex justify-center mb-8">
        <div class="relative">
          <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 border border-indigo-500/30 flex items-center justify-center">
            <svg class="w-8 h-8 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={1.5}>
              <path stroke-linecap="round" stroke-linejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z" />
            </svg>
          </div>
          <div class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full bg-[#09090b] border border-white/10 flex items-center justify-center">
            <span class="w-2.5 h-2.5 rounded-full bg-amber-400" />
          </div>
        </div>
      </div>

      <form onSubmit={handleLogin} class="space-y-5">
        {/* Email */}
        <div class="space-y-1.5">
          <label class="block text-sm font-medium text-zinc-300" for="admin-email">
            Email Address
          </label>
          <input
            id="admin-email"
            type="email"
            required
            value={email}
            onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
            class="input-field"
            placeholder="admin@example.com"
            autocomplete="email"
          />
        </div>

        {/* Password */}
        <div class="space-y-1.5">
          <label class="block text-sm font-medium text-zinc-300" for="admin-password">
            Password
          </label>
          <div class="relative">
            <input
              id="admin-password"
              type={showPassword ? "text" : "password"}
              required
              value={password}
              onInput={(e) =>
                setPassword((e.target as HTMLInputElement).value)}
              class="input-field pr-12"
              placeholder="••••••••"
              autocomplete="current-password"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              class="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300 transition-colors p-1 rounded-lg"
              aria-label={showPassword ? "Hide password" : "Show password"}
            >
              {showPassword
                ? (
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2}>
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                  </svg>
                )
                : (
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2}>
                    <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                )}
            </button>
          </div>
        </div>

        {/* Error */}
        {error && (
          <div class="alert alert-error animate-fade-in-down">
            <svg class="w-4 h-4 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2}>
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <div>
              <p class="font-semibold text-xs">Authentication Failed</p>
              <p class="mt-0.5 opacity-80 text-xs">{error}</p>
            </div>
          </div>
        )}

        {/* Submit */}
        <button
          type="submit"
          disabled={loading}
          class="btn-primary w-full text-base py-3.5 rounded-xl mt-2"
        >
          {loading
            ? (
              <>
                <svg class="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Signing in...
              </>
            )
            : (
              <>
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2.5}>
                  <path stroke-linecap="round" stroke-linejoin="round" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" />
                </svg>
                Sign In to Admin
              </>
            )}
        </button>
      </form>

      <p class="mt-6 text-center text-xs text-zinc-600">
        Only Supabase service accounts with admin privileges can log in.
      </p>
    </div>
  );
}
