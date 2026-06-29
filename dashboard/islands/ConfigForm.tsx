import { useEffect, useState } from "preact/hooks";
import { getSupabaseClient } from "../utils/supabase.ts";

interface ConfigFormProps {
  projectId: string;
  supabaseUrl: string;
  supabaseAnonKey: string;
  gatewayUrl: string;
}

interface Preferences {
  torbox_api_key?: string;
  real_debrid_api_key?: string;
  max_resolution?: string;
  exclude_av1?: boolean;
}

export default function ConfigForm(
  { supabaseUrl, supabaseAnonKey, gatewayUrl }: Omit<
    ConfigFormProps,
    "projectId"
  >,
) {
  const [torboxKey, setTorboxKey] = useState("");
  const [rdKey, setRdKey] = useState("");
  const [userId, setUserId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [maxResolution, setMaxResolution] = useState("4K");
  const [excludeAv1, setExcludeAv1] = useState(false);
  const [showTorboxKey, setShowTorboxKey] = useState(false);
  const [showRdKey, setShowRdKey] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [testingKeys, setTestingKeys] = useState(false);
  const [testResults, setTestResults] = useState<
    Record<string, unknown> | null
  >(null);

  const supabase = getSupabaseClient(supabaseUrl, supabaseAnonKey);

  useEffect(() => {
    const initializeAuth = async () => {
      try {
        const { data: { session }, error: sessionError } = await supabase.auth
          .getSession();

        if (sessionError) {
          console.error("Session error:", sessionError);
        }

        if (session?.user) {
          setUserId(session.user.id);
          await loadPreferences(session.user.id);
        } else {
          // Redirect to login if not authenticated
          globalThis.location.href = "/login";
          return;
        }
      } catch (err) {
        console.error("Unexpected error during auth initialization:", err);
      } finally {
        setLoading(false);
      }
    };

    initializeAuth();
  }, [supabase]);

  const handleSignOut = async () => {
    await supabase.auth.signOut();
    globalThis.location.href = "/login";
  };

  const loadPreferences = async (uid: string) => {
    try {
      const { data } = await supabase
        .from("preferences")
        .select("prefs_json")
        .eq("id", uid)
        .single();

      if (data?.prefs_json) {
        const prefs: Preferences = data.prefs_json;
        setTorboxKey(prefs.torbox_api_key || "");
        setRdKey(prefs.real_debrid_api_key || "");
        if (prefs.max_resolution) setMaxResolution(prefs.max_resolution);
        if (prefs.exclude_av1 !== undefined) setExcludeAv1(prefs.exclude_av1);
      }
    } catch (_e) {
      console.log("No existing preferences found or error loading them", e);
    }
  };

  const testApiKeys = async () => {
    setTestingKeys(true);
    setTestResults(null);
    try {
      const res = await fetch("/api/test_providers", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          torbox_api_key: torboxKey,
          real_debrid_api_key: rdKey,
        }),
      });
      if (res.ok) {
        setTestResults(await res.json());
      } else {
        setTestResults({ error: "Failed to test keys" });
      }
    } catch (_e) {
      setTestResults({ error: "Network error" });
    }
    setTestingKeys(false);
  };

  const savePreferences = async () => {
    if (!userId) {
      setSaveError(
        "Not authenticated. Ensure Anonymous Sign-In is enabled in Supabase.",
      );
      return;
    }
    setSaving(true);
    setSaveError(null);
    setSaveSuccess(false);

    const prefs_json: Preferences = {
      torbox_api_key: torboxKey,
      real_debrid_api_key: rdKey,
      max_resolution: maxResolution,
      exclude_av1: excludeAv1,
    };

    try {
      const { error } = await supabase
        .from("preferences")
        .upsert({ id: userId, prefs_json });

      if (error) {
        setSaveError("Failed to save: " + error.message);
      } else {
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 4000);
      }
    } catch (_err) {
      setSaveError("Unexpected error saving preferences. Please try again.");
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div class="space-y-4">
        {/* Skeleton loaders */}
        {[1, 2].map((i) => (
          <div
            key={i}
            class="glass-card-strong p-8 rounded-2xl"
          >
            <div class="flex items-center gap-4 mb-6">
              <div class="w-11 h-11 rounded-xl shimmer" />
              <div class="h-5 w-40 rounded-lg shimmer" />
            </div>
            <div class="space-y-4">
              <div class="h-3.5 w-24 rounded shimmer" />
              <div class="h-11 rounded-xl shimmer" />
              <div class="h-3.5 w-32 rounded shimmer" />
              <div class="h-11 rounded-xl shimmer" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  const baseDomain = gatewayUrl.replace("https://", "").replace(
    "http://",
    "",
  );
  const installLink = userId
    ? `stremio://${baseDomain}/stremio/${userId}/manifest.json`
    : "#";

  return (
    <div class="space-y-6">
      {/* ── Auth error banner ── */}
      {!userId && (
        <div class="alert alert-error">
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
            <p class="font-semibold">Authentication Error</p>
            <p class="mt-0.5 opacity-80">
              Could not authenticate. Please ensure Anonymous Sign-In is enabled
              in your Supabase project settings.
            </p>
          </div>
        </div>
      )}

      {/* ── Provider API Keys Card ── */}
      <div class="glass-card-strong p-6 sm:p-8 rounded-2xl">
        {/* Card header */}
        <div class="flex items-center gap-3 mb-7">
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
                d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"
              />
            </svg>
          </div>
          <div>
            <h2 class="text-lg font-semibold text-white">Provider Keys</h2>
            <p class="text-xs text-zinc-500 mt-0.5">
              Stored encrypted. Never exposed to clients.
            </p>
          </div>
        </div>

        <div class="space-y-5">
          {/* TorBox */}
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <label
                class="block text-sm font-medium text-zinc-300"
                for="torbox-key"
              >
                TorBox API Key
              </label>
              <div class="flex items-center gap-2">
                <span
                  class={`w-1.5 h-1.5 rounded-full ${
                    torboxKey
                      ? "bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.8)]"
                      : "bg-zinc-700"
                  }`}
                />
                <span class="text-[11px] text-zinc-500 font-medium">
                  {torboxKey ? "Connected" : "Not set"}
                </span>
              </div>
            </div>
            <div class="relative">
              <input
                id="torbox-key"
                type={showTorboxKey ? "text" : "password"}
                value={torboxKey}
                onInput={(e) =>
                  setTorboxKey((e.target as HTMLInputElement).value)}
                class="input-field pr-12"
                placeholder="tb-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
              />
              <button
                type="button"
                onClick={() => setShowTorboxKey(!showTorboxKey)}
                class="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300 transition-colors p-1 rounded-lg"
                aria-label={showTorboxKey ? "Hide key" : "Show key"}
              >
                {showTorboxKey
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
                        d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                      />
                    </svg>
                  )}
              </button>
            </div>
            <p class="text-xs text-zinc-600">
              Get your key at{" "}
              <a
                href="https://torbox.app/settings"
                target="_blank"
                rel="noopener noreferrer"
                class="text-indigo-400 hover:text-indigo-300 underline underline-offset-2 transition-colors"
              >
                torbox.app/settings
              </a>
            </p>
          </div>

          {/* Separator */}
          <div class="divider" />

          {/* Real-Debrid */}
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <label
                class="block text-sm font-medium text-zinc-300"
                for="rd-key"
              >
                Real-Debrid API Key
              </label>
              <div class="flex items-center gap-2">
                <span
                  class={`w-1.5 h-1.5 rounded-full ${
                    rdKey
                      ? "bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.8)]"
                      : "bg-zinc-700"
                  }`}
                />
                <span class="text-[11px] text-zinc-500 font-medium">
                  {rdKey ? "Connected" : "Not set"}
                </span>
              </div>
            </div>
            <div class="relative">
              <input
                id="rd-key"
                type={showRdKey ? "text" : "password"}
                value={rdKey}
                onInput={(e) => setRdKey((e.target as HTMLInputElement).value)}
                class="input-field pr-12"
                placeholder="XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
              />
              <button
                type="button"
                onClick={() => setShowRdKey(!showRdKey)}
                class="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300 transition-colors p-1 rounded-lg"
                aria-label={showRdKey ? "Hide key" : "Show key"}
              >
                {showRdKey
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
                        d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                      />
                    </svg>
                  )}
              </button>
            </div>
            <p class="text-xs text-zinc-600">
              Get your key at{" "}
              <a
                href="https://real-debrid.com/apitoken"
                target="_blank"
                rel="noopener noreferrer"
                class="text-indigo-400 hover:text-indigo-300 underline underline-offset-2 transition-colors"
              >
                real-debrid.com/apitoken
              </a>
            </p>
          </div>
        </div>

        {/* Test Results */}
        {testResults && !testResults.error && (
          <div class="mt-6 space-y-3 animate-fade-in">
            {testResults.torbox && (
              <div class="p-4 rounded-xl bg-zinc-800/30 border border-white/5 flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div
                    class={`w-2 h-2 rounded-full ${
                      testResults.torbox.valid
                        ? (testResults.torbox.premium
                          ? "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]"
                          : "bg-amber-400")
                        : "bg-red-400"
                    }`}
                  />
                  <div>
                    <p class="text-sm font-semibold text-white">TorBox</p>
                    <p class="text-xs text-zinc-400">
                      {testResults.torbox.valid
                        ? (testResults.torbox.premium
                          ? "Premium Active"
                          : "Free Plan")
                        : "Invalid Key or Expired"}
                    </p>
                  </div>
                </div>
                {testResults.torbox.valid && testResults.torbox.expires_at && (
                  <span class="text-xs text-zinc-400 font-medium bg-white/5 px-2.5 py-1 rounded-lg">
                    Valid until: {new Date(testResults.torbox.expires_at)
                      .toLocaleDateString()}
                  </span>
                )}
              </div>
            )}
            {testResults.real_debrid && (
              <div class="p-4 rounded-xl bg-zinc-800/30 border border-white/5 flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div
                    class={`w-2 h-2 rounded-full ${
                      testResults.real_debrid.valid
                        ? (testResults.real_debrid.premium
                          ? "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]"
                          : "bg-amber-400")
                        : "bg-red-400"
                    }`}
                  />
                  <div>
                    <p class="text-sm font-semibold text-white">Real-Debrid</p>
                    <p class="text-xs text-zinc-400">
                      {testResults.real_debrid.valid
                        ? (testResults.real_debrid.premium
                          ? "Premium Active"
                          : "Free Plan")
                        : "Invalid Key or Expired"}
                    </p>
                  </div>
                </div>
                {testResults.real_debrid.valid &&
                  testResults.real_debrid.expires_at && (
                  <span class="text-xs text-zinc-400 font-medium bg-white/5 px-2.5 py-1 rounded-lg">
                    Valid until: {new Date(testResults.real_debrid.expires_at)
                      .toLocaleDateString()}
                  </span>
                )}
              </div>
            )}
          </div>
        )}

        {/* Test Buttons Row */}
        <div class="mt-6 flex items-center justify-end">
          <button
            type="button"
            onClick={testApiKeys}
            disabled={testingKeys || (!torboxKey && !rdKey)}
            class="flex items-center gap-2 px-4 py-2 bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-300 text-sm font-medium rounded-xl border border-indigo-500/20 shadow-sm transition-all duration-200 disabled:opacity-50"
          >
            {testingKeys
              ? (
                <>
                  <svg
                    class="animate-spin h-4 w-4"
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
                    />
                    <path
                      class="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                  </svg>
                  Testing...
                </>
              )
              : "Test API Keys"}
          </button>
        </div>
      </div>

      {/* ── Playback Preferences Card ── */}
      <div class="glass-card-strong p-6 sm:p-8 rounded-2xl">
        <div class="flex items-center gap-3 mb-7">
          <div class="icon-box bg-purple-500/10">
            <svg
              class="w-5 h-5 text-purple-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width={1.75}
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
              />
            </svg>
          </div>
          <div>
            <h2 class="text-lg font-semibold text-white">
              Playback Preferences
            </h2>
            <p class="text-xs text-zinc-500 mt-0.5">
              Tune quality and compatibility for your device.
            </p>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
          {/* Max resolution */}
          <div class="space-y-2">
            <label
              class="block text-sm font-medium text-zinc-300"
              for="max-resolution"
            >
              Maximum Resolution
            </label>
            <div class="relative">
              <select
                id="max-resolution"
                value={maxResolution}
                onChange={(e) =>
                  setMaxResolution((e.target as HTMLSelectElement).value)}
                class="select-field"
              >
                <option value="4K">4K Ultra HD</option>
                <option value="1080p">1080p Full HD</option>
                <option value="720p">720p HD</option>
              </select>
            </div>
            <p class="text-xs text-zinc-600">
              Atlas won't serve sources above this quality.
            </p>
          </div>

          {/* AV1 toggle */}
          <div class="space-y-2">
            <p class="block text-sm font-medium text-zinc-300">
              Device Compatibility
            </p>
            <button
              type="button"
              role="switch"
              aria-checked={excludeAv1}
              onClick={() => setExcludeAv1(!excludeAv1)}
              class="toggle-wrapper group"
            >
              <div class={`toggle-track ${excludeAv1 ? "active" : ""}`}>
                <div class="toggle-thumb" />
              </div>
              <div class="text-left">
                <span class="text-sm font-medium text-zinc-300 group-hover:text-white transition-colors block">
                  Exclude AV1 Codec
                </span>
                <span class="text-xs text-zinc-600 block mt-0.5">
                  Recommended for older TVs & Apple devices
                </span>
              </div>
            </button>
          </div>
        </div>

        {/* Save row */}
        <div class="pt-8 mt-2 border-t border-white/[0.06] flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          {/* Success / error feedback */}
          <div class="min-h-[1.5rem]">
            {saveSuccess && (
              <div class="flex items-center gap-2 text-emerald-400 text-sm font-medium animate-fade-in">
                <svg
                  class="w-4 h-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  stroke-width={2.5}
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
                Preferences saved successfully!
              </div>
            )}
            {saveError && (
              <div class="flex items-center gap-2 text-red-400 text-sm animate-fade-in">
                <svg
                  class="w-4 h-4 flex-shrink-0"
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
                {saveError}
              </div>
            )}
          </div>

          <button
            type="button"
            onClick={savePreferences}
            disabled={saving || !userId}
            class="btn-primary w-full sm:w-auto"
          >
            {saving
              ? (
                <>
                  <svg
                    class="animate-spin h-4 w-4"
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
                    />
                    <path
                      class="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                  </svg>
                  Saving...
                </>
              )
              : (
                <>
                  <svg
                    class="w-4 h-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width={2.5}
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                  Save Preferences
                </>
              )}
          </button>
        </div>
      </div>

      {/* ── Install Section ── */}
      <div class="relative overflow-hidden rounded-2xl">
        <div class="absolute inset-0 bg-gradient-to-br from-indigo-600/15 via-purple-600/10 to-transparent" />
        <div class="absolute inset-0 border border-indigo-500/20 rounded-2xl" />
        <div class="absolute -top-10 -right-10 w-48 h-48 bg-indigo-500/15 rounded-full blur-3xl pointer-events-none" />

        <div class="relative p-6 sm:p-8">
          {/* Header */}
          <div class="flex items-start gap-3 mb-6">
            <div class="icon-box bg-indigo-500/20 flex-shrink-0">
              <svg
                class="w-5 h-5 text-indigo-300"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width={1.75}
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                />
              </svg>
            </div>
            <div>
              <h2 class="text-lg font-semibold text-white">
                Install to Stremio
              </h2>
              <p class="text-sm text-zinc-400 mt-0.5">
                {userId
                  ? "Your unique Atlas endpoint is configured. Click below to add it to Stremio instantly."
                  : "Save your preferences first to generate your personal endpoint."}
              </p>
            </div>
          </div>

          {/* Steps */}
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-7">
            {[
              { n: "1", text: "Save your API keys above" },
              { n: "2", text: "Click Install Addon" },
              { n: "3", text: "Stremio opens and adds Atlas" },
            ].map((s) => (
              <div key={s.n} class="flex items-center gap-3">
                <div class="w-7 h-7 rounded-full bg-indigo-500/20 border border-indigo-500/30 flex items-center justify-center text-xs font-bold text-indigo-300 flex-shrink-0">
                  {s.n}
                </div>
                <p class="text-sm text-zinc-400">{s.text}</p>
              </div>
            ))}
          </div>

          {/* Action */}
          <div class="flex flex-col sm:flex-row gap-3 items-start sm:items-center">
            <a
              href={installLink}
              onClick={(e) => {
                if (!userId) {
                  e.preventDefault();
                  setSaveError(
                    "You must wait for authentication to complete before installing.",
                  );
                }
              }}
              class={`inline-flex items-center gap-2.5 px-7 py-3.5 rounded-xl font-semibold text-sm transition-all duration-200 ${
                userId
                  ? "bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white shadow-glow-md hover:shadow-glow-lg hover:-translate-y-0.5"
                  : "bg-zinc-800 text-zinc-500 cursor-not-allowed border border-white/5"
              }`}
            >
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
                  d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                />
              </svg>
              Install Addon
            </a>

            {userId && (
              <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4 w-full justify-between">
                <div class="flex items-center gap-2 text-xs text-zinc-500">
                  <svg
                    class="w-3.5 h-3.5 text-emerald-500"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width={2.5}
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                    />
                  </svg>
                  <span>Authenticated & ready</span>
                </div>
                <button
                  type="button"
                  onClick={handleSignOut}
                  class="text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
                >
                  Sign Out
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
