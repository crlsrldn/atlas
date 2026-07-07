<script lang="ts">
  import AuthForm from '$lib/components/AuthForm.svelte';
  import { enhance } from '$app/forms';
  
  let { data, form } = $props<{
    data: import('./$types').PageData;
    form: import('./$types').ActionData;
  }>();

  let view: 'waitlist' | 'invite' = $state('waitlist');
</script>

<svelte:head>
  <title>Sign Up — Atlas</title>
</svelte:head>

<div class="flex-grow flex items-center justify-center p-4 min-h-[80vh]">
  {#if data.signupsOpen}
    <AuthForm type="signup" />
  {:else}
    <div class="glass-card p-8 max-w-md w-full mx-auto animate-fade-in-up">
      <div class="text-center mb-8">
        <div class="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-indigo-500/10 text-indigo-400 mb-4">
          <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        </div>
        <h2 class="text-2xl font-bold text-white">Atlas is in Private Beta</h2>
        <p class="text-zinc-400 mt-2 text-sm">
          We are currently limiting new signups to ensure stability. Join the waitlist or use an invite code.
        </p>
      </div>

      <div class="flex p-1 mb-6 rounded-lg bg-black/10 dark:bg-white/5">
        <button 
          class="flex-1 py-1.5 text-sm font-medium rounded-md transition-all {view === 'waitlist' ? 'bg-zinc-800 text-white shadow-sm' : 'text-zinc-400 hover:text-zinc-300'}"
          onclick={() => view = 'waitlist'}
        >
          Join Waitlist
        </button>
        <button 
          class="flex-1 py-1.5 text-sm font-medium rounded-md transition-all {view === 'invite' ? 'bg-zinc-800 text-white shadow-sm' : 'text-zinc-400 hover:text-zinc-300'}"
          onclick={() => view = 'invite'}
        >
          Have an Invite?
        </button>
      </div>

      {#if form?.error}
        <div class="alert alert-error mb-4">
          <div class="flex-1">{form.error}</div>
        </div>
      {/if}

      {#if form?.success && form?.message}
        <div class="alert alert-success mb-4">
          <div class="flex-1">{form.message}</div>
        </div>
      {/if}

      {#if view === 'waitlist'}
        <form method="POST" action="?/waitlist" class="space-y-4" use:enhance>
          <div class="space-y-1.5">
            <label for="email" class="block text-sm font-medium text-zinc-300">Email</label>
            <input type="email" id="email" name="email" required class="input-field w-full py-2.5 px-3" placeholder="you@example.com" />
          </div>
          <button type="submit" class="btn-primary w-full py-2.5">Join Waitlist</button>
        </form>
      {:else}
        <form method="POST" action="?/signupWithInvite" class="space-y-4" use:enhance>
          <div class="space-y-1.5">
            <label for="invite-email" class="block text-sm font-medium text-zinc-300">Email</label>
            <input type="email" id="invite-email" name="email" required class="input-field w-full py-2.5 px-3" placeholder="you@example.com" />
          </div>
          <div class="space-y-1.5">
            <label for="invite-password" class="block text-sm font-medium text-zinc-300">Password</label>
            <input type="password" id="invite-password" name="password" required minlength="6" class="input-field w-full py-2.5 px-3" placeholder="••••••••" />
          </div>
          <div class="space-y-1.5">
            <label for="code" class="block text-sm font-medium text-zinc-300">Invite Code</label>
            <input type="text" id="code" name="code" required class="input-field w-full py-2.5 px-3 uppercase font-mono" placeholder="ATLAS-XXXXXX" />
          </div>
          <button type="submit" class="btn-primary w-full py-2.5">Create Account</button>
        </form>
      {/if}
    </div>
  {/if}
</div>
