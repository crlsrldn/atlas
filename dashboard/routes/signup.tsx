import AuthForm from "../islands/AuthForm.tsx";

export default function Signup() {
  const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL") || "";
  const supabaseAnonKey = Deno.env.get("PUBLIC_SUPABASE_ANON_KEY") || "";

  return (
    <div class="flex-grow flex items-center justify-center p-4 min-h-[80vh]">
      <AuthForm
        type="signup"
        supabaseUrl={supabaseUrl}
        supabaseAnonKey={supabaseAnonKey}
      />
    </div>
  );
}
