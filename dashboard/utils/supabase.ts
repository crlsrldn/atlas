import { createClient } from "@supabase/supabase-js";

export function getSupabaseClient(supabaseUrl: string, supabaseAnonKey: string) {
  return createClient(supabaseUrl, supabaseAnonKey);
}
