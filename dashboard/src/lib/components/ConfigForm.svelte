<script lang="ts">
  import { onMount } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { createBrowserClient } from '@supabase/ssr';

  const supabaseUrl = env.PUBLIC_SUPABASE_URL || '';
  const supabaseAnonKey = env.PUBLIC_SUPABASE_ANON_KEY || '';
  const gatewayUrl = env.PUBLIC_GATEWAY_URL || 'http://127.0.0.1:8080';

  const supabase = createBrowserClient(supabaseUrl, supabaseAnonKey);

  let userId: string | null = $state(null);
  let loading = $state(true);
  let saving = $state(false);
  let saveSuccess = $state(false);
  let saveError: string | null = $state(null);

  type Profile = { id: string; user_id: string; profile_name: string; prefs_json: any };
  let profiles = $state<Profile[]>([]);
  let currentProfileId = $state('');
  let newProfileName = $state('');
  let showNewProfileModal = $state(false);

  // Form State
  let torboxKey = $state('');
  let maxResolution = $state('4K');
  let sortPreference = $state('balanced');
  let excludeAv1 = $state(false);
  let streamLimit = $state(5);
  let maxSizeGb = $state(0);
  let isPremium = $state(false);
  let monetizationEnabled = $state(false);
  let showTorboxKey = $state(false);
  let deviceProfile = $state('');
  let copiedLink = $state(false);
  
  let testingKeys = $state(false);
  let testResults = $state<{
    error?: string;
    torbox?: { valid: boolean; premium: boolean; expires_at: string };
  } | null>(null);

  onMount(async () => {
    try {
      const res = await fetch('/api/global-config');
      if (res.ok) {
        const data = await res.json();
        monetizationEnabled = data.monetization_enabled === true;
      }
    } catch (e) {
      console.error('Failed to fetch global config:', e);
    }

    try {
      const { data: { session }, error: sessionError } = await supabase.auth.getSession();
      if (sessionError) console.error('Session error:', sessionError);

      if (session?.user) {
        userId = session.user.id;
        await loadProfiles(session.user.id);
      } else {
        window.location.href = '/login';
        return;
      }
    } catch (err) {
      console.error('Unexpected error during auth initialization:', err);
    } finally {
      loading = false;
    }
  });

  async function loadProfiles(uid: string) {
    try {
      const { data } = await supabase
        .from('preferences')
        .select('*')
        .eq('user_id', uid)
        .order('created_at', { ascending: true });

      if (data && data.length > 0) {
        profiles = data;
        selectProfile(data[0].id);
      } else {
        // Create default profile if none exists
        await createProfile('Default Profile');
      }
    } catch (e) {
      console.log('Error loading preferences', e);
    }
  }

  function selectProfile(id: string) {
    currentProfileId = id;
    const p = profiles.find(x => x.id === id);
    if (p && p.prefs_json) {
      const prefs = p.prefs_json;
      torboxKey = prefs.torbox_api_key || '';
      maxResolution = prefs.max_resolution || '4K';
      sortPreference = prefs.sort_preference || 'balanced';
      excludeAv1 = prefs.exclude_av1 || false;
      streamLimit = prefs.stream_limit !== undefined ? prefs.stream_limit : 5;
      maxSizeGb = prefs.max_size_gb || 0;
      isPremium = prefs.is_premium || false;
      deviceProfile = prefs.device_profile || '';
    }
  }

  async function createProfile(name: string) {
    if (!userId) return;
    try {
      const { data, error } = await supabase
        .from('preferences')
        .insert({ user_id: userId, profile_name: name, prefs_json: {} })
        .select()
        .single();
      
      if (data) {
        profiles = [...profiles, data];
        selectProfile(data.id);
      }
      showNewProfileModal = false;
      newProfileName = '';
    } catch (e) {
      console.error('Failed to create profile', e);
    }
  }

  async function testApiKeys() {
    testingKeys = true;
    testResults = null;
    try {
      const res = await fetch('/api/test_providers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ torbox_api_key: torboxKey }),
      });
      if (res.ok) {
        testResults = await res.json();
      } else {
        testResults = { error: 'Failed to test keys' };
      }
    } catch (_e) {
      testResults = { error: 'Network error' };
    }
    testingKeys = false;
  }

  async function savePreferences() {
    if (!userId) {
      saveError = 'Not authenticated. Ensure Anonymous Sign-In is enabled in Supabase.';
      return;
    }
    saving = true;
    saveError = null;
    saveSuccess = false;

    const prefs_json = {
      torbox_api_key: torboxKey,
      max_resolution: maxResolution,
      sort_preference: sortPreference,
      exclude_av1: excludeAv1,
      stream_limit: streamLimit,
      max_size_gb: maxSizeGb,
      device_profile: deviceProfile,
    };

    try {
      const currentProf = profiles.find(p => p.id === currentProfileId);
      const existingJson = currentProf?.prefs_json || {};
      const newJson = { ...existingJson, ...prefs_json };

      const { error } = await supabase
        .from('preferences')
        .update({ prefs_json: newJson })
        .eq('id', currentProfileId);

      if (error) {
        saveError = 'Failed to save: ' + error.message;
      } else {
        saveSuccess = true;
        setTimeout(() => (saveSuccess = false), 4000);

        fetch('/api/telemetry', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            event_type: 'config_saved',
            event_data: {
              has_torbox_key: !!torboxKey,
              max_resolution: maxResolution,
              sort_preference: sortPreference,
              exclude_av1: excludeAv1,
              stream_limit: streamLimit,
              is_premium: isPremium,
              has_device_profile: !!deviceProfile
            }
          })
        }).catch(() => {});
      }
    } catch (_err) {
      saveError = 'Unexpected error saving preferences. Please try again.';
    } finally {
      saving = false;
    }
  }

  async function handleSignOut() {
    await supabase.auth.signOut();
    window.location.href = '/login';
  }

  let baseDomain = $derived(gatewayUrl.replace('https://', '').replace('http://', ''));
  let installLink = $derived(currentProfileId ? `stremio://${baseDomain}/stremio/${currentProfileId}/manifest.json` : '#');
</script>

{#if showNewProfileModal}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="glass-card-strong p-6 rounded-2xl w-full max-w-sm animate-fade-in-up">
      <h3 class="text-lg font-bold text-zinc-900 dark:text-white mb-4">Create New Profile</h3>
      <input type="text" bind:value={newProfileName} placeholder="e.g. Living Room TV" class="input-field mb-4" />
      <div class="flex gap-3 justify-end">
        <button type="button" onclick={() => showNewProfileModal = false} class="px-4 py-2 text-sm font-medium text-zinc-500 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200">Cancel</button>
        <button type="button" onclick={() => createProfile(newProfileName)} disabled={!newProfileName.trim()} class="btn-primary py-2 text-sm">Create</button>
      </div>
    </div>
  </div>
{/if}

{#if loading}
  <div class="space-y-4">
    {#each [1, 2] as i}
      <div class="glass-card-strong p-8 rounded-2xl">
        <div class="flex items-center gap-4 mb-6">
          <div class="w-11 h-11 rounded-xl shimmer"></div>
          <div class="h-5 w-40 rounded-lg shimmer"></div>
        </div>
        <div class="space-y-4">
          <div class="h-3.5 w-24 rounded shimmer"></div>
          <div class="h-11 rounded-xl shimmer"></div>
          <div class="h-3.5 w-32 rounded shimmer"></div>
          <div class="h-11 rounded-xl shimmer"></div>
        </div>
      </div>
    {/each}
  </div>
{:else}
  <div class="space-y-6">
    <!-- Auth error banner -->
    {#if !userId}
      <div class="alert alert-error">
        <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <div>
          <p class="font-semibold">Authentication Error</p>
          <p class="mt-0.5 opacity-80">
            Could not authenticate. Please ensure Anonymous Sign-In is enabled in your Supabase project settings.
          </p>
        </div>
      </div>
    {/if}

    <!-- Profile Selector Card -->
    <div class="glass-card-strong p-6 sm:p-8 rounded-2xl">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-lg font-semibold text-zinc-900 dark:text-white">Active Profile</h2>
          <p class="text-xs text-zinc-500 dark:text-zinc-400 mt-0.5">Switch profiles to manage different configurations.</p>
        </div>
        <div class="flex items-center gap-3 w-full sm:w-auto">
          <select class="select-field flex-grow sm:flex-grow-0" value={currentProfileId} onchange={(e) => selectProfile(e.currentTarget.value)}>
            {#each profiles as profile}
              <option value={profile.id}>{profile.profile_name}</option>
            {/each}
          </select>
          <button type="button" onclick={() => showNewProfileModal = true} class="btn-primary py-2 px-3 text-sm whitespace-nowrap">
            + New
          </button>
        </div>
      </div>
    </div>

    <!-- Provider API Keys Card -->
    <div class="glass-card-strong p-6 sm:p-8 rounded-2xl">
      <div class="flex items-center gap-3 mb-7">
        <div class="icon-box bg-indigo-500/10">
          <svg class="w-5 h-5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
          </svg>
        </div>
        <div>
          <h2 class="text-lg font-semibold text-zinc-900 dark:text-white">Provider Keys</h2>
          <p class="text-xs text-zinc-500 dark:text-zinc-400 mt-0.5">Stored encrypted. Never exposed to clients.</p>
        </div>
      </div>

      {#if monetizationEnabled && !isPremium}
        <div class="mb-8 p-5 bg-gradient-to-br from-indigo-50 to-purple-50 dark:from-indigo-500/10 dark:to-purple-500/10 border border-indigo-100 dark:border-indigo-500/20 rounded-xl relative overflow-hidden">
          <div class="absolute top-0 right-0 w-32 h-32 bg-indigo-500/10 dark:bg-indigo-500/20 blur-3xl rounded-full -mr-16 -mt-16 pointer-events-none"></div>
          <div class="relative flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between">
            <div>
              <h3 class="text-base font-bold text-indigo-900 dark:text-indigo-100 flex items-center gap-2">
                <svg class="w-5 h-5 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
                </svg>
                Upgrade to Atlas Premium
              </h3>
              <p class="text-sm text-indigo-700/80 dark:text-indigo-200/70 mt-1">Unlock 4K streaming and instant uncached downloads.</p>
            </div>
            <button
              onclick={() => window.location.href = '/api/stripe-checkout'}
              type="button"
              class="whitespace-nowrap px-5 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-semibold rounded-xl shadow-lg shadow-indigo-500/30 transition-all hover:scale-105 active:scale-95"
            >
              Upgrade Now
            </button>
          </div>
        </div>
      {/if}

      <div class="space-y-5">
        <!-- TorBox -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300" for="torbox-key">
              TorBox API Key
            </label>
            <div class="flex items-center gap-2">
              <span class={`w-1.5 h-1.5 rounded-full ${torboxKey ? "bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.8)]" : "bg-zinc-300 dark:bg-zinc-700"}`}></span>
              <span class="text-[11px] text-zinc-500 font-medium">{torboxKey ? "Connected" : "Not set"}</span>
            </div>
          </div>
          <div class="relative">
            <input
              id="torbox-key"
              type={showTorboxKey ? "text" : "password"}
              bind:value={torboxKey}
              class="input-field pr-12"
              placeholder="tb-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
            />
            <button
              type="button"
              onclick={() => showTorboxKey = !showTorboxKey}
              class="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-zinc-600 dark:text-zinc-500 dark:hover:text-zinc-300 transition-colors p-1 rounded-lg"
              aria-label={showTorboxKey ? "Hide key" : "Show key"}
            >
              {#if showTorboxKey}
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                </svg>
              {:else}
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              {/if}
            </button>
          </div>
          <p class="text-xs text-zinc-500 dark:text-zinc-600">
            Get your key at <a href="https://torbox.app/settings" target="_blank" rel="noopener noreferrer" class="text-indigo-600 hover:text-indigo-700 dark:text-indigo-400 dark:hover:text-indigo-300 underline underline-offset-2 transition-colors">torbox.app/settings</a>
          </p>
        </div>
      </div>

      <!-- Test Results -->
      {#if testResults && !testResults.error}
        <div class="mt-6 space-y-3 animate-fade-in">
          {#if testResults.torbox}
            <div class="p-4 rounded-xl bg-zinc-100 dark:bg-zinc-800/30 border border-black/5 dark:border-white/5 flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class={`w-2 h-2 rounded-full ${testResults.torbox.valid ? (testResults.torbox.premium ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]' : 'bg-amber-400') : 'bg-red-400'}`}></div>
                <div>
                  <p class="text-sm font-semibold text-zinc-900 dark:text-white">TorBox</p>
                  <p class="text-xs text-zinc-500 dark:text-zinc-400">
                    {testResults.torbox.valid ? (testResults.torbox.premium ? 'Premium Active' : 'Free Plan') : 'Invalid Key or Expired'}
                  </p>
                </div>
              </div>
              {#if testResults.torbox.valid && testResults.torbox.expires_at}
                <span class="text-xs text-zinc-600 dark:text-zinc-400 font-medium bg-black/5 dark:bg-white/5 px-2.5 py-1 rounded-lg">
                  Valid until: {new Date(testResults.torbox.expires_at).toLocaleDateString()}
                </span>
              {/if}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Test Buttons Row -->
      <div class="mt-6 flex items-center justify-end">
        <button
          type="button"
          onclick={testApiKeys}
          disabled={testingKeys || !torboxKey}
          class="flex items-center gap-2 px-4 py-2 bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-300 text-sm font-medium rounded-xl border border-indigo-500/20 shadow-sm transition-all duration-200 disabled:opacity-50"
        >
          {#if testingKeys}
            <svg class="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Testing...
          {:else}
            Test API Keys
          {/if}
        </button>
      </div>
    </div>

    <!-- Playback Preferences Card -->
    <div class="glass-card-strong p-6 sm:p-8 rounded-2xl">
      <div class="flex items-center gap-3 mb-7">
        <div class="icon-box bg-purple-500/10">
          <svg class="w-5 h-5 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
        </div>
        <div>
          <h2 class="text-lg font-semibold text-zinc-900 dark:text-white">Playback Preferences</h2>
          <p class="text-xs text-zinc-500 dark:text-zinc-400 mt-0.5">Tune quality and compatibility for your device.</p>
        </div>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
        <!-- Max resolution -->
        <div class="space-y-2">
          <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300" for="max-resolution">Maximum Resolution</label>
          <div class="relative">
            <select id="max-resolution" bind:value={maxResolution} class="select-field">
              <option value="4K">4K Ultra HD</option>
              <option value="1080p">1080p Full HD</option>
              <option value="720p">720p HD</option>
            </select>
          </div>
          <p class="text-xs text-zinc-500 dark:text-zinc-600">Atlas won't serve sources above this quality.</p>
        </div>

        <!-- Sort Preference -->
        <div class="space-y-2">
          <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300" for="sort-preference">Stream Sorting</label>
          <div class="relative">
            <select id="sort-preference" bind:value={sortPreference} class="select-field">
              <option value="balanced">Balanced (Recommended)</option>
              <option value="quality">Quality First (Largest Files)</option>
              <option value="speed">Speed First (Smallest Files)</option>
            </select>
          </div>
          <p class="text-xs text-zinc-500 dark:text-zinc-600 mt-1.5">How streams are ordered in Stremio.</p>
        </div>

        <!-- Stream Limit -->
        <div class="space-y-2">
          <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300" for="stream-limit">Results per Title</label>
          <div class="relative">
            <select id="stream-limit" bind:value={streamLimit} class="select-field">
              <option value="5">5 Streams (Fastest)</option>
              <option value="10">10 Streams</option>
              <option value="20">20 Streams</option>
              <option value="50">50 Streams</option>
            </select>
          </div>
          <p class="text-xs text-zinc-500 dark:text-zinc-600 mt-1.5">How many streams to show in the list.</p>
        </div>

        <!-- Maximum File Size -->
        <div class="space-y-2">
          <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300" for="max-size">Maximum Stream Size</label>
          <div class="relative">
            <select id="max-size" bind:value={maxSizeGb} class="select-field">
              <option value={0}>Unlimited</option>
              <option value={5}>5 GB</option>
              <option value={10}>10 GB</option>
              <option value={15}>15 GB</option>
              <option value={20}>20 GB</option>
              <option value={30}>30 GB</option>
            </select>
          </div>
          <p class="text-xs text-zinc-500 dark:text-zinc-600 mt-1.5">Filters out streams larger than this size.</p>
        </div>

        <!-- AV1 toggle -->
        <div class="space-y-2">
          <p class="block text-sm font-medium text-zinc-700 dark:text-zinc-300">Device Compatibility</p>
          <button type="button" role="switch" aria-checked={excludeAv1} onclick={() => excludeAv1 = !excludeAv1} class="toggle-wrapper group">
            <div class={`toggle-track ${excludeAv1 ? 'active' : ''}`}>
              <div class="toggle-thumb"></div>
            </div>
            <div class="text-left">
              <span class="text-sm font-medium text-zinc-700 group-hover:text-zinc-900 dark:text-zinc-300 dark:group-hover:text-white transition-colors block">Exclude AV1 Codec</span>
              <span class="text-xs text-zinc-500 dark:text-zinc-600 block mt-0.5">Recommended for older TVs & Apple devices</span>
            </div>
          </button>
        </div>

        <!-- Device Profile -->
        <div class="space-y-2">
          <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300 flex items-center justify-between" for="device-profile">
            <span>AI Device Profile</span>
            {#if monetizationEnabled && !isPremium}
              <span class="text-xs bg-amber-500/10 text-amber-500 px-2 py-0.5 rounded-full font-medium">Premium</span>
            {/if}
          </label>
          <div class="relative">
            <textarea
              id="device-profile"
              bind:value={deviceProfile}
              placeholder="e.g. LG C2 OLED TV with Sonos Arc Soundbar"
              disabled={monetizationEnabled && !isPremium}
              class={`input-field min-h-[80px] resize-y ${monetizationEnabled && !isPremium ? 'opacity-50 cursor-not-allowed' : ''}`}
            ></textarea>
          </div>
          <p class="text-xs text-zinc-500 dark:text-zinc-600 mt-1.5">Describe your hardware setup. Atlas AI will automatically optimize streams for your specific capabilities.</p>
        </div>
      </div>

      <!-- Save row -->
      <div class="pt-8 mt-2 border-t border-black/5 dark:border-white/[0.06] flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div class="min-h-[1.5rem]">
          {#if saveSuccess}
            <div class="flex items-center gap-2 text-emerald-400 text-sm font-medium animate-fade-in">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Preferences saved successfully!
            </div>
          {/if}
          {#if saveError}
            <div class="flex items-center gap-2 text-red-400 text-sm animate-fade-in">
              <svg class="w-4 h-4 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              {saveError}
            </div>
          {/if}
        </div>

        <button type="button" onclick={savePreferences} disabled={saving || !userId} class="btn-primary w-full sm:w-auto">
          {#if saving}
            <svg class="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Saving...
          {:else}
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
            </svg>
            Save Preferences
          {/if}
        </button>
      </div>
    </div>

    <!-- Install Section -->
    <div class="relative overflow-hidden rounded-2xl">
      <div class="absolute inset-0 bg-gradient-to-br from-indigo-600/15 via-purple-600/10 to-transparent"></div>
      <div class="absolute inset-0 border border-indigo-500/20 rounded-2xl"></div>
      <div class="absolute -top-10 -right-10 w-48 h-48 bg-indigo-500/15 rounded-full blur-3xl pointer-events-none"></div>

      <div class="relative p-6 sm:p-8">
        <div class="flex items-start gap-3 mb-6">
          <div class="icon-box bg-indigo-500/20 flex-shrink-0">
            <svg class="w-5 h-5 text-indigo-300" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
          </div>
          <div>
            <h2 class="text-lg font-semibold text-zinc-900 dark:text-white">Install to Stremio</h2>
            <p class="text-sm text-zinc-500 dark:text-zinc-400 mt-0.5">
              {userId ? "Your unique Atlas endpoint is configured. Click below to add it to Stremio instantly." : "Save your preferences first to generate your personal endpoint."}
            </p>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-7">
          {#each [{ n: "1", text: "Save your API keys above" }, { n: "2", text: "Click Install Addon" }, { n: "3", text: "Stremio opens and adds Atlas" }] as s}
            <div class="flex items-center gap-3">
              <div class="w-7 h-7 rounded-full bg-indigo-500/20 border border-indigo-500/30 flex items-center justify-center text-xs font-bold text-indigo-300 flex-shrink-0">
                {s.n}
              </div>
              <p class="text-sm text-zinc-500 dark:text-zinc-400">{s.text}</p>
            </div>
          {/each}
        </div>

        <div class="flex flex-col sm:flex-row gap-3 items-start sm:items-center">
          <a
            href={installLink}
            onclick={(e) => {
              if (!userId) {
                e.preventDefault();
                saveError = "You must wait for authentication to complete before installing.";
              }
            }}
            class={`inline-flex items-center gap-2.5 px-7 py-3.5 rounded-xl font-semibold text-sm transition-all duration-200 ${userId ? "bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white shadow-glow-md hover:shadow-glow-lg hover:-translate-y-0.5" : "bg-zinc-100 dark:bg-zinc-800 text-zinc-400 dark:text-zinc-500 cursor-not-allowed border-black/5 dark:border-white/5 border"}`}
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Install Addon
          </a>

          <button
            type="button"
            onclick={() => {
              if (userId) {
                // If it's a stremio:// link, we probably want to copy the https:// version
                const httpsLink = installLink.startsWith('stremio://') ? installLink.replace('stremio://', 'https://') : installLink;
                navigator.clipboard.writeText(httpsLink);
                copiedLink = true;
                setTimeout(() => copiedLink = false, 2000);
              }
            }}
            disabled={!userId}
            class={`inline-flex items-center gap-2.5 px-5 py-3.5 rounded-xl font-semibold text-sm transition-all duration-200 ${userId ? "bg-white dark:bg-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-700 text-zinc-900 dark:text-white border-black/10 dark:border-white/10 border" : "bg-zinc-100 dark:bg-zinc-800 text-zinc-400 dark:text-zinc-500 cursor-not-allowed border-black/5 dark:border-white/5 border"}`}
          >
            {#if copiedLink}
              <svg class="w-4 h-4 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
              </svg>
              Copied!
            {:else}
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" />
              </svg>
              Copy Link
            {/if}
          </button>
          
          {#if userId}
            <div class="mt-6 flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4 justify-between">
              <div class="flex items-center gap-2 text-xs text-zinc-500">
                <svg class="w-3.5 h-3.5 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
                <span>Authenticated & ready</span>
              </div>
              <button type="button" onclick={handleSignOut} class="text-xs text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 transition-colors">
                Sign Out
              </button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
