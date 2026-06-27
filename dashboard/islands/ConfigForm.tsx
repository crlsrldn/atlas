import { useState, useEffect } from "preact/hooks";
import { supabase } from "../utils/supabase.ts";

export default function ConfigForm({ projectId }: { projectId: string }) {
  const [torboxKey, setTorboxKey] = useState("");
  const [rdKey, setRdKey] = useState("");
  const [userId, setUserId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    supabase.auth.getSession().then(({ data: { session } }) => {
      if (session?.user) {
        setUserId(session.user.id);
        loadPreferences(session.user.id);
      } else {
        supabase.auth.signInAnonymously().then(({ data, error }) => {
          if (data.user) {
            setUserId(data.user.id);
            loadPreferences(data.user.id);
          } else {
            console.error("Anonymous sign in failed", error);
            setLoading(false);
          }
        });
      }
    });
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
      }
    } catch (e) {
      console.log("No existing preferences found", e);
    } finally {
      setLoading(false);
    }
  };

  const savePreferences = async () => {
    if (!userId) return;
    setSaving(true);
    
    const prefs_json = {
      torbox_api_key: torboxKey,
      real_debrid_api_key: rdKey,
      max_resolution: "4K",
      exclude_av1: false,
    };

    const { error } = await supabase
      .from('preferences')
      .upsert({ id: userId, prefs_json });
      
    if (error) {
      alert("Failed to save preferences: " + error.message);
    } else {
      alert("Configurations saved!");
    }
    setSaving(false);
  };

  if (loading) {
    return <div class="p-6">Loading configurations...</div>;
  }

  const installLink = userId 
    ? `stremio://cindral-atlas-gateway-dev.fly.dev/stremio/${userId}/manifest.json`
    : "#";

  return (
    <div>
      <div class="bg-gray-100 p-6 rounded-lg mb-8">
        <h2 class="text-2xl font-semibold mb-4">Integrations</h2>
        
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">TorBox API Key</label>
          <input 
            type="password" 
            value={torboxKey}
            onInput={(e) => setTorboxKey((e.target as HTMLInputElement).value)}
            class="w-full px-4 py-2 border rounded-md" 
            placeholder="Enter your TorBox API key" 
          />
        </div>
        
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">RealDebrid API Key</label>
          <input 
            type="password" 
            value={rdKey}
            onInput={(e) => setRdKey((e.target as HTMLInputElement).value)}
            class="w-full px-4 py-2 border rounded-md" 
            placeholder="Enter your RealDebrid API key" 
          />
        </div>

        <button 
          onClick={savePreferences}
          disabled={saving}
          class="bg-black text-white px-4 py-2 rounded-md hover:bg-gray-800 transition disabled:opacity-50"
        >
          {saving ? "Saving..." : "Save Configurations"}
        </button>
      </div>

      <div class="bg-blue-50 border border-blue-200 p-6 rounded-lg">
        <h2 class="text-2xl font-semibold mb-2">Stremio Installation</h2>
        <p class="text-gray-700 mb-4">Click the button below to install Project Atlas to your Stremio.</p>
        <a 
          href={installLink} 
          class="inline-block bg-blue-600 text-white px-6 py-3 rounded-md font-semibold hover:bg-blue-700 transition"
        >
          Install on Stremio
        </a>
      </div>
    </div>
  );
}
