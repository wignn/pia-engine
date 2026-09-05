import type { Config } from "tailwindcss";

export default {
  content: [
    "./src/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        tv: {
          bg: "#131722",
          panel: "#1e222d",
          border: "#2a2e39",
          hover: "#2a2e39",
          text: "#d1d4dc",
          muted: "#787b86",
          green: "#089981",
          red: "#f23645",
          blue: "#2962ff",
          surface: "#181b27"
        }
      },
      fontFamily: {
        sans: ["-apple-system", "BlinkMacSystemFont", "Trebuchet MS", "Roboto", "Ubuntu", "sans-serif"],
        mono: ["Consolas", "Courier New", "monospace"]
      }
    },
  },
  plugins: [],
} satisfies Config;
