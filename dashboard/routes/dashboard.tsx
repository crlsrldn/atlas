import ConfigForm from "../islands/ConfigForm.tsx";

export default function SubscriberDashboard() {
  const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL") ||
    "http://127.0.0.1:54321";
  const supabaseAnonKey = Deno.env.get("PUBLIC_SUPABASE_ANON_KEY") ||
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImRlZmF1bHQiLCJyb2xlIjoiYW5vbiIsImlhdCI6MTY5NjQ1MjIyNSwiZXhwIjoyMDEwMDUyMjI1fQ.xxx";
  const gatewayUrl = Deno.env.get("PUBLIC_GATEWAY_URL") ||
    "http://127.0.0.1:8080";

  return (
    <div class="px-4 py-12 mx-auto w-full max-w-3xl animate-fade-in-up">
      {/* Page header */}
      <div class="mb-10">
        <div class="flex items-center gap-3 mb-6">
          <div class="icon-box bg-indigo-500/10">
            <svg class="w-5 h-5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={1.75}>
              <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </div>
          <div>
            <p class="text-xs font-semibold text-zinc-500 uppercase tracking-widest mb-0.5">
              My Setup
            </p>
            <h1 class="text-2xl md:text-3xl font-bold text-white tracking-tight">
              Subscriber Settings
            </h1>
          </div>
        </div>
        <p class="text-zinc-400 text-base leading-relaxed">
          Connect your debrid providers and configure your playback preferences. Your API keys are encrypted end-to-end.
        </p>

        {/* Security note */}
        <div class="mt-5 flex items-center gap-2.5 text-xs text-zinc-500">
          <svg class="w-3.5 h-3.5 text-emerald-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2.5}>
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
          </svg>
          <span>Keys are encrypted server-side and never sent to the client.</span>
        </div>
      </div>

      {/* The main form island */}
      <ConfigForm
        projectId="atlas"
        supabaseUrl={supabaseUrl}
        supabaseAnonKey={supabaseAnonKey}
        gatewayUrl={gatewayUrl}
      />
    </div>
  );
}
