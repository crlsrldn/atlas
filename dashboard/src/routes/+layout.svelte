<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';

  let { children, data }: { children: any; data: any } = $props();
  
  let isAuthenticated = $derived(!!data.session);
  let pathname = $derived($page.url.pathname);

  let navLinks = $derived(isAuthenticated
    ? [
        { href: "/", label: "Home" },
        { href: "/dashboard", label: "Dashboard" },
      ]
    : [
        { href: "/", label: "Home" },
        { href: "/login", label: "Sign In" },
      ]);

  async function handleSignOut() {
    // Implement signout logic later via form action or supabase client
    const { supabase } = data;
    await supabase.auth.signOut();
    window.location.href = '/';
  }
</script>

<div class="fixed inset-0 pointer-events-none overflow-hidden z-0">
  <div class="absolute -top-[20%] -left-[10%] w-[70%] h-[60%] bg-emerald-600/8 rounded-full blur-[120px]"></div>
  <div class="absolute -bottom-[20%] -right-[10%] w-[60%] h-[55%] bg-teal-600/8 rounded-full blur-[120px]"></div>
  <div class="absolute top-[40%] left-[50%] -translate-x-1/2 w-[40%] h-[30%] bg-emerald-500/4 rounded-full blur-[100px]"></div>
</div>

<header class="sticky top-0 z-50 w-full">
  <div class="absolute inset-0 bg-[#09090b]/80 backdrop-blur-2xl border-b border-white/[0.06] transition-colors duration-200"></div>
  <nav class="relative max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
    <!-- Logo -->
    <a href="/" class="flex items-center gap-3 group flex-shrink-0">
      <div class="relative w-8 h-8">
        <div class="absolute inset-0 rounded-xl bg-gradient-to-br from-emerald-500 to-teal-600 opacity-90 group-hover:opacity-100 transition-opacity"></div>
        <div class="absolute inset-0 rounded-xl bg-gradient-to-br from-emerald-500 to-teal-600 blur-md opacity-0 group-hover:opacity-60 transition-opacity"></div>
        <div class="relative flex items-center justify-center w-full h-full text-white font-bold text-sm rounded-xl">
          A
        </div>
      </div>
      <span class="font-bold text-lg text-white tracking-tight">
        Atlas
      </span>
    </a>

    <!-- Nav links -->
    <div class="hidden sm:flex items-center gap-1">
      {#each navLinks as { href, label }}
        {@const isActive = pathname === href || (href !== "/" && pathname.startsWith(href))}
        <a
          {href}
          class="nav-link {isActive ? 'nav-link-active text-emerald-300 bg-emerald-500/10' : ''}"
        >
          {label}
        </a>
      {/each}

      {#if isAuthenticated}
        <button
          type="button"
          onclick={handleSignOut}
          class="nav-link text-zinc-400 hover:text-white hover:bg-white/5 transition-colors ml-2"
        >
          Sign Out
        </button>
      {/if}
    </div>

    <!-- Mobile links -->
    <div class="sm:hidden flex items-center gap-2 text-sm text-zinc-400">
      {#each navLinks.filter(l => l.href !== "/") as { href, label }}
        <a {href} class="nav-link text-xs">
          {label}
        </a>
      {/each}
      {#if isAuthenticated}
        <button
          type="button"
          onclick={handleSignOut}
          class="nav-link text-xs"
        >
          Sign Out
        </button>
      {/if}
    </div>
  </nav>
</header>

<main class="flex-grow flex flex-col relative z-10 w-full min-h-[calc(100vh-16rem)]">
  {@render children()}
</main>

<footer class="relative z-10 border-t border-white/[0.06] py-10 mt-auto transition-colors duration-200">
  <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="flex flex-col sm:flex-row items-center justify-between gap-4">
      <div class="flex items-center gap-2">
        <div class="w-6 h-6 rounded-lg bg-gradient-to-br from-emerald-500 to-teal-600 flex items-center justify-center text-white font-bold text-xs">
          A
        </div>
        <span class="text-sm font-medium text-zinc-400">
          Project Atlas
        </span>
      </div>
      <p class="text-sm text-zinc-600">
        &copy; {new Date().getFullYear()} Project Atlas. Built with privacy first.
      </p>
      <div class="flex items-center gap-4 text-xs text-zinc-600">
        <a href="/admin" class="hover:text-zinc-400 transition-colors">
          Admin Console
        </a>
        <span class="flex items-center gap-1.5">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
          All Systems Operational
        </span>
      </div>
    </div>
  </div>
</footer>
