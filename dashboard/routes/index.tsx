export default function Home() {
  return (
    <div class="px-4 py-8 mx-auto relative z-10 flex flex-col items-center justify-center min-h-screen space-y-32 mb-20">
      
      {/* Hero Section */}
      <div class="max-w-4xl mx-auto text-center space-y-8 mt-20 animate-fade-in-up">
        <div class="inline-flex items-center px-4 py-2 rounded-full bg-white/5 border border-white/10 backdrop-blur-md text-sm font-medium text-indigo-300 mb-4 shadow-[0_0_15px_rgba(79,70,229,0.2)]">
          <span class="flex h-2 w-2 rounded-full bg-indigo-500 mr-2 animate-pulse"></span>
          Atlas Core v1.0 is Live
        </div>
        
        <h1 class="text-6xl md:text-8xl font-extrabold tracking-tight text-transparent bg-clip-text bg-gradient-to-br from-white via-white to-gray-400 drop-shadow-sm">
          Project <span class="bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 to-purple-500">Atlas</span>
        </h1>
        
        <p class="text-xl md:text-3xl text-gray-400 font-light max-w-2xl mx-auto leading-relaxed">
          The <span class="text-white font-medium">intelligence layer</span> for your media. Automatically resolve, rank, and stream with zero friction.
        </p>
        
        <div class="flex flex-col sm:flex-row gap-6 justify-center items-center mt-12 pt-8">
          <a
            href="/dashboard"
            class="group relative inline-flex items-center justify-center px-8 py-4 font-bold text-white bg-white/10 backdrop-blur-lg rounded-2xl overflow-hidden transition-all duration-300 hover:scale-105 hover:bg-white/15 border border-white/20 shadow-[0_0_40px_rgba(79,70,229,0.3)] hover:shadow-[0_0_60px_rgba(79,70,229,0.5)]"
          >
            <div class="absolute inset-0 bg-gradient-to-r from-indigo-500/40 to-purple-500/40 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <span class="relative flex items-center gap-2">
              Subscriber Dashboard
              <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
            </span>
          </a>
          
          <a
            href="/admin"
            class="group inline-flex items-center justify-center px-8 py-4 font-medium text-gray-300 bg-transparent rounded-2xl transition-all duration-300 hover:text-white hover:bg-white/5 border border-transparent hover:border-white/10"
          >
            Admin Access
          </a>
        </div>
      </div>

      {/* Features Section */}
      <div class="max-w-6xl mx-auto w-full grid grid-cols-1 md:grid-cols-3 gap-8 px-4">
        <div class="bg-white/5 border border-white/10 p-8 rounded-3xl backdrop-blur-md">
          <div class="p-3 bg-indigo-500/20 rounded-2xl w-fit mb-6">
            <svg class="w-8 h-8 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <h3 class="text-2xl font-semibold text-white mb-4">Zero-Friction Setup</h3>
          <p class="text-gray-400 leading-relaxed">
            One-click installation into Stremio. No more fiddling with complicated URLs, exposed API keys, or manual provider configuration.
          </p>
        </div>

        <div class="bg-white/5 border border-white/10 p-8 rounded-3xl backdrop-blur-md">
          <div class="p-3 bg-purple-500/20 rounded-2xl w-fit mb-6">
            <svg class="w-8 h-8 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
          </div>
          <h3 class="text-2xl font-semibold text-white mb-4">Intelligent Ranking</h3>
          <p class="text-gray-400 leading-relaxed">
            Atlas Core analyzes structural evidence, health, and speed to automatically rank the highest quality source tailored to your device profile.
          </p>
        </div>

        <div class="bg-white/5 border border-white/10 p-8 rounded-3xl backdrop-blur-md">
          <div class="p-3 bg-emerald-500/20 rounded-2xl w-fit mb-6">
            <svg class="w-8 h-8 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <h3 class="text-2xl font-semibold text-white mb-4">Privacy Preserving</h3>
          <p class="text-gray-400 leading-relaxed">
            Your integration keys are securely stored and never exposed to the client app. Telemetry is anonymized to protect your watch history.
          </p>
        </div>
      </div>

      {/* Providers Section */}
      <div class="max-w-4xl mx-auto text-center space-y-12">
        <div>
          <h2 class="text-3xl md:text-5xl font-bold text-white mb-4">Supported Integrations</h2>
          <p class="text-xl text-gray-400">Atlas connects directly with industry-leading Debrid and NAS services.</p>
        </div>
        
        <div class="flex flex-wrap justify-center gap-6">
          <div class="px-8 py-4 bg-white/5 border border-white/10 rounded-2xl backdrop-blur-sm">
            <span class="text-xl font-medium text-white">TorBox</span>
          </div>
          <div class="px-8 py-4 bg-white/5 border border-white/10 rounded-2xl backdrop-blur-sm">
            <span class="text-xl font-medium text-white">Real Debrid</span>
          </div>
          <div class="px-8 py-4 bg-white/5 border border-white/10 rounded-2xl backdrop-blur-sm opacity-50 cursor-not-allowed">
            <span class="text-xl font-medium text-gray-400">Local NAS (Coming Soon)</span>
          </div>
        </div>
      </div>

    </div>
  );
}
