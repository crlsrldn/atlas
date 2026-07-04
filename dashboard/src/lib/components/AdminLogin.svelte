<script lang="ts">
  import { env } from '$env/dynamic/public';
  import { createBrowserClient } from '@supabase/ssr';

  let supabaseUrl = env.PUBLIC_SUPABASE_URL || '';
  let supabaseAnonKey = env.PUBLIC_SUPABASE_ANON_KEY || '';

  const supabase = createBrowserClient(supabaseUrl, supabaseAnonKey);

  let email = $state('');
  let password = $state('');
  let loading = $state(false);
  let errorMsg = $state<string | null>(null);

  async function handleLogin(e: Event) {
    e.preventDefault();
    loading = true;
    errorMsg = null;

    try {
      const { data, error } = await supabase.auth.signInWithPassword({
        email,
        password,
      });

      if (error) throw error;
      if (!data.session) throw new Error("No session created");

      document.cookie = `sb-admin-token=${data.session.access_token}; Path=/; max-age=86400; SameSite=Lax`;
      window.location.reload();
    } catch (err: any) {
      errorMsg = err.message || "Failed to sign in";
    } finally {
      loading = false;
    }
  }
</script>

<form onsubmit={handleLogin} class="space-y-6">
  {#if errorMsg}
    <div class="p-4 bg-red-50 dark:bg-red-500/10 border border-red-200 dark:border-red-500/20 rounded-xl text-red-600 dark:text-red-400 text-sm">
      {errorMsg}
    </div>
  {/if}

  <div class="space-y-2">
    <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300" for="email">
      Admin Email
    </label>
    <div class="relative">
      <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-zinc-400">
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
        </svg>
      </div>
      <input
        id="email"
        type="email"
        bind:value={email}
        required
        class="input-field pl-10"
        placeholder="admin@example.com"
      />
    </div>
  </div>

  <div class="space-y-2">
    <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300" for="password">
      Password
    </label>
    <div class="relative">
      <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-zinc-400">
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      </div>
      <input
        id="password"
        type="password"
        bind:value={password}
        required
        class="input-field pl-10"
        placeholder="••••••••"
      />
    </div>
  </div>

  <button type="submit" disabled={loading} class="btn-primary w-full flex justify-center">
    {#if loading}
      <svg class="animate-spin -ml-1 mr-3 h-5 w-5" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
      Authenticating...
    {:else}
      Access Console
    {/if}
  </button>
</form>
