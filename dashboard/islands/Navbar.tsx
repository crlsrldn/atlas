import { useEffect, useState } from "preact/hooks";
import { getSupabaseClient } from "../utils/supabase.ts";

interface NavbarProps {
  pathname: string;
  supabaseUrl: string;
  supabaseAnonKey: string;
}

export default function Navbar({ pathname, supabaseUrl, supabaseAnonKey }: NavbarProps) {
  const [isAuthenticated, setIsAuthenticated] = useState<boolean | null>(null);

  useEffect(() => {
    const supabase = getSupabaseClient(supabaseUrl, supabaseAnonKey);

    // Get initial session
    supabase.auth.getSession().then(({ data: { session } }) => {
      setIsAuthenticated(!!session);
    });

    // Listen for auth changes
    const { data: { subscription } } = supabase.auth.onAuthStateChange(
      (_event, session) => {
        setIsAuthenticated(!!session);
      },
    );

    return () => {
      subscription.unsubscribe();
    };
  }, [supabaseUrl, supabaseAnonKey]);

  const handleSignOut = async () => {
    const supabase = getSupabaseClient(supabaseUrl, supabaseAnonKey);
    await supabase.auth.signOut();
    globalThis.location.href = "/";
  };

  const navLinks = isAuthenticated
    ? [
        { href: "/", label: "Home" },
        { href: "/dashboard", label: "Dashboard" },
      ]
    : [
        { href: "/", label: "Home" },
        { href: "/login", label: "Sign In" },
      ];

  return (
    <header class="sticky top-0 z-50 w-full">
      <div class="absolute inset-0 bg-[#09090b]/80 backdrop-blur-2xl border-b border-white/[0.06]" />
      <nav class="relative max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
        {/* Logo */}
        <a href="/" class="flex items-center gap-3 group flex-shrink-0">
          <div class="relative w-8 h-8">
            <div class="absolute inset-0 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 opacity-90 group-hover:opacity-100 transition-opacity" />
            <div class="absolute inset-0 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 blur-md opacity-0 group-hover:opacity-60 transition-opacity" />
            <div class="relative flex items-center justify-center w-full h-full text-white font-bold text-sm rounded-xl">
              A
            </div>
          </div>
          <span class="font-bold text-lg text-white tracking-tight">Atlas</span>
        </a>

        {/* Nav links */}
        <div class="hidden sm:flex items-center gap-1">
          {navLinks.map(({ href, label }) => {
            const isActive = pathname === href || (href !== "/" && pathname.startsWith(href));
            return (
              <a
                key={href}
                href={href}
                class={`nav-link ${isActive ? "nav-link-active text-indigo-300 bg-indigo-500/10" : ""}`}
              >
                {label}
              </a>
            );
          })}
          
          {isAuthenticated && (
            <button
              onClick={handleSignOut}
              class="nav-link text-zinc-400 hover:text-white hover:bg-white/5 transition-colors ml-2"
            >
              Sign Out
            </button>
          )}
        </div>

        {/* Mobile: hamburger placeholder */}
        <div class="sm:hidden flex items-center gap-2 text-sm text-zinc-400">
          {navLinks.filter((l) => l.href !== "/").map(({ href, label }) => (
            <a key={href} href={href} class="nav-link text-xs">
              {label}
            </a>
          ))}
          {isAuthenticated && (
            <button onClick={handleSignOut} class="nav-link text-xs">
              Sign Out
            </button>
          )}
        </div>
      </nav>
    </header>
  );
}
