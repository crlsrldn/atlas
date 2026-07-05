<script lang="ts">
  import type { PageData } from './$types';
  import AdminLogin from '$lib/components/AdminLogin.svelte';
  import Chart from 'chart.js/auto';

  let { data }: { data: PageData } = $props();

  let resolutionCanvas: HTMLCanvasElement;
  let latencyCanvas: HTMLCanvasElement;
  let playbackCanvas: HTMLCanvasElement;

  $effect(() => {
    if (data.analytics && typeof window !== 'undefined') {
      const commonOptions = {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            labels: { color: '#a1a1aa', font: { family: 'inherit' } }
          }
        },
        scales: {
          x: { ticks: { color: '#a1a1aa' }, grid: { color: '#ffffff10' } },
          y: { ticks: { color: '#a1a1aa' }, grid: { color: '#ffffff10' } }
        }
      };

      if (resolutionCanvas) {
        new Chart(resolutionCanvas, {
          type: 'doughnut',
          data: {
            labels: Object.keys(data.analytics.resolution),
            datasets: [{
              data: Object.values(data.analytics.resolution),
              backgroundColor: ['#10b981', '#6366f1', '#a855f7', '#3f3f46'],
              borderWidth: 0,
              hoverOffset: 4
            }]
          },
          options: {
            responsive: true,
            maintainAspectRatio: false,
            cutout: '75%',
            plugins: {
              legend: { position: 'bottom', labels: { color: '#a1a1aa', padding: 20, usePointStyle: true } }
            }
          }
        });
      }

      if (latencyCanvas) {
        new Chart(latencyCanvas, {
          type: 'line',
          data: {
            labels: data.analytics.latencyTimeline.map((t: any) => t.time),
            datasets: [{
              label: 'Resolution Latency (ms)',
              data: data.analytics.latencyTimeline.map((t: any) => t.latency),
              borderColor: '#10b981',
              backgroundColor: 'rgba(16, 185, 129, 0.1)',
              fill: true,
              tension: 0.4,
              borderWidth: 2,
              pointRadius: 0,
              pointHitRadius: 10
            }]
          },
          options: {
            ...commonOptions,
            plugins: { legend: { display: false } },
            interaction: { intersect: false, mode: 'index' }
          }
        });
      }

      if (playbackCanvas) {
        new Chart(playbackCanvas, {
          type: 'bar',
          data: {
            labels: ['Success', 'Failure'],
            datasets: [{
              data: [data.analytics.playback.success, data.analytics.playback.failure],
              backgroundColor: ['#10b981', '#ef4444'],
              borderRadius: 6,
              barThickness: 40
            }]
          },
          options: {
            ...commonOptions,
            plugins: { legend: { display: false } }
          }
        });
      }
    }
  });

  const colorMap: Record<string, { bg: string; icon: string; badge: string; glow: string }> = {
    indigo: {
      bg: "bg-indigo-500/10",
      icon: "text-indigo-400",
      badge: "text-indigo-300 bg-indigo-500/10 border-indigo-500/20",
      glow: "hover:border-indigo-500/30",
    },
    purple: {
      bg: "bg-purple-500/10",
      icon: "text-purple-400",
      badge: "text-purple-300 bg-purple-500/10 border-purple-500/20",
      glow: "hover:border-purple-500/30",
    },
    emerald: {
      bg: "bg-emerald-500/10",
      icon: "text-emerald-400",
      badge: "text-emerald-300 bg-emerald-500/10 border-emerald-500/20",
      glow: "hover:border-emerald-500/30",
    },
    amber: {
      bg: "bg-amber-500/10",
      icon: "text-amber-400",
      badge: "text-amber-300 bg-amber-500/10 border-amber-500/20",
      glow: "hover:border-amber-500/30",
    },
  };
</script>

<svelte:head>
  <title>System Console — Atlas</title>
</svelte:head>

{#if data.needsLogin}
  <div class="min-h-[80vh] flex flex-col items-center justify-center px-4 py-16 animate-fade-in-up">
    <div class="w-full max-w-sm">
      <div class="text-center mb-8 space-y-2">
        <div class="section-label mx-auto w-fit mb-4">Admin Access</div>
        <h1 class="text-3xl font-bold text-zinc-900 dark:text-white tracking-tight">
          System Console
        </h1>
        <p class="text-zinc-500 dark:text-zinc-400 text-sm">
          Sign in to view live Atlas Core metrics.
        </p>
      </div>

      {#if data.error}
        <div class="alert alert-error mb-6">
          <div class="flex-1">{data.error}</div>
        </div>
      {/if}

      <div class="glass-card-strong p-8 rounded-2xl">
        <AdminLogin />
      </div>
    </div>
  </div>
{:else if data.error}
  <div class="min-h-[80vh] flex flex-col items-center justify-center px-4">
    <div class="glass-card-strong p-8 rounded-2xl max-w-md w-full text-center space-y-4 border-red-500/30">
      <h2 class="text-xl font-bold text-zinc-900 dark:text-white">
        Access Error
      </h2>
      <p class="text-zinc-500 dark:text-zinc-400">{data.error}</p>
      <form method="POST" action="?/logout">
        <button type="submit" class="btn-primary mt-4">Sign Out</button>
      </form>
    </div>
  </div>
{:else}
  <div class="px-4 py-12 mx-auto w-full max-w-5xl animate-fade-in-up">
    <!-- Page header -->
    <div class="mb-10 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
      <div class="flex items-center gap-3">
        <div class="icon-box bg-indigo-500/10">
          <svg class="w-5 h-5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
          </svg>
        </div>
        <div>
          <p class="text-xs font-semibold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest mb-0.5">
            Admin
          </p>
          <h1 class="text-2xl md:text-3xl font-bold text-zinc-900 dark:text-white tracking-tight">
            System Overview
          </h1>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <!-- Live indicator -->
        <div class="flex items-center gap-2 px-4 py-2 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
          <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse shadow-[0_0_8px_rgba(52,211,153,0.8)]"></span>
          <span class="text-xs font-semibold text-emerald-300">Live Data</span>
        </div>
      </div>
    </div>

    <!-- Metric cards grid -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5 mb-10">
      {#each [
        { label: "Total Users", value: (data.totalUsers ?? 0).toLocaleString(), color: "indigo", description: "Registered subscribers" },
        { label: "Streams Resolved", value: (data.streamsResolved ?? 0).toLocaleString(), color: "purple", description: "Successful stream events" },
        { label: "Success Rate", value: data.successRate, color: "emerald", description: "Stream resolution success" },
        { label: "Avg. Resolution", value: data.avgLatency, color: "amber", description: "Source selection speed" }
      ] as m}
        {@const c = colorMap[m.color]}
        <div class={`stat-card border border-black/5 dark:border-white/[0.07] transition-all duration-200 ${c.glow} hover:-translate-y-0.5`}>
          <div class="flex items-center justify-between mb-5">
            <div class={`icon-box icon-box-sm ${c.bg}`}>
              <!-- Simplified icons for port -->
              <span class={`w-5 h-5 ${c.icon}`}>■</span>
            </div>
          </div>
          <p class="text-4xl font-black text-zinc-900 dark:text-white tracking-tight tabular-nums">
            {m.value}
          </p>
          <p class="text-sm font-medium text-zinc-500 dark:text-zinc-400 mt-1">
            {m.label}
          </p>
          <p class="text-xs text-zinc-500 dark:text-zinc-600 mt-1">
            {m.description}
          </p>
        </div>
      {/each}
    </div>

    <!-- Analytics Section -->
    <div class="mb-10 space-y-5">
      <div class="flex items-center gap-3 mb-6">
        <h2 class="text-xl font-bold text-zinc-900 dark:text-white">Telemetry & Analytics</h2>
      </div>
      
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-5">
        <!-- Resolution Distribution -->
        <div class="glass-card-strong p-6 rounded-2xl flex flex-col">
          <h3 class="text-sm font-semibold text-zinc-900 dark:text-white mb-6">Resolution Demand</h3>
          <div class="relative flex-grow min-h-[250px]">
            <canvas bind:this={resolutionCanvas}></canvas>
          </div>
        </div>

        <!-- Latency Timeline -->
        <div class="glass-card-strong p-6 rounded-2xl flex flex-col lg:col-span-2">
          <h3 class="text-sm font-semibold text-zinc-900 dark:text-white mb-6">Provider Latency</h3>
          <div class="relative flex-grow min-h-[250px]">
            <canvas bind:this={latencyCanvas}></canvas>
          </div>
        </div>

        <!-- Playback Reliability -->
        <div class="glass-card-strong p-6 rounded-2xl flex flex-col lg:col-span-3">
          <h3 class="text-sm font-semibold text-zinc-900 dark:text-white mb-6">Playback Reliability</h3>
          <div class="relative flex-grow min-h-[250px]">
            <canvas bind:this={playbackCanvas}></canvas>
          </div>
        </div>
      </div>
    </div>

    <!-- Secondary info section -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
      <!-- System Status -->
      <div class="glass-card-strong p-6 rounded-2xl">
        <div class="flex items-center gap-3 mb-5">
          <div class="icon-box icon-box-sm bg-emerald-500/10">
            <svg class="w-4 h-4 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M5.636 18.364a9 9 0 010-12.728m12.728 0a9 9 0 010 12.728m-9.9-2.829a5 5 0 010-7.07m7.072 0a5 5 0 010 7.07M13 12a1 1 0 11-2 0 1 1 0 012 0z" />
            </svg>
          </div>
          <h2 class="font-semibold text-zinc-900 dark:text-white">System Health</h2>
        </div>

        <div class="space-y-3">
          {#each [
            { name: "Atlas Gateway", status: "Operational", color: "emerald" },
            { name: "Supabase DB", status: "Operational", color: "emerald" },
            ...(data.providers || []).map((p: any) => ({
              name: p.name,
              status: p.latencyMs ? `${p.status} (${p.latencyMs}ms)` : p.status,
              color: p.color,
            }))
          ] as s}
            <div class="flex items-center justify-between py-2 border-b border-black/5 dark:border-white/[0.04] last:border-0">
              <span class="text-sm text-zinc-700 dark:text-zinc-300">{s.name}</span>
              <div class="flex items-center gap-2">
                <span class={`w-1.5 h-1.5 rounded-full ${s.color === "emerald" ? "bg-emerald-400" : "bg-amber-400"}`}></span>
                <span class={`text-xs font-medium ${s.color === "emerald" ? "text-emerald-400" : "text-amber-400"}`}>
                  {s.status}
                </span>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <div class="glass-card-strong p-6 rounded-2xl flex flex-col">
        <h3 class="text-sm font-semibold text-zinc-900 dark:text-white mb-3">
          Account
        </h3>
        <div class="flex-grow"></div>
        <form method="POST" action="?/logout">
          <button
            type="submit"
            class="flex items-center justify-center gap-2 px-4 py-2 bg-red-50 dark:bg-red-500/10 hover:bg-red-100 dark:hover:bg-red-500/20 text-red-600 dark:text-red-400 text-sm font-medium rounded-xl border border-red-200 dark:border-red-500/20 shadow-sm transition-all duration-200 w-full"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
            </svg>
            Sign Out of Console
          </button>
        </form>
      </div>
    </div>
  </div>
{/if}
