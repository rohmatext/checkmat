import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [
    tailwindcss(),
    sveltekit({
      // SPA build langsung ke ../frontend (dipakai Tauri sebagai frontendDist)
      adapter: adapter({
        pages: "../frontend",
        assets: "../frontend",
        fallback: "index.html",
      }),
      prerender: { entries: [] },
    }),
  ],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
});
