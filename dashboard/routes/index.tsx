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
      color: "emerald",
      title: "High-Speed Resolution",
      description:
        "We instantly parse your TorBox cache to serve lightning-fast streaming links without proxying media bytes.",
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
      color: "teal",
      title: "Smart Verification",
      description:
        "Atlas analyzes structural evidence to surface the most reliable, highest quality source for your device.",
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
      title: "Secure & Private",
      description:
        "API keys are heavily encrypted. All telemetry is 100% anonymized to protect your viewing habits.",
    },
  ];

  return (
    <div class="flex flex-col">
      {
        /* ═══════════════════════════════════════
          HERO SECTION
          ═══════════════════════════════════════ */
      }
      <section class="relative flex flex-col items-center justify-center min-h-[88vh] px-4 text-center overflow-hidden bg-[#12141b]">
        {/* Modern dark radial gradient background */}
        <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-emerald-900/10 via-[#12141b] to-[#12141b] pointer-events-none" />

        <div class="relative z-10 max-w-4xl mx-auto space-y-10 animate-fade-in-up">
          <div class="space-y-6">
            <h1 class="text-5xl sm:text-6xl md:text-7xl lg:text-8xl font-black tracking-tight leading-[1] text-white">
              Fast. Secure.<br />
              <span class="text-[#04BF8A]">High-speed streaming.</span>
            </h1>
            <p class="text-lg sm:text-xl md:text-2xl text-zinc-400 font-light max-w-2xl mx-auto leading-relaxed text-balance">
              Atlas brings modern, premium intelligence to your TorBox integration. Seamless and easy for anyone.
            </p>
          </div>

          <div class="flex flex-col sm:flex-row items-center justify-center gap-4 pt-4">
            <a
              href="/signup"
              class="btn-primary text-base px-8 py-4 rounded-2xl shadow-glow-md"
            >
              Get Started
            </a>
          </div>
        </div>
      </section>

      {
        /* ═══════════════════════════════════════
          FEATURES SECTION
          ═══════════════════════════════════════ */
      }
      <section class="px-4 py-24 sm:py-32 bg-[#12141b]">
        <div class="max-w-5xl mx-auto">
          <div class="text-center space-y-4 mb-16">
            <h2 class="text-3xl md:text-4xl font-bold text-white tracking-tight">
              A premium experience for TorBox.
            </h2>
            <p class="text-lg text-zinc-400 max-w-2xl mx-auto">
              Everything you need to upgrade your setup instantly.
            </p>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            {features.map((feature) => {
              const colorMap: Record<
                string,
                { icon: string; bg: string; border: string }
              > = {
                emerald: {
                  icon: "text-[#04BF8A]",
                  bg: "bg-[#04BF8A]/10",
                  border: "group-hover:border-[#04BF8A]/30",
                },
                teal: {
                  icon: "text-teal-400",
                  bg: "bg-teal-500/10",
                  border: "group-hover:border-teal-500/30",
                },
              };
              const c = colorMap[feature.color];

              return (
                <div
                  key={feature.title}
                  class={`glass-card group p-8 flex flex-col gap-5 ${c.border} transition-all duration-300 hover:-translate-y-1 bg-[#1a1c23]/80`}
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
          CTA SECTION
          ═══════════════════════════════════════ */
      }
      <section class="px-4 py-24 sm:py-32 bg-[#12141b] border-t border-white/[0.04]">
        <div class="max-w-4xl mx-auto text-center space-y-8">
          <h2 class="text-3xl md:text-5xl font-bold text-white tracking-tight">
            Ready for the next level?
          </h2>
          <p class="text-zinc-400 text-lg max-w-lg mx-auto">
            Takes under 60 seconds to configure your TorBox key and generate your secure install URL.
          </p>
          <a
            href="/dashboard"
            class="btn-primary text-base px-8 py-4 rounded-2xl shadow-glow-lg"
          >
            Configure Atlas
          </a>
        </div>
      </section>
    </div>
  );
}
