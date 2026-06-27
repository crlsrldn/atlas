export default function AdminDashboard() {
  return (
    <div class="relative z-10 px-4 py-16 mx-auto max-w-screen-md min-h-screen animate-fade-in-up">
      <div class="mb-12 text-center">
        <a href="/" class="inline-block text-indigo-400 hover:text-indigo-300 font-medium mb-6 transition-colors">
          &larr; Back to Home
        </a>
        <h1 class="text-4xl md:text-5xl font-extrabold text-white tracking-tight drop-shadow-sm">Admin Dashboard</h1>
        <p class="text-gray-400 mt-4 text-lg">System metrics and overview.</p>
      </div>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div class="bg-white/5 border border-white/10 p-8 rounded-2xl shadow-lg backdrop-blur-md">
          <div class="flex items-center gap-3 mb-4">
            <div class="p-2 bg-indigo-500/20 rounded-lg">
              <svg class="w-6 h-6 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
            </div>
            <h2 class="text-xl font-medium text-gray-300">Total Users</h2>
          </div>
          <p class="text-5xl font-extrabold text-white">142</p>
        </div>
        
        <div class="bg-white/5 border border-white/10 p-8 rounded-2xl shadow-lg backdrop-blur-md">
          <div class="flex items-center gap-3 mb-4">
            <div class="p-2 bg-purple-500/20 rounded-lg">
              <svg class="w-6 h-6 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
            <h2 class="text-xl font-medium text-gray-300">Streams Resolved</h2>
          </div>
          <p class="text-5xl font-extrabold text-white">8,432</p>
        </div>
      </div>
    </div>
  );
}
