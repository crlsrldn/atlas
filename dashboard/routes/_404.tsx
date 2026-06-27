import { Head } from "$fresh/runtime.ts";

export default function Error404() {
  return (
    <>
      <Head>
        <title>404 — Page Not Found | Atlas</title>
      </Head>
      <div class="flex flex-col items-center justify-center min-h-[80vh] px-4 text-center animate-fade-in-up">
        {/* Decorative orb */}
        <div class="absolute w-96 h-96 bg-indigo-600/10 rounded-full blur-3xl pointer-events-none" />

        <div class="relative z-10 space-y-8 max-w-md">
          {/* Error code */}
          <div class="space-y-2">
            <p class="text-8xl font-black text-transparent bg-clip-text bg-gradient-to-br from-zinc-700 to-zinc-800 select-none">
              404
            </p>
            <div class="divider" />
          </div>

          {/* Message */}
          <div class="space-y-3">
            <h1 class="text-2xl font-bold text-white">Page Not Found</h1>
            <p class="text-zinc-400 leading-relaxed">
              The page you were looking for doesn't exist or has been moved. Let's get you back on track.
            </p>
          </div>

          {/* Actions */}
          <div class="flex flex-col sm:flex-row items-center justify-center gap-3">
            <a href="/" class="btn-primary px-8 py-3 rounded-xl">
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2}>
                <path stroke-linecap="round" stroke-linejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
              </svg>
              Go Home
            </a>
            <a href="/dashboard" class="btn-ghost px-8 py-3 rounded-xl">
              My Dashboard
            </a>
          </div>
        </div>
      </div>
    </>
  );
}
