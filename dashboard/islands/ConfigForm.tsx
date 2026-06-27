import { useState, useEffect } from "preact/hooks";
import { getSupabaseClient } from "../utils/supabase.ts";

export default function ConfigForm({ 
  projectId, 
  supabaseUrl, 
  supabaseAnonKey, 
  gatewayUrl 
}: { 
  projectId: string, 
  supabaseUrl: string, 
  supabaseAnonKey: string, 
  gatewayUrl: string 
}) {
  const [torboxKey, setTorboxKey] = useState("");
  const [rdKey, setRdKey] = useState("");
  const [userId, setUserId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [maxResolution, setMaxResolution] = useState("4K");
  const [excludeAv1, setExcludeAv1] = useState(false);

  const supabase = getSupabaseClient(supabaseUrl, supabaseAnonKey);

  useEffect(() => {
    const initializeAuth = async () => {
      try {
        const { data: { session }, error: sessionError } = await supabase.auth.getSession();
        
        if (sessionError) {
          console.error("Session error:", sessionError);
        }

        if (session?.user) {
          setUserId(session.user.id);
          await loadPreferences(session.user.id);
        } else {
          const { data, error } = await supabase.auth.signInAnonymously();
          if (error) {
            console.error("Anonymous sign in failed", error);
            // It's possible anonymous sign in is disabled in the Supabase dashboard
          } else if (data.user) {
            setUserId(data.user.id);
            await loadPreferences(data.user.id);
          }
        }
      } catch (err) {
        console.error("Unexpected error during auth initialization:", err);
      } finally {
        setLoading(false);
      }
    };

    initializeAuth();
  }, []);

  const loadPreferences = async (uid: string) => {
    try {
      const { data, error } = await supabase
        .from('preferences')
        .select('prefs_json')
        .eq('id', uid)
        .single();
        
      if (data && data.prefs_json) {
        const prefs = data.prefs_json;
        setTorboxKey(prefs.torbox_api_key || "");
        setRdKey(prefs.real_debrid_api_key || "");
        if (prefs.max_resolution) setMaxResolution(prefs.max_resolution);
        if (prefs.exclude_av1 !== undefined) setExcludeAv1(prefs.exclude_av1);
      }
    } catch (e) {
      console.log("No existing preferences found or error loading them", e);
    }
  };

  const savePreferences = async () => {
    if (!userId) {
      alert("Error: You are not authenticated. Please ensure Anonymous Sign-In is enabled in Supabase.");
      return;
    }
    setSaving(true);
    
    const prefs_json = {
      torbox_api_key: torboxKey,
      real_debrid_api_key: rdKey,
      max_resolution: maxResolution,
      exclude_av1: excludeAv1,
    };

    try {
      const { error } = await supabase
        .from('preferences')
        .upsert({ id: userId, prefs_json });
        
      if (error) {
        alert("Failed to save preferences: " + error.message);
      } else {
        // Success
      }
    } catch (err) {
      alert("Unexpected error saving preferences.");
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div class="flex justify-center items-center p-12">
        <div class="flex space-x-2 animate-pulse">
          <div class="w-3 h-3 bg-indigo-500 rounded-full"></div>
          <div class="w-3 h-3 bg-purple-500 rounded-full"></div>
          <div class="w-3 h-3 bg-indigo-500 rounded-full"></div>
        </div>
      </div>
    );
  }

  // gatewayUrl example: http://127.0.0.1:8080 or https://cindral-atlas-gateway-dev.fly.dev
  // We need to convert it to stremio:// for the installation
  const baseDomain = gatewayUrl.replace("https://", "").replace("http://", "");
  const installLink = userId 
    ? `stremio://${baseDomain}/stremio/${userId}/manifest.json`
    : "#";

  return (
    <div class="space-y-8 w-full">
      <div class="bg-[#09090b]/50 border border-white/10 p-6 sm:p-8 rounded-2xl shadow-xl backdrop-blur-xl">
        <h2 class="text-xl sm:text-2xl font-semibold mb-6 text-zinc-100 flex items-center gap-3">
          <div class="p-2 bg-indigo-500/20 rounded-lg">
            <svg class="w-5 h-5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </div>
          Provider Integrations
        </h2>
        
        {!userId && (
          <div class="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-200 text-sm flex items-start gap-3">
            <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <div>
              <p class="font-semibold">Authentication Error</p>
              <p class="mt-1 opacity-90">Could not authenticate. Please ensure Anonymous Sign-In is enabled in your Supabase project settings.</p>
            </div>
          </div>
        )}

        <div class="space-y-6">
          <div class="space-y-5">
            <div>
              <label class="block text-sm font-medium text-zinc-300 mb-1.5">TorBox API Key</label>
              <input 
                type="password" 
                value={torboxKey}
                onInput={(e) => setTorboxKey((e.target as HTMLInputElement).value)}
                class="w-full px-4 py-2.5 bg-black/40 border border-white/10 rounded-xl text-white placeholder-zinc-600 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/50 transition-all text-sm" 
                placeholder="Enter your TorBox API key" 
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium text-zinc-300 mb-1.5">RealDebrid API Key</label>
              <input 
                type="password" 
                value={rdKey}
                onInput={(e) => setRdKey((e.target as HTMLInputElement).value)}
                class="w-full px-4 py-2.5 bg-black/40 border border-white/10 rounded-xl text-white placeholder-zinc-600 focus:outline-none focus:border-purple-500/50 focus:ring-1 focus:ring-purple-500/50 transition-all text-sm" 
                placeholder="Enter your RealDebrid API key" 
              />
            </div>
          </div>

          <div class="border-t border-white/10 pt-6 mt-6">
            <h3 class="text-sm font-medium text-zinc-300 mb-4 uppercase tracking-wider">Playback Options</h3>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
              <div>
                <label class="block text-sm font-medium text-zinc-300 mb-1.5">Max Resolution</label>
                <div class="relative">
                  <select 
                    value={maxResolution}
                    onChange={(e) => setMaxResolution((e.target as HTMLSelectElement).value)}
                    class="w-full px-4 py-2.5 bg-black/40 border border-white/10 rounded-xl text-white focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/50 transition-all appearance-none text-sm"
                  >
                    <option value="4K">4K Ultra HD</option>
                    <option value="1080p">1080p Full HD</option>
                    <option value="720p">720p HD</option>
                  </select>
                  <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-3 text-zinc-400">
                    <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                  </div>
                </div>
              </div>
              
              <div class="flex flex-col justify-center pt-1">
                <label class="block text-sm font-medium text-zinc-300 mb-2">Device Compatibility</label>
                <label class="relative inline-flex items-center cursor-pointer group">
                  <input 
                    type="checkbox" 
                    checked={excludeAv1}
                    onChange={(e) => setExcludeAv1((e.target as HTMLInputElement).checked)}
                    class="sr-only peer" 
                  />
                  <div class="w-11 h-6 bg-white/10 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-zinc-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500 group-hover:bg-white/20 transition-colors"></div>
                  <span class="ml-3 text-sm font-medium text-zinc-400 group-hover:text-zinc-300 transition-colors">Exclude AV1 Codec</span>
                </label>
              </div>
            </div>
          </div>

          <div class="pt-4 flex justify-end">
            <button 
              onClick={savePreferences}
              disabled={saving || !userId}
              class="w-full sm:w-auto relative group inline-flex items-center justify-center px-6 py-2.5 text-sm font-medium text-white bg-white/10 hover:bg-white/20 border border-white/10 rounded-xl overflow-hidden transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <span class="relative flex items-center gap-2">
                {saving ? (
                  <>
                    <svg class="animate-spin h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Saving Changes...
                  </>
                ) : (
                  "Save Preferences"
                )}
              </span>
            </button>
          </div>
        </div>
      </div>

      <div class="bg-gradient-to-br from-indigo-500/10 to-purple-500/10 border border-indigo-500/20 p-6 sm:p-8 rounded-2xl shadow-lg backdrop-blur-md relative overflow-hidden flex flex-col md:flex-row items-center justify-between gap-6">
        <div class="absolute top-0 right-0 -mt-4 -mr-4 w-32 h-32 bg-indigo-500/20 rounded-full blur-2xl"></div>
        
        <div class="relative z-10 flex-1 text-center md:text-left">
          <h2 class="text-xl font-semibold mb-2 text-white flex items-center justify-center md:justify-start gap-2">
            <svg class="w-5 h-5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Ready to Install
          </h2>
          <p class="text-zinc-400 text-sm md:text-base">Your unique Atlas endpoint is configured. Click the button to add it to Stremio instantly.</p>
        </div>
        
        <a 
          href={installLink}
          onClick={(e) => {
            if (!userId) {
              e.preventDefault();
              alert("You must wait for authentication to complete before installing.");
            }
          }}
          class={`relative z-10 inline-flex items-center justify-center px-6 py-3 font-semibold text-white rounded-xl transition-all duration-300 shadow-lg whitespace-nowrap ${
            userId 
              ? "bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 hover:scale-105 hover:shadow-[0_0_20px_rgba(79,70,229,0.4)]" 
              : "bg-zinc-800 cursor-not-allowed opacity-50"
          }`}
        >
          <span class="flex items-center gap-2">
            Install Addon
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
            </svg>
          </span>
        </a>
      </div>
    </div>
  );
}
