import { type PageProps } from "$fresh/server.ts";
export default function App({ Component }: PageProps) {
  return (
    <html>
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Atlas | The Intelligence Layer for Media</title>
        <link rel="stylesheet" href="/styles.css" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
        <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet" />
        <style>
          {`
            body {
              font-family: 'Inter', sans-serif;
              background-color: #0f1115;
              color: #f3f4f6;
              margin: 0;
            }
          `}
        </style>
      </head>
      <body class="antialiased min-h-screen bg-[#0f1115] text-white selection:bg-indigo-500 selection:text-white">
        <div class="relative overflow-hidden min-h-screen">
          {/* Subtle background glow effect */}
          <div class="absolute top-[-20%] left-[-10%] w-[50%] h-[50%] bg-indigo-600/20 rounded-full blur-[120px] pointer-events-none" />
          <div class="absolute bottom-[-20%] right-[-10%] w-[50%] h-[50%] bg-purple-600/20 rounded-full blur-[120px] pointer-events-none" />
          
          <Component />
        </div>
      </body>
    </html>
  );
}
