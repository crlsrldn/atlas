import { useState } from "preact/hooks";
import { getSupabaseClient } from "../utils/supabase.ts";

interface AuthFormProps {
  type: "login" | "signup";
  supabaseUrl: string;
  supabaseAnonKey: string;
}

export default function AuthForm(
  { type, supabaseUrl, supabaseAnonKey }: AuthFormProps,
) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [showPassword, setShowPassword] = useState(false);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError("");
    setSuccess("");

    try {
      const supabase = getSupabaseClient(supabaseUrl, supabaseAnonKey);

      if (type === "signup") {
        const { data, error: signUpError } = await supabase.auth.signUp({
          email: email,
          password: password,
          options: {
            emailRedirectTo: `${globalThis.location.origin}/dashboard`,
          },
        });
        if (signUpError) throw signUpError;

        if (data.session) {
          // Auto-login if email confirmations are disabled
          globalThis.location.href = "/dashboard";
          return;
        }

        setSuccess("Check your email for the confirmation link.");
      } else {
        const { error: signInError } = await supabase.auth.signInWithPassword({
          email: email,
          password: password,
        });
        if (signInError) throw signInError;

        // On successful login, redirect to dashboard
        globalThis.location.href = "/dashboard";
      }
    } catch (err: unknown) {
      setError(
        (err as Error).message || "An error occurred during authentication.",
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="glass-card p-8 max-w-md w-full mx-auto animate-fade-in-up">
      <div class="text-center mb-8">
        <div class="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-indigo-500/10 text-indigo-400 mb-4">
          <svg
            class="w-6 h-6"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width={2}
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"
            />
          </svg>
        </div>
        <h2 class="text-2xl font-bold text-white">
          {type === "signup" ? "Create your account" : "Welcome back"}
        </h2>
        <p class="text-zinc-400 mt-2 text-sm">
          {type === "signup"
            ? "Start streaming with the best sources."
            : "Sign in to manage your configuration."}
        </p>
      </div>

      <form onSubmit={handleSubmit} class="space-y-5">
        {error && (
          <div class="alert alert-error">
            <svg
              class="w-4 h-4 mt-0.5 flex-shrink-0"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width={2}
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <div class="flex-1">{error}</div>
          </div>
        )}

        {success && (
          <div class="alert alert-success">
            <svg
              class="w-4 h-4 mt-0.5 flex-shrink-0"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width={2}
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <div class="flex-1">{success}</div>
          </div>
        )}

        <div class="space-y-1.5">
          <label class="block text-sm font-medium text-zinc-300">
            Email
          </label>
          <input
            type="email"
            class="input-field"
            placeholder="you@example.com"
            value={email}
            onInput={(e) => setEmail((e.target as HTMLInputElement).value)}
            required
            disabled={loading}
          />
        </div>

        <div class="space-y-1.5">
          <label class="block text-sm font-medium text-zinc-300">
            Password
          </label>
          <div class="relative">
            <input
              type={showPassword ? "text" : "password"}
              class="input-field pr-10"
              placeholder="••••••••"
              value={password}
              onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
              required
              disabled={loading}
            />
            <button
              type="button"
              class="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300 transition-colors"
              onClick={() => setShowPassword(!showPassword)}
              tabIndex={-1}
            >
              {showPassword
                ? (
                  <svg
                    class="w-4 h-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width={2}
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
                    />
                  </svg>
                )
                : (
                  <svg
                    class="w-4 h-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width={2}
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                    />
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.543 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                    />
                  </svg>
                )}
            </button>
          </div>
        </div>

        <button
          type="submit"
          class="btn-primary w-full py-2.5 mt-2 shadow-glow-sm"
          disabled={loading}
        >
          {loading
            ? (
              <span class="flex items-center justify-center gap-2">
                <svg
                  class="animate-spin h-4 w-4 text-white"
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                  >
                  </circle>
                  <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  >
                  </path>
                </svg>
                {type === "signup" ? "Creating account..." : "Signing in..."}
              </span>
            )
            : (
              type === "signup" ? "Create Account" : "Sign In"
            )}
        </button>
      </form>

      <div class="mt-6 text-center text-sm text-zinc-500">
        {type === "signup"
          ? (
            <>
              Already have an account?{" "}
              <a
                href="/login"
                class="text-indigo-400 hover:text-indigo-300 font-medium transition-colors"
              >
                Sign in
              </a>
            </>
          )
          : (
            <>
              Don't have an account?{" "}
              <a
                href="/signup"
                class="text-indigo-400 hover:text-indigo-300 font-medium transition-colors"
              >
                Sign up
              </a>
            </>
          )}
      </div>
    </div>
  );
}
