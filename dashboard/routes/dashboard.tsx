import ConfigForm from "../islands/ConfigForm.tsx";

export default function SubscriberDashboard() {
  const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL") || "http://127.0.0.1:54321";
  const supabaseAnonKey = Deno.env.get("PUBLIC_SUPABASE_ANON_KEY") || "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImRlZmF1bHQiLCJyb2xlIjoiYW5vbiIsImlhdCI6MTY5NjQ1MjIyNSwiZXhwIjoyMDEwMDUyMjI1fQ.xxx";
  const gatewayUrl = Deno.env.get("PUBLIC_GATEWAY_URL") || "http://127.0.0.1:8080";

  return (
    <div class="px-4 py-12 mx-auto w-full max-w-4xl animate-fade-in-up">
      <div class="mb-10 text-left">
        <h1 class="text-3xl md:text-4xl font-bold text-white tracking-tight drop-shadow-sm">Subscriber Settings</h1>
        <p class="text-zinc-400 mt-2 text-base md:text-lg">Manage your integrations and stream preferences.</p>
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
