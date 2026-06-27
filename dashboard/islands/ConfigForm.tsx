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
    <div class="space-y-8">
      <div class="bg-white/5 border border-white/10 p-8 rounded-2xl shadow-lg backdrop-blur-md">
        <h2 class="text-2xl font-semibold mb-6 text-white flex items-center gap-2">
          <svg class="w-6 h-6 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          Provider Integrations
        </h2>
        
        {!userId && (
          <div class="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-200 text-sm">
            Warning: Could not authenticate. Please ensure Anonymous Sign-In is enabled in your Supabase project settings.
          </div>
        )}

        <div class="space-y-6">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">TorBox API Key</label>
            <input 
              type="password" 
              value={torboxKey}
              onInput={(e) => setTorboxKey((e.target as HTMLInputElement).value)}
              class="w-full px-4 py-3 bg-black/20 border border-white/10 rounded-xl text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 transition-all" 
              placeholder="Enter your TorBox API key" 
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">RealDebrid API Key</label>
            <input 
              type="password" 
              value={rdKey}
              onInput={(e) => setRdKey((e.target as HTMLInputElement).value)}
              class="w-full px-4 py-3 bg-black/20 border border-white/10 rounded-xl text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500/50 transition-all" 
              placeholder="Enter your RealDebrid API key" 
            />
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-2">Max Resolution</label>
              <select 
                value={maxResolution}
                onChange={(e) => setMaxResolution((e.target as HTMLSelectElement).value)}
                class="w-full px-4 py-3 bg-black/20 border border-white/10 rounded-xl text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50 transition-all appearance-none"
              >
                <option value="4K">4K</option>
                <option value="1080p">1080p</option>
                <option value="720p">720p</option>
              </select>
            </div>
            
            <div class="flex flex-col justify-center">
              <label class="block text-sm font-medium text-gray-300 mb-3">Device Compatibility</label>
              <label class="relative inline-flex items-center cursor-pointer">
                <input 
                  type="checkbox" 
                  checked={excludeAv1}
                  onChange={(e) => setExcludeAv1((e.target as HTMLInputElement).checked)}
                  class="sr-only peer" 
                />
                <div class="w-11 h-6 bg-white/10 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
                <span class="ml-3 text-sm font-medium text-gray-400">Exclude AV1 Codec</span>
              </label>
            </div>
          </div>

          <button 
            onClick={savePreferences}
            disabled={saving || !userId}
            class="w-full sm:w-auto relative group inline-flex items-center justify-center px-8 py-3 font-medium text-white bg-indigo-600 rounded-xl overflow-hidden transition-all duration-300 hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <span class="relative flex items-center gap-2">
              {saving ? (
                <>
                  <svg class="animate-spin h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Saving...
                </>
              ) : (
                "Save Configurations"
              )}
            </span>
          </button>
        </div>
      </div>

      <div class="bg-gradient-to-br from-indigo-500/10 to-purple-500/10 border border-indigo-500/20 p-8 rounded-2xl shadow-lg backdrop-blur-md relative overflow-hidden">
        <div class="absolute top-0 right-0 -mt-4 -mr-4 w-32 h-32 bg-indigo-500/20 rounded-full blur-2xl"></div>
        
        <h2 class="text-2xl font-semibold mb-2 text-white flex items-center gap-2">
          <svg class="w-6 h-6 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          Stremio Installation
        </h2>
        <p class="text-gray-300 mb-6 text-lg">Your unique Atlas endpoint is ready. Click below to install it directly into Stremio.</p>
        
        <a 
          href={installLink}
          onClick={(e) => {
            if (!userId) {
              e.preventDefault();
              alert("You must wait for authentication to complete before installing.");
            }
          }}
          class={`inline-flex items-center justify-center px-8 py-4 font-bold text-white rounded-xl transition-all duration-300 shadow-lg ${
            userId 
              ? "bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 hover:scale-105 hover:shadow-indigo-500/25" 
              : "bg-gray-700 cursor-not-allowed opacity-50"
          }`}
        >
          <span class="flex items-center gap-2">
            Install on Stremio
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
            </svg>
          </span>
        </a>
      </div>
    </div>
  );
}
