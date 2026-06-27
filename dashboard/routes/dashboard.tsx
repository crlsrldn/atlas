import ConfigForm from "../islands/ConfigForm.tsx";

export default function SubscriberDashboard() {
  const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL") || "http://127.0.0.1:54321";
  const supabaseAnonKey = Deno.env.get("PUBLIC_SUPABASE_ANON_KEY") || "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImRlZmF1bHQiLCJyb2xlIjoiYW5vbiIsImlhdCI6MTY5NjQ1MjIyNSwiZXhwIjoyMDEwMDUyMjI1fQ.xxx";
  const gatewayUrl = Deno.env.get("PUBLIC_GATEWAY_URL") || "http://127.0.0.1:8080";

  return (
    <div class="px-4 py-8 mx-auto max-w-screen-md min-h-screen">
      <h1 class="text-4xl font-bold mb-8">Subscriber Dashboard</h1>
      <ConfigForm 
        projectId="atlas" 
        supabaseUrl={supabaseUrl} 
        supabaseAnonKey={supabaseAnonKey} 
        gatewayUrl={gatewayUrl} 
      />
    </div>
  );
}
