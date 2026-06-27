import { type PageProps } from "$fresh/server.ts";

export default function App({ Component }: PageProps) {
  return (
    <html lang="en">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Atlas | The Intelligence Layer for Media</title>
        <link rel="stylesheet" href="/styles.css" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
        <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet" />
        <style>
          {`
            body {
              font-family: 'Inter', sans-serif;
              background-color: #09090b; /* zinc-950 */
              color: #fafafa; /* zinc-50 */
              margin: 0;
            }
          `}
        </style>
      </head>
      <body class="antialiased min-h-screen bg-[#09090b] text-zinc-50 selection:bg-indigo-500/30 selection:text-white flex flex-col">
        <div class="relative flex-grow flex flex-col min-h-screen overflow-x-hidden">
          {/* Subtle background glow effect */}
          <div class="fixed top-[-25%] left-[-15%] w-[60%] h-[60%] bg-indigo-600/10 rounded-full blur-[140px] pointer-events-none" />
          <div class="fixed bottom-[-20%] right-[-10%] w-[50%] h-[50%] bg-purple-600/10 rounded-full blur-[140px] pointer-events-none" />
          
          {/* Global Navbar */}
          <header class="sticky top-0 z-50 w-full border-b border-white/5 bg-[#09090b]/80 backdrop-blur-xl">
            <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
              <a href="/" class="flex items-center gap-2 group">
                <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white font-bold shadow-[0_0_15px_rgba(79,70,229,0.3)] group-hover:shadow-[0_0_25px_rgba(79,70,229,0.5)] transition-shadow">
                  A
                </div>
                <span class="font-bold tracking-tight text-xl text-white">Atlas</span>
              </a>
              <nav class="flex items-center gap-6 text-sm font-medium">
                <a href="/dashboard" class="text-zinc-400 hover:text-white transition-colors">Dashboard</a>
                <a href="/admin" class="text-zinc-400 hover:text-white transition-colors">Admin</a>
                <a href="https://github.com/crlsrldn/atlas" target="_blank" class="hidden sm:inline-flex px-4 py-2 bg-white/5 hover:bg-white/10 border border-white/10 rounded-lg transition-colors text-white items-center gap-2">
                  <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path fill-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd" />
                  </svg>
                  GitHub
                </a>
              </nav>
            </div>
          </header>

          <main class="flex-grow flex flex-col relative z-10 w-full">
            <Component />
          </main>
          
          <footer class="py-8 text-center text-zinc-500 text-sm border-t border-white/5 relative z-10">
            <p>&copy; {new Date().getFullYear()} Project Atlas. All rights reserved.</p>
          </footer>
        </div>
      </body>
    </html>
  );
}
