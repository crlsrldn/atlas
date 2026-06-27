<script lang="ts">
  import { onMount } from 'svelte';
  import { BackendUnavailableError, backendFetch, checkBackendHealth } from '$lib/backend';

  type StreamDetail = {
    title: string;
    provider_name: string;
    url: string;
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
  };

  let stremioId = 'tt0133093';
  let mediaType = 'movie';
  let streams: StreamDetail[] = [];
  let backendOnline = false;
  let isLoading = false;
  let message = 'Checking backend...';

  $: bestStream = streams[0];
  $: nextStreams = streams.slice(1, 4);

  onMount(async () => {
    try {
      backendOnline = await checkBackendHealth();
      message = backendOnline ? '' : 'Atlas backend is offline.';
    } catch (err) {
      backendOnline = false;
      message = err instanceof BackendUnavailableError ? 'Atlas backend is offline.' : 'Backend check failed.';
    }
  });

  async function smartPlay() {
    isLoading = true;
    message = '';
    streams = [];

    try {
      const res = await backendFetch(`/inspect/${mediaType}/${encodeURIComponent(stremioId)}.json`);
      if (!res.ok) {
        message = 'Smart Play could not resolve this title.';
        return;
      }

      const data = await res.json();
      streams = data.streams ?? [];
      message = streams.length === 0 ? 'No playable streams found.' : '';
    } catch (err) {
      console.error(err);
      message = err instanceof BackendUnavailableError
        ? 'Atlas backend is offline.'
        : 'Smart Play failed.';
    } finally {
      isLoading = false;
    }
  }

  function play(stream: StreamDetail) {
    window.open(stream.url, '_blank', 'noopener,noreferrer');
  }
</script>

<section class="surface">
  <div class="command">
    <div class="heading">
      <div class:online={backendOnline} class="status-dot"></div>
      <div>
        <h2>Smart Play</h2>
        <p>{backendOnline ? 'Atlas is ready to resolve playback.' : 'Start the backend to resolve playback.'}</p>
      </div>
    </div>

    <div class="controls">
      <select bind:value={mediaType} aria-label="Media type">
        <option value="movie">Movie</option>
        <option value="series">Series</option>
      </select>
      <input bind:value={stremioId} aria-label="Stremio ID" placeholder="tt0133093 or tt0944947:1:2" />
      <button on:click={smartPlay} disabled={isLoading || !backendOnline || !stremioId.trim()}>
        {isLoading ? 'Resolving' : 'Resolve'}
      </button>
    </div>
  </div>

  {#if message}
    <div class="notice">{message}</div>
  {/if}

  {#if bestStream}
    <div class="recommendation">
      <div class="score">
        <strong>{bestStream.score}</strong>
        <span>{bestStream.confidence}% confidence</span>
      </div>
      <div class="stream-main">
        <span class="eyebrow">Best match</span>
        <h3>{bestStream.title}</h3>
        <div class="meta">
          <span>{bestStream.provider_name}</span>
          <span>{bestStream.resolution}</span>
          <span>{bestStream.video_codec}</span>
          {#if bestStream.audio_codec}<span>{bestStream.audio_codec}{bestStream.audio_channels ? ` ${bestStream.audio_channels}` : ''}</span>{/if}
          {#if bestStream.bitrate_mbps}<span>{bestStream.bitrate_mbps.toFixed(1)} Mbps</span>{/if}
          {#if bestStream.has_dolby_vision}<span>Dolby Vision</span>{:else if bestStream.has_hdr}<span>HDR</span>{/if}
          {#if bestStream.has_subtitles}<span>Subtitles</span>{/if}
          {#if bestStream.provider_latency_ms}<span>{bestStream.provider_latency_ms} ms</span>{/if}
        </div>
        <div class="reasons">{bestStream.reasons.join(' · ')}</div>
      </div>
      <button class="play" on:click={() => play(bestStream)}>Play</button>
    </div>
  {/if}

  {#if nextStreams.length > 0}
    <div class="queue">
      <h3>Fallbacks</h3>
      {#each nextStreams as stream}
        <button class="queue-row" on:click={() => play(stream)}>
          <span>{stream.provider_name}</span>
          <strong>{stream.title}</strong>
          <small>{stream.resolution} · {stream.confidence}% confidence</small>
        </button>
      {/each}
    </div>
  {/if}
</section>

<style>
  .surface {
    display: grid;
    gap: 1rem;
    max-width: 980px;
  }

  .command,
  .recommendation,
  .queue {
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 1.25rem;
  }

  .heading {
    align-items: center;
    display: flex;
    gap: 0.9rem;
    margin-bottom: 1.25rem;
  }

  .status-dot {
    background: #f87171;
    border-radius: 50%;
    box-shadow: 0 0 0 5px rgba(248, 113, 113, 0.12);
    height: 12px;
    width: 12px;
  }

  .status-dot.online {
    background: #4ade80;
    box-shadow: 0 0 0 5px rgba(74, 222, 128, 0.12);
  }

  h2,
  h3,
  p {
    margin: 0;
  }

  h2 {
    font-size: 2rem;
    font-weight: 650;
  }

  .heading p,
  .reasons,
  .queue-row small {
    color: #a8a8a8;
  }

  .controls {
    display: grid;
    gap: 0.75rem;
    grid-template-columns: 130px minmax(240px, 1fr) 120px;
  }

  select,
  input,
  button {
    border-radius: 8px;
    font: inherit;
    min-height: 46px;
  }

  select,
  input {
    background: rgba(0, 0, 0, 0.42);
    border: 1px solid rgba(255, 255, 255, 0.18);
    color: #fff;
    padding: 0 0.9rem;
  }

  button {
    border: 0;
    cursor: pointer;
    font-weight: 700;
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .notice {
    background: rgba(250, 204, 21, 0.12);
    border: 1px solid rgba(250, 204, 21, 0.35);
    border-radius: 8px;
    color: #fef3c7;
    padding: 0.85rem 1rem;
  }

  .recommendation {
    align-items: center;
    display: grid;
    gap: 1rem;
    grid-template-columns: 96px minmax(0, 1fr) 110px;
  }

  .score strong,
  .score span {
    display: block;
  }

  .score strong {
    font-size: 2.2rem;
  }

  .score span,
  .eyebrow {
    color: #a8a8a8;
    font-size: 0.85rem;
  }

  .stream-main h3 {
    font-size: 1.15rem;
    margin: 0.2rem 0 0.55rem;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 0.55rem;
  }

  .meta span {
    background: rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    color: #e2e2e2;
    font-size: 0.82rem;
    padding: 0.25rem 0.45rem;
  }

  .play,
  .controls button {
    background: #fff;
    color: #000;
  }

  .queue {
    display: grid;
    gap: 0.65rem;
  }

  .queue h3 {
    color: #dcdcdc;
    font-size: 1rem;
    margin-bottom: 0.2rem;
  }

  .queue-row {
    background: rgba(255, 255, 255, 0.055);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: #fff;
    display: grid;
    gap: 0.25rem;
    grid-template-columns: 120px minmax(0, 1fr) 160px;
    padding: 0.75rem 0.9rem;
    text-align: left;
  }

  .queue-row strong,
  .queue-row small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (max-width: 760px) {
    .controls,
    .recommendation,
    .queue-row {
      grid-template-columns: 1fr;
    }

    .play {
      width: 100%;
    }
  }
</style>
