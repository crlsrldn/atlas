export default function SubscriberDashboard() {
  return (
    <div class="px-4 py-8 mx-auto max-w-screen-md min-h-screen">
      <h1 class="text-4xl font-bold mb-8">Subscriber Dashboard</h1>
      
      <div class="bg-gray-100 p-6 rounded-lg mb-8">
        <h2 class="text-2xl font-semibold mb-4">Integrations</h2>
        
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">TorBox API Key</label>
          <input type="password" class="w-full px-4 py-2 border rounded-md" placeholder="Enter your TorBox API key" />
        </div>
        
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">RealDebrid API Key</label>
          <input type="password" class="w-full px-4 py-2 border rounded-md" placeholder="Enter your RealDebrid API key" />
        </div>

        <button class="bg-black text-white px-4 py-2 rounded-md hover:bg-gray-800 transition">Save Configurations</button>
      </div>

      <div class="bg-blue-50 border border-blue-200 p-6 rounded-lg">
        <h2 class="text-2xl font-semibold mb-2">Stremio Installation</h2>
        <p class="text-gray-700 mb-4">Click the button below to install Project Atlas to your Stremio.</p>
        <a 
          href="stremio://127.0.0.1:8080/stremio/demo-install-token/manifest.json" 
          class="inline-block bg-blue-600 text-white px-6 py-3 rounded-md font-semibold hover:bg-blue-700 transition"
        >
          Install on Stremio
        </a>
      </div>
    </div>
  );
}
