export default function Home() {
  return (
    <div class="px-4 py-16 mx-auto relative z-10 flex flex-col items-center justify-start w-full max-w-7xl gap-y-32">
      
      {/* Hero Section */}
      <div class="max-w-4xl mx-auto text-center space-y-8 mt-12 md:mt-24 animate-fade-in-up">
        <div class="inline-flex items-center px-4 py-2 rounded-full bg-white/5 border border-white/10 backdrop-blur-md text-sm font-medium text-indigo-300 mb-2 shadow-[0_0_15px_rgba(79,70,229,0.15)]">
          <span class="flex h-2 w-2 rounded-full bg-indigo-500 mr-2 animate-pulse"></span>
          Atlas Core v1.0 is Live
        </div>
        
        <h1 class="text-5xl md:text-7xl font-extrabold tracking-tight text-transparent bg-clip-text bg-gradient-to-br from-white via-white to-zinc-400 drop-shadow-sm leading-[1.1]">
          Project <span class="bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 to-purple-500">Atlas</span>
        </h1>
        
        <p class="text-lg md:text-2xl text-zinc-400 font-light max-w-2xl mx-auto leading-relaxed">
          The <span class="text-white font-medium">intelligence layer</span> for your media. Automatically resolve, rank, and stream with zero friction.
        </p>
        
        <div class="flex justify-center pt-8">
          <a
            href="/dashboard"
            class="group relative inline-flex items-center justify-center px-8 py-4 text-lg font-semibold text-white bg-white/10 backdrop-blur-lg rounded-2xl overflow-hidden transition-all duration-300 hover:scale-105 hover:bg-white/15 border border-white/20 shadow-[0_0_30px_rgba(79,70,229,0.2)] hover:shadow-[0_0_50px_rgba(79,70,229,0.4)]"
          >
            <div class="absolute inset-0 bg-gradient-to-r from-indigo-500/40 to-purple-500/40 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <span class="relative flex items-center gap-2">
              Configure Integration
              <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
            </span>
          </a>
        </div>
      </div>

      {/* Features Section */}
      <div class="w-full grid grid-cols-1 md:grid-cols-3 gap-6">
        <div class="bg-white/5 hover:bg-white/10 border border-white/10 p-8 rounded-3xl backdrop-blur-md transition-colors group">
          <div class="p-3 bg-indigo-500/20 rounded-2xl w-fit mb-6 group-hover:scale-110 transition-transform">
            <svg class="w-7 h-7 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <h3 class="text-xl font-semibold text-white mb-3">Zero-Friction Setup</h3>
          <p class="text-zinc-400 leading-relaxed text-sm md:text-base">
            One-click installation into Stremio. No more fiddling with complicated URLs, exposed API keys, or manual provider configuration.
          </p>
        </div>

        <div class="bg-white/5 hover:bg-white/10 border border-white/10 p-8 rounded-3xl backdrop-blur-md transition-colors group">
          <div class="p-3 bg-purple-500/20 rounded-2xl w-fit mb-6 group-hover:scale-110 transition-transform">
            <svg class="w-7 h-7 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
          </div>
          <h3 class="text-xl font-semibold text-white mb-3">Intelligent Ranking</h3>
          <p class="text-zinc-400 leading-relaxed text-sm md:text-base">
            Atlas Core analyzes structural evidence, health, and speed to automatically rank the highest quality source tailored to your device profile.
          </p>
        </div>

        <div class="bg-white/5 hover:bg-white/10 border border-white/10 p-8 rounded-3xl backdrop-blur-md transition-colors group">
          <div class="p-3 bg-emerald-500/20 rounded-2xl w-fit mb-6 group-hover:scale-110 transition-transform">
            <svg class="w-7 h-7 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <h3 class="text-xl font-semibold text-white mb-3">Privacy Preserving</h3>
          <p class="text-zinc-400 leading-relaxed text-sm md:text-base">
            Your integration keys are securely stored and never exposed to the client app. Telemetry is anonymized to protect your watch history.
          </p>
        </div>
      </div>

      {/* Providers Section */}
      <div class="w-full text-center space-y-10 pb-16">
        <div>
          <h2 class="text-3xl md:text-4xl font-bold text-white mb-3">Supported Integrations</h2>
          <p class="text-lg text-zinc-400 max-w-2xl mx-auto">Atlas connects directly with industry-leading Debrid and NAS services securely via Appwrite.</p>
        </div>
        
        <div class="flex flex-wrap justify-center gap-4">
          <div class="px-6 py-3 bg-white/5 border border-white/10 rounded-xl backdrop-blur-sm flex items-center gap-3">
            <div class="w-2 h-2 rounded-full bg-green-500"></div>
            <span class="text-lg font-medium text-white">TorBox</span>
          </div>
          <div class="px-6 py-3 bg-white/5 border border-white/10 rounded-xl backdrop-blur-sm flex items-center gap-3">
            <div class="w-2 h-2 rounded-full bg-green-500"></div>
            <span class="text-lg font-medium text-white">Real Debrid</span>
          </div>
          <div class="px-6 py-3 bg-white/5 border border-white/10 rounded-xl backdrop-blur-sm opacity-50 cursor-not-allowed flex items-center gap-3">
            <div class="w-2 h-2 rounded-full bg-zinc-600"></div>
            <span class="text-lg font-medium text-zinc-400">Local NAS (Soon)</span>
          </div>
        </div>
      </div>

    </div>
  );
}
