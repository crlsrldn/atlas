import { createClient } from "@supabase/supabase-js";

// Note: In a real app, these should come from environment variables.
// For MVP we can inject them at build time or use public placeholders.
const supabaseUrl = "http://127.0.0.1:54321"; // Supabase Local URL Placeholder
const supabaseAnonKey = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImRlZmF1bHQiLCJyb2xlIjoiYW5vbiIsImlhdCI6MTY5NjQ1MjIyNSwiZXhwIjoyMDEwMDUyMjI1fQ.xxx";

export const supabase = createClient(supabaseUrl, supabaseAnonKey);
