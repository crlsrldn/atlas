export default function Home() {
  return (
    <div class="px-4 py-8 mx-auto relative z-10 flex flex-col items-center justify-center min-h-screen">
      
      {/* Hero Section */}
      <div class="max-w-4xl mx-auto text-center space-y-8 animate-fade-in-up">
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
      
    </div>
  );
}
