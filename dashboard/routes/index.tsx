export default function Home() {
  return (
    <div class="px-4 py-8 mx-auto bg-[#86efac]">
      <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center min-h-screen">
        <h1 class="text-6xl font-bold mb-4 text-center">Project Atlas</h1>
        <p class="text-2xl text-center mb-8">
          The intelligence layer for your media.
        </p>
        <div class="flex gap-4">
          <a
            href="/dashboard"
            class="bg-black text-white px-6 py-3 rounded-md font-semibold hover:bg-gray-800 transition"
          >
            Get Started
          </a>
          <a
            href="/admin"
            class="bg-white text-black px-6 py-3 rounded-md font-semibold hover:bg-gray-100 transition shadow-sm"
          >
            Admin Access
          </a>
        </div>
      </div>
    </div>
  );
}
