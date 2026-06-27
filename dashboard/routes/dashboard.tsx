import ConfigForm from "../islands/ConfigForm.tsx";

export default function SubscriberDashboard() {
  const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL") || "http://127.0.0.1:54321";
  const supabaseAnonKey = Deno.env.get("PUBLIC_SUPABASE_ANON_KEY") || "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImRlZmF1bHQiLCJyb2xlIjoiYW5vbiIsImlhdCI6MTY5NjQ1MjIyNSwiZXhwIjoyMDEwMDUyMjI1fQ.xxx";
  const gatewayUrl = Deno.env.get("PUBLIC_GATEWAY_URL") || "http://127.0.0.1:8080";

  return (
    <div class="relative z-10 px-4 py-16 mx-auto max-w-screen-md min-h-screen animate-fade-in-up">
      <div class="mb-12 text-center">
        <a href="/" class="inline-block text-indigo-400 hover:text-indigo-300 font-medium mb-6 transition-colors">
          &larr; Back to Home
        </a>
        <h1 class="text-4xl md:text-5xl font-extrabold text-white tracking-tight drop-shadow-sm">Subscriber Dashboard</h1>
        <p class="text-gray-400 mt-4 text-lg">Manage your integrations and stream preferences.</p>
      </div>
      
      <ConfigForm 
        projectId="atlas" 
        supabaseUrl={supabaseUrl} 
        supabaseAnonKey={supabaseAnonKey} 
        gatewayUrl={gatewayUrl} 
      />
    </div>
  );
}
