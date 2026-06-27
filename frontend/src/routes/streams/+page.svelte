<script lang="ts">
  import { BackendUnavailableError, backendFetch } from '$lib/backend';

  type StreamDetail = {
    title: string;
    provider_name: string;
    url: string;
    hash?: string;
    score: number;
    confidence: number;
    reasons: string[];
    resolution: string;
    video_codec: string;
    audio_codec?: string;
    audio_channels?: string;
    bitrate_mbps?: number;
    has_hdr: boolean;
    has_dolby_vision: boolean;
    has_subtitles: boolean;
    provider_latency_ms?: number;
    playback_successes: number;
    playback_failures: number;
  };

  let stremioId = 'tt0133093';
  let mediaType = 'movie';
  let streams: StreamDetail[] = [];
  let isLoading = false;
  let message = '';

  async function inspectStreams() {
    isLoading = true;
    message = '';
    streams = [];

    try {
      const res = await backendFetch(`/inspect/${mediaType}/${encodeURIComponent(stremioId)}.json`);
      if (!res.ok) {
        message = 'Stream inspection failed.';
        return;
      }
      const data = await res.json();
      streams = data.streams ?? [];
      if (streams.length === 0) {
        message = 'No streams found.';
      }
    } catch (err) {
      console.error(err);
      message = err instanceof BackendUnavailableError
        ? 'Atlas backend is offline.'
        : 'Stream inspection failed.';
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="header">
  <h2>Streams</h2>
  <p>Inspect ranked source decisions before playback.</p>
</div>

<div class="tool-row">
  <select bind:value={mediaType}>
    <option value="movie">Movie</option>
    <option value="series">Series</option>
  </select>
  <input bind:value={stremioId} placeholder="tt0133093 or tt0944947:1:2" />
  <button on:click={inspectStreams} disabled={isLoading}>
    {isLoading ? 'Inspecting...' : 'Inspect'}
  </button>
</div>

{#if message}
  <div class="notice">{message}</div>
{/if}

<div class="streams">
  {#each streams as stream}
    <div class="stream-row">
      <div class="rank">
        <strong>{stream.score}</strong>
        <span>{stream.confidence}%</span>
      </div>
      <div class="main">
        <h3>{stream.title}</h3>
        <div class="meta">
          <span>{stream.provider_name}</span>
          <span>{stream.resolution}</span>
          <span>{stream.video_codec}</span>
          {#if stream.audio_codec}<span>{stream.audio_codec}{stream.audio_channels ? ` ${stream.audio_channels}` : ''}</span>{/if}
          {#if stream.bitrate_mbps}<span>{stream.bitrate_mbps.toFixed(1)} Mbps</span>{/if}
          {#if stream.has_dolby_vision}<span>Dolby Vision</span>{:else if stream.has_hdr}<span>HDR</span>{/if}
          {#if stream.has_subtitles}<span>Subtitles</span>{/if}
          {#if stream.provider_latency_ms}<span>{stream.provider_latency_ms} ms</span>{/if}
        </div>
        <div class="reasons">{stream.reasons.join(' · ')}</div>
      </div>
      <div class="history">
        <span>{stream.playback_successes} ok</span>
        <span>{stream.playback_failures} fail</span>
      </div>
    </div>
  {/each}
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

  .tool-row {
    display: grid;
    grid-template-columns: 130px minmax(220px, 1fr) 130px;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
    max-width: 760px;
  }

  select,
  input,
  button {
    background: rgba(0, 0, 0, 0.45);
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 0.5rem;
    color: #fff;
    font-size: 1rem;
    padding: 0.8rem 1rem;
  }

  button {
    background: #fff;
    color: #000;
    cursor: pointer;
    font-weight: 700;
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.65;
  }

  .notice {
    color: #fecaca;
    margin-bottom: 1rem;
  }

  .streams {
    display: grid;
    gap: 0.8rem;
  }

  .stream-row {
    align-items: center;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 0.5rem;
    display: grid;
    gap: 1rem;
    grid-template-columns: 80px minmax(0, 1fr) 92px;
    padding: 1rem;
  }

  .rank strong,
  .rank span,
  .history span {
    display: block;
  }

  .rank strong {
    font-size: 1.45rem;
  }

  .rank span,
  .history,
  .reasons {
    color: #aaa;
    font-size: 0.85rem;
  }

  h3 {
    font-size: 1rem;
    margin: 0 0 0.45rem 0;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 0.45rem;
  }

  .meta span {
    background: rgba(255, 255, 255, 0.08);
    border-radius: 0.35rem;
    color: #ddd;
    font-size: 0.8rem;
    padding: 0.25rem 0.45rem;
  }
</style>
