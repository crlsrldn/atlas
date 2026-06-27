<script lang="ts">
  import { onMount } from 'svelte';
  import { BackendUnavailableError, BACKEND_BASE_URL, backendFetch, checkBackendHealth } from '$lib/backend';

  type Account = {
    user_id: string;
    profile_id: string;
    install_token: string;
    plan: 'free' | 'pro';
    subscription_status: string;
    monthly_resolve_quota: number;
    monthly_resolve_count: number;
    stremio_manifest_path: string;
    preferences: Preferences;
  };

  type Preferences = {
    torbox_api_key: string;
    real_debrid_api_key: string;
    gemini_api_key: string;
    has_torbox_api_key: boolean;
    has_real_debrid_api_key: boolean;
    has_gemini_api_key: boolean;
    max_resolution: string;
    prefer_hdr: boolean;
    exclude_av1: boolean;
    profile: string;
    mobile_data_saver: boolean;
    home_theater_mode: boolean;
    family_mode: boolean;
    preferred_language: string;
    subtitle_mode: string;
  };

  let backendOnline = false;
  let isSaving = false;
  let isTestingProviders = false;
  let isOpeningBilling = false;
  let loadMessage = 'Checking Atlas Cloud...';
  let saveMessage = '';
  let account: Account | null = null;
  let providerStatuses: Array<{
    provider: string;
    configured: boolean;
    status: string;
    latency_ms?: number;
    message: string;
  }> = [];

  let torboxApiKey = '';
  let realDebridApiKey = '';
  let geminiApiKey = '';
  let maxResolution = '4K';
  let preferHdr = true;
  let excludeAv1 = false;
  let profile = 'home_theater';
  let mobileDataSaver = false;
  let homeTheaterMode = true;
  let familyMode = false;
  let preferredLanguage = 'English';
  let subtitleMode = 'auto';

  $: manifestUrl = account ? `${BACKEND_BASE_URL}${account.stremio_manifest_path}` : '';
  $: quotaPercent = account
    ? Math.min(100, Math.round((account.monthly_resolve_count / account.monthly_resolve_quota) * 100))
    : 0;
  $: validationErrors = [
    torboxApiKey.trim() && torboxApiKey.trim().length < 12 ? 'TorBox API key looks too short.' : '',
    realDebridApiKey.trim() && realDebridApiKey.trim().length < 12 ? 'Real Debrid API key looks too short.' : '',
    geminiApiKey.trim() && geminiApiKey.trim().length < 12 ? 'Gemini API key looks too short.' : '',
    familyMode && subtitleMode === 'off' ? 'Family profiles should keep subtitles on or automatic.' : ''
  ].filter(Boolean);

  onMount(loadAccount);

  async function loadAccount() {
    try {
      backendOnline = await checkBackendHealth();
      const session = await backendFetch('/auth/session', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({})
      });
      if (!session.ok) {
        loadMessage = 'Atlas Cloud session could not be created.';
        return;
      }

      const accountRes = await backendFetch('/v1/account');
      if (!accountRes.ok) {
        loadMessage = 'Account could not be loaded.';
        return;
      }

      const loadedAccount: Account = await accountRes.json();
      account = loadedAccount;
      applyPreferences(loadedAccount.preferences);
      loadMessage = '';
    } catch (err) {
      console.error(err);
      backendOnline = false;
      loadMessage = err instanceof BackendUnavailableError ? 'Atlas backend is offline.' : 'Account could not be loaded.';
    }
  }

  function applyPreferences(preferences: Preferences) {
    maxResolution = preferences.max_resolution || '4K';
    preferHdr = preferences.prefer_hdr ?? true;
    excludeAv1 = preferences.exclude_av1 ?? false;
    profile = preferences.profile || 'home_theater';
    mobileDataSaver = preferences.mobile_data_saver ?? false;
    homeTheaterMode = preferences.home_theater_mode ?? true;
    familyMode = preferences.family_mode ?? false;
    preferredLanguage = preferences.preferred_language || 'English';
    subtitleMode = preferences.subtitle_mode || 'auto';
  }

  async function saveSettings() {
    if (validationErrors.length > 0) {
      saveMessage = validationErrors[0];
      return;
    }

    isSaving = true;
    saveMessage = '';

    try {
      const res = await backendFetch('/v1/preferences', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          torbox_api_key: torboxApiKey,
          real_debrid_api_key: realDebridApiKey,
          gemini_api_key: geminiApiKey,
          max_resolution: maxResolution,
          prefer_hdr: preferHdr,
          exclude_av1: excludeAv1,
          profile,
          mobile_data_saver: mobileDataSaver,
          home_theater_mode: homeTheaterMode,
          family_mode: familyMode,
          preferred_language: preferredLanguage,
          subtitle_mode: subtitleMode
        })
      });

      if (!res.ok) {
        saveMessage = 'Settings could not be saved.';
        return;
      }

      const preferences = await res.json();
      torboxApiKey = '';
      realDebridApiKey = '';
      geminiApiKey = '';
      if (account) {
        account.preferences = preferences;
      }
      saveMessage = 'Settings saved to Atlas Cloud.';
    } catch (err) {
      console.error(err);
      saveMessage = err instanceof BackendUnavailableError ? 'Atlas backend is offline.' : 'Settings could not be saved.';
    } finally {
      isSaving = false;
    }
  }

  async function testProviders() {
    isTestingProviders = true;
    saveMessage = '';
    try {
      const res = await backendFetch('/v1/providers/status');
      providerStatuses = res.ok ? await res.json() : [];
      if (!res.ok) {
        saveMessage = 'Provider checks failed.';
      }
    } catch (err) {
      console.error(err);
      saveMessage = err instanceof BackendUnavailableError ? 'Atlas backend is offline.' : 'Provider checks failed.';
    } finally {
      isTestingProviders = false;
    }
  }

  async function openBilling() {
    isOpeningBilling = true;
    try {
      const res = await backendFetch('/v1/billing/checkout', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          success_url: window.location.href,
          cancel_url: window.location.href
        })
      });
      if (res.ok) {
        const data = await res.json();
        window.open(data.checkout_url, '_blank', 'noopener,noreferrer');
      }
    } finally {
      isOpeningBilling = false;
    }
  }

  function providerStateLabel(status: string, configured: boolean) {
    if (!configured) return 'Needs key';
    if (status === 'ok') return 'Healthy';
    if (status === 'error') return 'Attention';
    return 'Unknown';
  }
</script>

<section class="shell">
  <div class="header">
    <div>
      <h2>Atlas Cloud</h2>
      <p>Manage hosted Smart Play, provider vault keys, and your Stremio install URL.</p>
    </div>
    <span class:online={backendOnline} class="status">{backendOnline ? 'Online' : 'Offline'}</span>
  </div>

  {#if loadMessage}
    <div class="notice">{loadMessage}</div>
  {/if}

  {#if account}
    <div class="account-grid">
      <div class="panel">
        <span class="eyebrow">Plan</span>
        <h3>{account.plan.toUpperCase()}</h3>
        <p>{account.subscription_status.replaceAll('_', ' ')}</p>
        <button on:click={openBilling} disabled={isOpeningBilling}>
          {isOpeningBilling ? 'Opening billing' : 'Manage billing'}
        </button>
      </div>

      <div class="panel">
        <span class="eyebrow">Monthly resolves</span>
        <h3>{account.monthly_resolve_count} / {account.monthly_resolve_quota}</h3>
        <div class="meter"><span style={`width: ${quotaPercent}%`}></span></div>
        <p>Free tier is capped so the hosted resolver stays cheap to run.</p>
      </div>

      <div class="panel install">
        <span class="eyebrow">Stremio add-on</span>
        <input readonly value={manifestUrl} aria-label="Hosted manifest URL" />
        <p>Install this tenant-scoped URL in Stremio. Atlas resolves and redirects; it never proxies video bytes.</p>
      </div>
    </div>
  {/if}

  <div class="settings-grid">
    <div class="panel">
      <h3>Provider Vault</h3>
      <label>
        TorBox API Key
        <input type="password" bind:value={torboxApiKey} placeholder={account?.preferences.has_torbox_api_key ? 'Configured. Leave blank to keep.' : 'Paste key'} />
      </label>
      <label>
        Real Debrid API Key
        <input type="password" bind:value={realDebridApiKey} placeholder={account?.preferences.has_real_debrid_api_key ? 'Configured. Leave blank to keep.' : 'Paste key'} />
      </label>
      <label>
        Gemini API Key
        <input type="password" bind:value={geminiApiKey} placeholder={account?.preferences.has_gemini_api_key ? 'Configured. Leave blank to keep.' : 'Optional AI catalog key'} />
      </label>
      <button class="secondary" on:click={testProviders} disabled={isTestingProviders || !backendOnline}>
        {isTestingProviders ? 'Testing providers' : 'Test providers'}
      </button>

      {#if providerStatuses.length > 0}
        <div class="providers">
          {#each providerStatuses as provider}
            <div class:ok={provider.status === 'ok'} class:error={provider.status === 'error'} class="provider">
              <strong>{provider.provider}</strong>
              <span>{providerStateLabel(provider.status, provider.configured)}{provider.latency_ms ? ` · ${provider.latency_ms} ms` : ''}</span>
              <small>{provider.message}</small>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="panel">
      <h3>Smart Play</h3>
      <label>
        Maximum Resolution
        <select bind:value={maxResolution}>
          <option value="4K">4K</option>
          <option value="1080p">1080p</option>
          <option value="720p">720p</option>
        </select>
      </label>
      <label>
        Active Profile
        <select bind:value={profile}>
          <option value="home_theater">Home Theater</option>
          <option value="balanced">Balanced</option>
          <option value="mobile">Mobile</option>
          <option value="family">Family</option>
        </select>
      </label>
      <label>
        Language
        <select bind:value={preferredLanguage}>
          <option value="English">English</option>
          <option value="Spanish">Spanish</option>
          <option value="French">French</option>
          <option value="German">German</option>
          <option value="Japanese">Japanese</option>
        </select>
      </label>
      <label>
        Subtitles
        <select bind:value={subtitleMode}>
          <option value="auto">Automatic</option>
          <option value="always">Always on</option>
          <option value="forced">Forced only</option>
          <option value="off">Off</option>
        </select>
      </label>

      <div class="toggles">
        <label><input type="checkbox" bind:checked={preferHdr} /> Prefer HDR / Dolby Vision</label>
        <label><input type="checkbox" bind:checked={excludeAv1} /> Exclude AV1</label>
        <label><input type="checkbox" bind:checked={mobileDataSaver} /> Mobile data saver</label>
        <label><input type="checkbox" bind:checked={homeTheaterMode} /> Home theater priority</label>
        <label><input type="checkbox" bind:checked={familyMode} /> Family mode</label>
      </div>
    </div>
  </div>

  <div class="actions">
    {#if validationErrors.length > 0}
      <div class="validation">{validationErrors.join(' ')}</div>
    {/if}
    <button on:click={saveSettings} disabled={isSaving || validationErrors.length > 0 || !backendOnline}>
      {isSaving ? 'Saving' : 'Save cloud settings'}
    </button>
    {#if saveMessage}<span>{saveMessage}</span>{/if}
  </div>
</section>

<style>
  .shell {
    display: grid;
    gap: 1.25rem;
    max-width: 1120px;
  }

  .header {
    align-items: center;
    display: flex;
    justify-content: space-between;
    gap: 1rem;
  }

  h2,
  h3,
  p {
    margin: 0;
  }

  h2 {
    font-size: 2.3rem;
    font-weight: 650;
  }

  p,
  .eyebrow,
  small {
    color: #a7a7a7;
  }

  .status {
    border: 1px solid rgba(248, 113, 113, 0.5);
    border-radius: 999px;
    color: #fecaca;
    padding: 0.4rem 0.75rem;
  }

  .status.online {
    border-color: rgba(74, 222, 128, 0.5);
    color: #bbf7d0;
  }

  .notice,
  .validation {
    background: rgba(250, 204, 21, 0.12);
    border: 1px solid rgba(250, 204, 21, 0.35);
    border-radius: 8px;
    color: #fef3c7;
    padding: 0.85rem 1rem;
  }

  .account-grid,
  .settings-grid {
    display: grid;
    gap: 1rem;
  }

  .account-grid {
    grid-template-columns: 220px 260px minmax(0, 1fr);
  }

  .settings-grid {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }

  .panel {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    display: grid;
    gap: 0.9rem;
    padding: 1.1rem;
  }

  .panel h3 {
    font-size: 1.15rem;
  }

  .install input,
  label input,
  select {
    background: rgba(0, 0, 0, 0.42);
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 8px;
    color: #fff;
    font: inherit;
    min-height: 44px;
    padding: 0 0.8rem;
    width: 100%;
  }

  label {
    color: #d9d9d9;
    display: grid;
    gap: 0.4rem;
  }

  button {
    background: #fff;
    border: 0;
    border-radius: 8px;
    color: #000;
    cursor: pointer;
    font: inherit;
    font-weight: 700;
    min-height: 44px;
    padding: 0 1rem;
  }

  button.secondary {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .meter {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 999px;
    height: 8px;
    overflow: hidden;
  }

  .meter span {
    background: #4ade80;
    display: block;
    height: 100%;
  }

  .providers,
  .toggles {
    display: grid;
    gap: 0.55rem;
  }

  .provider {
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    display: grid;
    gap: 0.2rem;
    padding: 0.7rem;
  }

  .provider.ok {
    border-color: rgba(74, 222, 128, 0.35);
  }

  .provider.error {
    border-color: rgba(248, 113, 113, 0.35);
  }

  .actions {
    align-items: center;
    display: flex;
    flex-wrap: wrap;
    gap: 0.85rem;
  }

  @media (max-width: 900px) {
    .account-grid,
    .settings-grid {
      grid-template-columns: 1fr;
    }

    .header {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
