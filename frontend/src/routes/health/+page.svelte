<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { BackendUnavailableError, backendFetch } from '$lib/backend';
  import Chart from 'chart.js/auto';

  let status = {
    metadata: 'Healthy',
    sources: 'Healthy',
    torbox: '0 ms',
    cache: '0%',
    subtitle: 'Healthy',
    startup: '0 sec'
  };
  let telemetryMessage = '';

  let chartCanvas: HTMLCanvasElement;
  let latencyChart: Chart;

  async function fetchTelemetry() {
    try {
      const res = await backendFetch('/telemetry/recent');
      if (!res.ok) {
        telemetryMessage = 'Telemetry is unavailable.';
        return;
      }

      const payload = await res.json();
      telemetryMessage = payload.message ?? '';

      let totalLatency = 0;
      let latencyCount = 0;
      let hashesChecked = 0;
      let hashesCached = 0;

      let chartLabels: string[] = [];
      let chartData: number[] = [];

      const events = [...(payload.events ?? [])].reverse();
      events.forEach((event) => {
        try {
          if (event.event === 'torbox_cache_check') {
            const data = event.data;
            totalLatency += data.latency_ms;
            latencyCount++;
            
            hashesChecked += data.hashes_checked;
            hashesCached += data.hashes_cached;

            chartLabels.push(new Date(event.timestamp).toLocaleTimeString());
            chartData.push(data.latency_ms);
          }
        } catch (e) {}
      });

      if (latencyCount > 0) {
        status.torbox = `${Math.round(totalLatency / latencyCount)} ms`;
      }
      
      if (hashesChecked > 0) {
        status.cache = `${Math.round((hashesCached / hashesChecked) * 100)}%`;
      }

      // Update Chart
      if (latencyChart) {
        latencyChart.data.labels = chartLabels;
        latencyChart.data.datasets[0].data = chartData;
        latencyChart.update();
      }

    } catch (e) {
      console.error("Failed to fetch telemetry:", e);
      telemetryMessage = e instanceof BackendUnavailableError
        ? 'Atlas backend is offline.'
        : 'Telemetry is unavailable.';
    }
  }

  let interval: ReturnType<typeof setInterval>;

  onMount(() => {
    // Initialize Chart
    latencyChart = new Chart(chartCanvas, {
      type: 'line',
      data: {
        labels: [],
        datasets: [{
          label: 'TorBox Latency (ms)',
          data: [],
          borderColor: '#4ade80',
          tension: 0.4
        }]
      },
      options: {
        responsive: true,
        plugins: {
          legend: { display: false }
        },
        scales: {
          y: { beginAtZero: true, grid: { color: 'rgba(255,255,255,0.1)' } },
          x: { grid: { display: false } }
        }
      }
    });

    fetchTelemetry();
    interval = setInterval(fetchTelemetry, 5000);
  });

  onDestroy(() => {
    clearInterval(interval);
    if (latencyChart) latencyChart.destroy();
  });
</script>

<div class="header">
  <h2>Health Dashboard</h2>
  <p>Real-time telemetry of your Atlas Core engines.</p>
</div>

{#if telemetryMessage}
  <div class="notice">{telemetryMessage}</div>
{/if}

<div class="grid">
  <div class="stat-card">
    <h4>Metadata Engine</h4>
    <div class="value success">{status.metadata}</div>
  </div>

  <div class="stat-card">
    <h4>Sources Engine</h4>
    <div class="value success">{status.sources}</div>
  </div>

  <div class="stat-card">
    <h4>Avg TorBox Latency</h4>
    <div class="value highlight">{status.torbox}</div>
  </div>

  <div class="stat-card">
    <h4>Avg Cache Hit Rate</h4>
    <div class="value highlight">{status.cache}</div>
  </div>
</div>

<div class="chart-container">
  <h4>TorBox Latency History</h4>
  <canvas bind:this={chartCanvas}></canvas>
</div>

<style>
  .header {
    margin-bottom: 3rem;
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

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1.5rem;
  }

  .stat-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1rem;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  h4 {
    color: #888;
    margin: 0 0 1rem 0;
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .value {
    font-size: 2rem;
    font-weight: 700;
  }

  .success {
    color: #4ade80;
  }

  .highlight {
    color: #fff;
  }

  .chart-container {
    margin-top: 2rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 2rem;
  }

  .notice {
    background: rgba(250, 204, 21, 0.12);
    border: 1px solid rgba(250, 204, 21, 0.35);
    border-radius: 8px;
    color: #fef3c7;
    margin-bottom: 1rem;
    padding: 0.85rem 1rem;
  }
</style>
