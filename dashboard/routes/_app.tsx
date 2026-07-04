import { type PageProps } from "$fresh/server.ts";
import Navbar from "../islands/Navbar.tsx";

export default function App({ Component, url }: PageProps) {
  const pathname = url.pathname;
  const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL") || "";
  const supabaseAnonKey = Deno.env.get("PUBLIC_SUPABASE_ANON_KEY") || "";

  return (
    <html lang="en" class="scroll-smooth">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta name="theme-color" content="#09090b" />
        <title>Atlas — Intelligence Layer for Media</title>
        <meta
          name="description"
          content="Atlas automatically resolves, ranks, and streams media with zero friction. The AI-powered intelligence layer for Stremio."
        />
        <link rel="icon" href="/favicon.ico" />
        <link rel="stylesheet" href="/styles.css" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
          rel="preconnect"
          href="https://fonts.gstatic.com"
          crossOrigin="anonymous"
        />
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&display=swap"
          rel="stylesheet"
        />
        <script
          // deno-lint-ignore react-no-danger
          dangerouslySetInnerHTML={{
            __html: `
              try {
                if (localStorage.theme === 'dark' || (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
                  document.documentElement.classList.add('dark');
                } else {
                  document.documentElement.classList.remove('dark');
                }
              } catch (_) {}
            `,
          }}
        />
      </head>
      <body class="antialiased min-h-screen flex flex-col overflow-x-hidden transition-colors duration-200">
        {/* ── Background ambient glows ── */}
        <div class="fixed inset-0 pointer-events-none overflow-hidden z-0">
          <div class="absolute -top-[20%] -left-[10%] w-[70%] h-[60%] bg-indigo-600/8 rounded-full blur-[120px]" />
          <div class="absolute -bottom-[20%] -right-[10%] w-[60%] h-[55%] bg-purple-600/8 rounded-full blur-[120px]" />
          <div class="absolute top-[40%] left-[50%] -translate-x-1/2 w-[40%] h-[30%] bg-indigo-500/4 rounded-full blur-[100px]" />
        </div>

        {/* ── Navigation ── */}
        <Navbar
          pathname={pathname}
          supabaseUrl={supabaseUrl}
          supabaseAnonKey={supabaseAnonKey}
        />

        {/* ── Main content ── */}
        <main class="flex-grow flex flex-col relative z-10 w-full">
          <Component />
        </main>

        {/* ── Footer ── */}
        <footer class="relative z-10 border-t border-black/10 dark:border-white/[0.06] py-10 mt-auto transition-colors duration-200">
          <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
            <div class="flex flex-col sm:flex-row items-center justify-between gap-4">
              <div class="flex items-center gap-2">
                <div class="w-6 h-6 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white font-bold text-xs">
                  A
                </div>
                <span class="text-sm font-medium text-zinc-400">
                  Project Atlas
                </span>
              </div>
              <p class="text-sm text-zinc-600">
                &copy; {new Date().getFullYear()}{" "}
                Project Atlas. Built with privacy first.
              </p>
              <div class="flex items-center gap-4 text-xs text-zinc-600">
                <a href="/admin" class="hover:text-zinc-400 transition-colors">
                  Admin Console
                </a>
                <span class="flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                  All Systems Operational
                </span>
              </div>
            </div>
          </div>
        </footer>
      </body>
    </html>
  );
}
