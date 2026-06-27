import { type Config } from "tailwindcss";

export default {
  content: ["{routes,islands,components}/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter", "ui-sans-serif", "system-ui", "sans-serif"],
      },
      colors: {
        brand: {
          50: "#eef2ff",
          100: "#e0e7ff",
          200: "#c7d2fe",
          300: "#a5b4fc",
          400: "#818cf8",
          500: "#6366f1",
          600: "#4f46e5",
          700: "#4338ca",
          800: "#3730a3",
          900: "#312e81",
          950: "#1e1b4b",
        },
      },
      animation: {
        "fade-in": "fadeIn 0.6s ease-out forwards",
        "fade-in-up": "fadeInUp 0.6s ease-out forwards",
        "fade-in-down": "fadeInDown 0.4s ease-out forwards",
        "slide-in-right": "slideInRight 0.5s ease-out forwards",
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "float": "float 6s ease-in-out infinite",
        "shimmer": "shimmer 2s linear infinite",
        "glow": "glow 2s ease-in-out infinite alternate",
        "spin-slow": "spin 8s linear infinite",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        fadeInUp: {
          "0%": { opacity: "0", transform: "translateY(20px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        fadeInDown: {
          "0%": { opacity: "0", transform: "translateY(-10px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        slideInRight: {
          "0%": { opacity: "0", transform: "translateX(-20px)" },
          "100%": { opacity: "1", transform: "translateX(0)" },
        },
        float: {
          "0%, 100%": { transform: "translateY(0px)" },
          "50%": { transform: "translateY(-10px)" },
        },
        shimmer: {
          "0%": { backgroundPosition: "-200% 0" },
          "100%": { backgroundPosition: "200% 0" },
        },
        glow: {
          "0%": { boxShadow: "0 0 20px rgba(99, 102, 241, 0.3)" },
          "100%": { boxShadow: "0 0 40px rgba(99, 102, 241, 0.6)" },
        },
      },
      backgroundImage: {
        "gradient-radial": "radial-gradient(var(--tw-gradient-stops))",
        "gradient-conic":
          "conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))",
        "shimmer-gradient":
          "linear-gradient(90deg, transparent, rgba(255,255,255,0.05), transparent)",
      },
      backdropBlur: {
        xs: "2px",
      },
      boxShadow: {
        "glow-sm": "0 0 10px rgba(99, 102, 241, 0.2)",
        "glow-md": "0 0 20px rgba(99, 102, 241, 0.3)",
        "glow-lg": "0 0 40px rgba(99, 102, 241, 0.4)",
        "glow-xl": "0 0 60px rgba(99, 102, 241, 0.5)",
        "glow-purple-sm": "0 0 10px rgba(139, 92, 246, 0.2)",
        "glow-purple-md": "0 0 20px rgba(139, 92, 246, 0.3)",
        "card":
          "0 4px 6px -1px rgba(0, 0, 0, 0.5), 0 2px 4px -2px rgba(0, 0, 0, 0.5)",
        "card-hover":
          "0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5)",
        "inner-glow": "inset 0 1px 0 rgba(255, 255, 255, 0.1)",
      },
    },
  },
  plugins: [],
} satisfies Config;
