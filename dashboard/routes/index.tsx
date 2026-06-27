export default function Home() {
  const features = [
    {
      icon: (
        <svg
          class="w-6 h-6"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width={1.75}
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M13 10V3L4 14h7v7l9-11h-7z"
          />
        </svg>
      ),
      color: "indigo",
      title: "Zero-Friction Setup",
      description:
        "One-click installation into Stremio. No exposed API keys, no complex URLs, no manual configuration. Just press play.",
    },
    {
      icon: (
        <svg
          class="w-6 h-6"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width={1.75}
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
          />
        </svg>
      ),
      color: "purple",
      title: "Intelligent Ranking",
      description:
        "Atlas Core analyzes structural evidence, health, and speed to automatically surface the highest quality source for your device.",
    },
    {
      icon: (
        <svg
          class="w-6 h-6"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width={1.75}
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
          />
        </svg>
      ),
      color: "emerald",
      title: "Privacy First",
      description:
        "Your API keys are encrypted and never exposed to the client. Telemetry is anonymized to protect your viewing habits.",
    },
  ];

  const providers = [
    { name: "TorBox", status: "live", color: "emerald" },
    { name: "Real-Debrid", status: "live", color: "emerald" },
    { name: "AllDebrid", status: "soon", color: "amber" },
    { name: "Local NAS", status: "soon", color: "amber" },
    { name: "Plex", status: "soon", color: "amber" },
  ];

  const stats = [
    { label: "Providers Supported", value: "2+", sublabel: "& growing" },
    { label: "Cold Start Time", value: "<5s", sublabel: "guaranteed" },
    { label: "Source Selection", value: "<2s", sublabel: "to playback" },
    { label: "Privacy Level", value: "100%", sublabel: "anonymized" },
  ];

  return (
    <div class="flex flex-col">
      {
        /* ═══════════════════════════════════════
          HERO SECTION
          ═══════════════════════════════════════ */
      }
      <section class="relative flex flex-col items-center justify-center min-h-[88vh] px-4 text-center overflow-hidden">
        {/* Decorative orbs */}
        <div class="absolute top-1/4 left-1/4 w-96 h-96 bg-indigo-600/12 rounded-full blur-3xl pointer-events-none" />
        <div class="absolute bottom-1/4 right-1/4 w-80 h-80 bg-purple-600/10 rounded-full blur-3xl pointer-events-none" />

        {/* Grid overlay */}
        <div
          class="absolute inset-0 pointer-events-none opacity-[0.015]"
          style={{
            backgroundImage:
              "linear-gradient(rgba(255,255,255,0.5) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.5) 1px, transparent 1px)",
            backgroundSize: "60px 60px",
          }}
        />

        <div class="relative z-10 max-w-4xl mx-auto space-y-8 animate-fade-in-up">
          {/* Status pill */}
          <div class="flex justify-center">
            <div class="badge badge-indigo">
              <span class="w-1.5 h-1.5 rounded-full bg-indigo-400 animate-pulse" />
              Atlas Core v1.0 — Live
            </div>
          </div>

          {/* Headline */}
          <div class="space-y-4">
            <h1 class="text-5xl sm:text-6xl md:text-7xl lg:text-8xl font-black tracking-[-0.03em] leading-[0.95] text-white">
              The intelligence
              <br />
              layer for <span class="gradient-text">media</span>.
            </h1>
            <p class="text-lg sm:text-xl md:text-2xl text-zinc-400 font-light max-w-2xl mx-auto leading-relaxed text-balance">
              Automatically resolve, rank, and stream from the best source.{" "}
              <span class="text-zinc-300 font-medium">
                No friction. Zero exposure.
              </span>
            </p>
          </div>

          {/* CTA buttons */}
          <div class="flex flex-col sm:flex-row items-center justify-center gap-4 pt-4">
            <a
              href="/signup"
              class="btn-primary text-base px-8 py-4 rounded-2xl shadow-glow-md"
            >
              <svg
                class="w-5 h-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width={2}
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
              Configure My Setup
            </a>
          </div>

          {/* Scroll indicator */}
          <div class="flex justify-center pt-8 opacity-40">
            <div class="flex flex-col items-center gap-2 text-xs text-zinc-500">
              <svg
                class="w-4 h-4 animate-bounce"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </div>
          </div>
        </div>
      </section>

      {
        /* ═══════════════════════════════════════
          STATS STRIP
          ═══════════════════════════════════════ */
      }
      <section class="px-4 py-6">
        <div class="max-w-5xl mx-auto">
          <div class="glass-card p-6 sm:p-8 grid grid-cols-2 md:grid-cols-4 gap-6 divide-x divide-white/[0.06] divide-y md:divide-y-0 divide-y">
            {stats.map((stat, i) => (
              <div
                key={stat.label}
                class={`${i > 0 ? "pl-6 sm:pl-8" : ""} ${
                  i > 1 ? "pt-6 md:pt-0" : ""
                } flex flex-col gap-1`}
              >
                <p class="text-3xl sm:text-4xl font-black text-white tracking-tight">
                  {stat.value}
                </p>
                <p class="text-xs font-semibold text-zinc-400 uppercase tracking-wide">
                  {stat.label}
                </p>
                <p class="text-xs text-zinc-600">{stat.sublabel}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {
        /* ═══════════════════════════════════════
          FEATURES SECTION
          ═══════════════════════════════════════ */
      }
      <section class="px-4 py-24 sm:py-32">
        <div class="max-w-5xl mx-auto">
          {/* Section header */}
          <div class="text-center space-y-4 mb-16">
            <div class="flex justify-center">
              <div class="section-label">
                <svg
                  class="w-3.5 h-3.5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  stroke-width={2.5}
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"
                  />
                </svg>
                Why Atlas
              </div>
            </div>
            <h2 class="text-4xl md:text-5xl font-bold text-white tracking-tight">
              Built different.
            </h2>
            <p class="text-lg text-zinc-400 max-w-2xl mx-auto">
              Every decision in Atlas is made with one goal: get you to the best
              version of what you want to watch, instantly.
            </p>
          </div>

          {/* Feature cards */}
          <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
            {features.map((feature) => {
              const colorMap: Record<
                string,
                { icon: string; bg: string; border: string }
              > = {
                indigo: {
                  icon: "text-indigo-400",
                  bg: "bg-indigo-500/10",
                  border: "group-hover:border-indigo-500/30",
                },
                purple: {
                  icon: "text-purple-400",
                  bg: "bg-purple-500/10",
                  border: "group-hover:border-purple-500/30",
                },
                emerald: {
                  icon: "text-emerald-400",
                  bg: "bg-emerald-500/10",
                  border: "group-hover:border-emerald-500/30",
                },
              };
              const c = colorMap[feature.color];

              return (
                <div
                  key={feature.title}
                  class={`glass-card group p-8 flex flex-col gap-5 ${c.border} transition-all duration-300 hover:-translate-y-1`}
                >
                  <div class={`icon-box ${c.bg}`}>
                    <span class={c.icon}>{feature.icon}</span>
                  </div>
                  <div class="space-y-2">
                    <h3 class="text-lg font-semibold text-white">
                      {feature.title}
                    </h3>
                    <p class="text-zinc-400 text-sm leading-relaxed">
                      {feature.description}
                    </p>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {
        /* ═══════════════════════════════════════
          HOW IT WORKS
          ═══════════════════════════════════════ */
      }
      <section class="px-4 py-20 bg-white/[0.02] border-y border-white/[0.06]">
        <div class="max-w-5xl mx-auto">
          <div class="text-center space-y-4 mb-16">
            <div class="flex justify-center">
              <div class="section-label">How it works</div>
            </div>
            <h2 class="text-4xl md:text-5xl font-bold text-white tracking-tight">
              Three steps to perfection.
            </h2>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-8 relative">
            {/* Connecting line (desktop) */}
            <div class="hidden md:block absolute top-8 left-[calc(33%+1rem)] right-[calc(33%+1rem)] h-px bg-gradient-to-r from-indigo-500/30 via-purple-500/30 to-indigo-500/30" />

            {[
              {
                step: "01",
                title: "Connect",
                desc:
                  "Add your TorBox or Real-Debrid API key in the subscriber dashboard — takes 30 seconds.",
                icon: "🔑",
              },
              {
                step: "02",
                title: "Install",
                desc:
                  "Click 'Install Addon' to add your unique Atlas endpoint to Stremio instantly.",
                icon: "📲",
              },
              {
                step: "03",
                title: "Watch",
                desc:
                  "Atlas does the rest. It picks the best verified source and delivers it to your screen.",
                icon: "🎬",
              },
            ].map((item) => (
              <div
                key={item.step}
                class="flex flex-col items-center text-center gap-4"
              >
                <div class="relative">
                  <div class="w-16 h-16 rounded-2xl glass-card-strong flex items-center justify-center text-2xl">
                    {item.icon}
                  </div>
                  <div class="absolute -top-2 -right-2 w-6 h-6 rounded-full bg-indigo-600 border-2 border-[#09090b] flex items-center justify-center">
                    <span class="text-[9px] font-bold text-white">
                      {item.step}
                    </span>
                  </div>
                </div>
                <div class="space-y-2">
                  <h3 class="font-semibold text-white text-lg">{item.title}</h3>
                  <p class="text-zinc-400 text-sm leading-relaxed max-w-xs mx-auto">
                    {item.desc}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {
        /* ═══════════════════════════════════════
          INTEGRATIONS SECTION
          ═══════════════════════════════════════ */
      }
      <section class="px-4 py-24 sm:py-32">
        <div class="max-w-5xl mx-auto text-center space-y-12">
          <div class="space-y-4">
            <div class="flex justify-center">
              <div class="section-label">Integrations</div>
            </div>
            <h2 class="text-4xl md:text-5xl font-bold text-white tracking-tight">
              Works with your stack.
            </h2>
            <p class="text-zinc-400 max-w-xl mx-auto">
              Atlas connects with industry-leading Debrid services and NAS
              providers. More integrations ship every sprint.
            </p>
          </div>

          <div class="flex flex-wrap justify-center gap-3">
            {providers.map((p) => {
              const isLive = p.status === "live";
              return (
                <div
                  key={p.name}
                  class={`flex items-center gap-3 px-5 py-3 rounded-2xl border transition-all ${
                    isLive
                      ? "glass-card border-white/10 hover:border-emerald-500/30"
                      : "border-white/[0.04] bg-white/[0.02] opacity-50 cursor-not-allowed"
                  }`}
                >
                  <span
                    class={`w-2 h-2 rounded-full flex-shrink-0 ${
                      isLive ? "bg-emerald-400" : "bg-zinc-600"
                    } ${isLive ? "shadow-[0_0_6px_rgba(52,211,153,0.8)]" : ""}`}
                  />
                  <span
                    class={`font-medium text-sm ${
                      isLive ? "text-white" : "text-zinc-500"
                    }`}
                  >
                    {p.name}
                  </span>
                  {!isLive && (
                    <span class="badge badge-amber text-[10px] py-0.5 px-2">
                      Soon
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {
        /* ═══════════════════════════════════════
          CTA SECTION
          ═══════════════════════════════════════ */
      }
      <section class="px-4 pb-24 sm:pb-32">
        <div class="max-w-5xl mx-auto">
          <div class="relative overflow-hidden rounded-3xl">
            {/* Background */}
            <div class="absolute inset-0 bg-gradient-to-br from-indigo-600/20 via-purple-600/15 to-transparent" />
            <div class="absolute inset-0 border border-indigo-500/20 rounded-3xl" />
            <div class="absolute -top-20 -right-20 w-64 h-64 bg-indigo-500/20 rounded-full blur-3xl" />
            <div class="absolute -bottom-20 -left-20 w-64 h-64 bg-purple-500/15 rounded-full blur-3xl" />

            <div class="relative z-10 p-10 sm:p-16 flex flex-col md:flex-row items-center justify-between gap-8 text-center md:text-left">
              <div class="space-y-4 flex-1">
                <h2 class="text-3xl sm:text-4xl font-bold text-white">
                  Ready to upgrade your streaming?
                </h2>
                <p class="text-zinc-300 text-lg max-w-lg">
                  Set up takes under 2 minutes. Your keys are encrypted. Your
                  watch history is private.
                </p>
              </div>
              <div class="flex flex-col sm:flex-row gap-3 flex-shrink-0">
                <a
                  href="/dashboard"
                  class="btn-primary text-base px-8 py-4 rounded-2xl shadow-glow-lg"
                >
                  Get Started Free
                  <svg
                    class="w-5 h-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width={2}
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M13 7l5 5m0 0l-5 5m5-5H6"
                    />
                  </svg>
                </a>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
