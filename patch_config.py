import sys

with open("dashboard/islands/ConfigForm.tsx", "r") as f:
    content = f.read()

# 1. Add states
state_insertion = """  const [saveError, setSaveError] = useState<string | null>(null);
  const [testingKeys, setTestingKeys] = useState(false);
  const [testResults, setTestResults] = useState<any>(null);"""
content = content.replace('  const [saveError, setSaveError] = useState<string | null>(null);', state_insertion)

# 2. Add testApiKeys function
func_insertion = """
  const testApiKeys = async () => {
    setTestingKeys(true);
    setTestResults(null);
    try {
      const res = await fetch("/api/test_providers", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ torbox_api_key: torboxKey, real_debrid_api_key: rdKey })
      });
      if (res.ok) {
        setTestResults(await res.json());
      } else {
        setTestResults({ error: "Failed to test keys" });
      }
    } catch (e) {
      setTestResults({ error: "Network error" });
    }
    setTestingKeys(false);
  };

  const savePreferences = async () => {"""
content = content.replace('  const savePreferences = async () => {', func_insertion)

# 3. Add UI elements just before Provider Keys Card closing div
# The closing div of Provider Keys Card is just before: {/* ── Playback Preferences Card ── */}
ui_insertion = """
        {/* Test Results */}
        {testResults && !testResults.error && (
          <div class="mt-6 space-y-3 animate-fade-in">
             {testResults.torbox && (
               <div class="p-4 rounded-xl bg-zinc-800/30 border border-white/5 flex items-center justify-between">
                 <div class="flex items-center gap-3">
                   <div class={`w-2 h-2 rounded-full ${testResults.torbox.valid ? (testResults.torbox.premium ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]' : 'bg-amber-400') : 'bg-red-400'}`} />
                   <div>
                     <p class="text-sm font-semibold text-white">TorBox</p>
                     <p class="text-xs text-zinc-400">
                       {testResults.torbox.valid ? (testResults.torbox.premium ? "Premium Active" : "Free Plan") : "Invalid Key or Expired"}
                     </p>
                   </div>
                 </div>
                 {testResults.torbox.valid && testResults.torbox.expires_at && (
                   <span class="text-xs text-zinc-400 font-medium bg-white/5 px-2.5 py-1 rounded-lg">Valid until: {new Date(testResults.torbox.expires_at).toLocaleDateString()}</span>
                 )}
               </div>
             )}
             {testResults.real_debrid && (
               <div class="p-4 rounded-xl bg-zinc-800/30 border border-white/5 flex items-center justify-between">
                 <div class="flex items-center gap-3">
                   <div class={`w-2 h-2 rounded-full ${testResults.real_debrid.valid ? (testResults.real_debrid.premium ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]' : 'bg-amber-400') : 'bg-red-400'}`} />
                   <div>
                     <p class="text-sm font-semibold text-white">Real-Debrid</p>
                     <p class="text-xs text-zinc-400">
                       {testResults.real_debrid.valid ? (testResults.real_debrid.premium ? "Premium Active" : "Free Plan") : "Invalid Key or Expired"}
                     </p>
                   </div>
                 </div>
                 {testResults.real_debrid.valid && testResults.real_debrid.expires_at && (
                   <span class="text-xs text-zinc-400 font-medium bg-white/5 px-2.5 py-1 rounded-lg">Valid until: {new Date(testResults.real_debrid.expires_at).toLocaleDateString()}</span>
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
            {testingKeys ? (
              <>
                <svg class="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Testing...
              </>
            ) : "Test API Keys"}
          </button>
        </div>
      </div>

      {/* ── Playback Preferences Card ── */}"""

content = content.replace('      </div>\n\n      {/* ── Playback Preferences Card ── */}', ui_insertion)

with open("dashboard/islands/ConfigForm.tsx", "w") as f:
    f.write(content)
print("ConfigForm.tsx successfully updated")
