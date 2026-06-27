<script lang="ts">
  import { onMount } from 'svelte';
  import { BackendUnavailableError, backendFetch, checkBackendHealth } from '$lib/backend';

  let torboxApiKey = '';
  let realDebridApiKey = '';
  let geminiApiKey = '';
  let maxResolution = '4K';
  let preferHdr = true;
  let excludeAv1 = false;
  let hasTorboxApiKey = false;
  let hasRealDebridApiKey = false;
  let hasGeminiApiKey = false;
  
  let isSaving = false;
  let saveMessage = '';
  let backendOnline = false;
  let loadMessage = 'Checking backend...';

  onMount(async () => {
    try {
      backendOnline = await checkBackendHealth();
      const res = await backendFetch('/user/preferences');
      if (res.ok) {
        const data = await res.json();
        torboxApiKey = data.torbox_api_key || '';
        realDebridApiKey = data.real_debrid_api_key || '';
        geminiApiKey = data.gemini_api_key || '';
        hasTorboxApiKey = data.has_torbox_api_key || false;
        hasRealDebridApiKey = data.has_real_debrid_api_key || false;
        hasGeminiApiKey = data.has_gemini_api_key || false;
        maxResolution = data.max_resolution || '4K';
        preferHdr = data.prefer_hdr ?? true;
        excludeAv1 = data.exclude_av1 ?? false;
        loadMessage = '';
      } else {
        loadMessage = 'Backend responded, but settings could not be loaded.';
      }
    } catch (err) {
      console.error("Could not load settings:", err);
      backendOnline = false;
      loadMessage = err instanceof BackendUnavailableError
        ? 'Atlas backend is offline. Start it with `cargo run` in the backend folder.'
        : 'Settings could not be loaded.';
    }
  });

  async function saveSettings() {
    isSaving = true;
    saveMessage = '';
    
    const payload = {
      torbox_api_key: torboxApiKey,
      real_debrid_api_key: realDebridApiKey,
      gemini_api_key: geminiApiKey,
      max_resolution: maxResolution,
      prefer_hdr: preferHdr,
      exclude_av1: excludeAv1,
    };

    try {
      const res = await backendFetch('/user/preferences', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });
      
      if (res.ok) {
        const data = await res.json();
        hasTorboxApiKey = data.has_torbox_api_key || false;
        hasRealDebridApiKey = data.has_real_debrid_api_key || false;
        hasGeminiApiKey = data.has_gemini_api_key || false;
        torboxApiKey = '';
        realDebridApiKey = '';
        geminiApiKey = '';
        saveMessage = 'Settings saved successfully.';
      } else {
        saveMessage = 'Failed to save settings.';
      }
    } catch (err) {
      console.error(err);
      saveMessage = err instanceof BackendUnavailableError
        ? 'Atlas backend is offline. Start it with `cargo run` in the backend folder.'
        : 'Error connecting to backend.';
    } finally {
      isSaving = false;
      setTimeout(() => saveMessage = '', 3000);
    }
  }
</script>

<div class="header">
  <h2>Settings</h2>
  <p>Configure your providers and playback preferences.</p>
</div>

<div class="settings-container">
  {#if loadMessage}
    <div class:online={backendOnline} class="backend-status">
      {loadMessage}
    </div>
  {/if}

  <div class="section">
    <h3>Providers</h3>
    <div class="form-group">
      <label for="torbox-key">TorBox API Key</label>
      <input type="password" id="torbox-key" bind:value={torboxApiKey} placeholder="Paste your TorBox API key here" />
      <small>{hasTorboxApiKey ? 'A TorBox key is configured. Leave blank to keep it.' : 'Your TorBox key is stored locally and never returned by the API.'}</small>
    </div>
    <div class="form-group">
      <label for="rd-key">Real Debrid API Key</label>
      <input type="password" id="rd-key" bind:value={realDebridApiKey} placeholder="Paste your Real Debrid API key here" />
      <small>{hasRealDebridApiKey ? 'A Real Debrid key is configured. Leave blank to keep it.' : 'Your Real Debrid key is stored locally and never returned by the API.'}</small>
    </div>
    <div class="form-group">
      <label for="gemini-key">Gemini API Key</label>
      <input type="password" id="gemini-key" bind:value={geminiApiKey} placeholder="Paste your Gemini API key here" />
      <small>{hasGeminiApiKey ? 'A Gemini key is configured. Leave blank to keep it.' : 'Used for the Atlas AI Recommendations catalog. Stored locally and never returned by the API.'}</small>
    </div>
  </div>

  <div class="section">
    <h3>Smart Play Preferences</h3>
    
    <div class="form-group">
      <label for="resolution">Maximum Resolution</label>
      <select id="resolution" bind:value={maxResolution}>
        <option value="4K">4K (2160p)</option>
        <option value="1080p">1080p</option>
        <option value="720p">720p</option>
      </select>
    </div>

    <div class="toggle-group">
      <label class="toggle">
        <input type="checkbox" bind:checked={preferHdr} />
        <span class="slider"></span>
        <span class="label-text">Prefer HDR / Dolby Vision</span>
      </label>
      <small>Atlas will prioritize HDR releases when available.</small>
    </div>

    <div class="toggle-group">
      <label class="toggle">
        <input type="checkbox" bind:checked={excludeAv1} />
        <span class="slider"></span>
        <span class="label-text">Exclude AV1 Codec</span>
      </label>
      <small>Enable this if your primary playback device cannot hardware-decode AV1.</small>
    </div>
  </div>

  <div class="actions">
    <button class="btn-primary" on:click={saveSettings} disabled={isSaving}>
      {isSaving ? 'Saving...' : 'Save Settings'}
    </button>
    {#if saveMessage}
      <span class="msg">{saveMessage}</span>
    {/if}
  </div>
</div>

<style>
  .header {
    margin-bottom: 2rem;
  }

  h2 {
    font-size: 2.5rem;
    font-weight: 600;
    margin: 0 0 0.5rem 0;
  }

  .header p {
    color: #888;
    font-size: 1.1rem;
    margin: 0;
  }

  .settings-container {
    max-width: 600px;
  }

  .backend-status {
    background: rgba(248, 113, 113, 0.12);
    border: 1px solid rgba(248, 113, 113, 0.35);
    border-radius: 0.5rem;
    color: #fecaca;
    margin-bottom: 1rem;
    padding: 0.85rem 1rem;
  }

  .backend-status.online {
    background: rgba(74, 222, 128, 0.12);
    border-color: rgba(74, 222, 128, 0.35);
    color: #bbf7d0;
  }

  .section {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1rem;
    padding: 2rem;
    margin-bottom: 2rem;
  }

  h3 {
    margin-top: 0;
    margin-bottom: 1.5rem;
    font-size: 1.2rem;
    border-bottom: 1px solid rgba(255,255,255,0.1);
    padding-bottom: 0.5rem;
  }

  .form-group {
    margin-bottom: 1.5rem;
    display: flex;
    flex-direction: column;
  }

  label {
    font-weight: 500;
    margin-bottom: 0.5rem;
  }

  input[type="password"], select {
    background: rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #fff;
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    font-size: 1rem;
    outline: none;
    transition: border-color 0.2s;
  }

  input[type="password"]:focus, select:focus {
    border-color: #fff;
  }

  small {
    color: #888;
    margin-top: 0.5rem;
    font-size: 0.85rem;
  }

  .toggle-group {
    margin-bottom: 1.5rem;
  }

  .toggle {
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .toggle input {
    display: none;
  }

  .slider {
    width: 40px;
    height: 20px;
    background-color: rgba(255, 255, 255, 0.2);
    border-radius: 20px;
    position: relative;
    transition: 0.3s;
    margin-right: 10px;
  }

  .slider::before {
    content: "";
    position: absolute;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background-color: #fff;
    top: 2px;
    left: 2px;
    transition: 0.3s;
  }

  input:checked + .slider {
    background-color: #fff;
  }

  input:checked + .slider::before {
    transform: translateX(20px);
    background-color: #000;
  }

  .label-text {
    font-weight: 500;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .btn-primary {
    background: #fff;
    color: #000;
    border: none;
    padding: 0.8rem 2rem;
    border-radius: 2rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: transform 0.2s ease;
  }

  .btn-primary:hover {
    transform: scale(1.05);
  }

  .btn-primary:disabled {
    opacity: 0.7;
    cursor: not-allowed;
    transform: none;
  }

  .msg {
    color: #4ade80;
    font-weight: 500;
  }
</style>
