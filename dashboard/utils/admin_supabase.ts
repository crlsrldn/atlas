import { createClient } from "https://esm.sh/@supabase/supabase-js@2.39.3";

export function getAdminSupabaseClient() {
  const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL");
  const supabaseServiceKey = Deno.env.get("SUPABASE_SERVICE_ROLE_KEY");

  if (!supabaseUrl || !supabaseServiceKey) {
    console.warn("Admin Supabase client missing URL or Service Role Key");
    return null;
  }

  // Use the service role key to bypass RLS for admin queries
  return createClient(supabaseUrl, supabaseServiceKey, {
    auth: {
      autoRefreshToken: false,
      persistSession: false,
    },
  });
}
