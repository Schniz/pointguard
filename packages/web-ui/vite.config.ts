import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";

// https://vitejs.dev/config/
export default defineConfig(() => ({
  plugins: [react()],
  base: "/admin",
  server: {
    proxy: {
      "/api": `http://localhost:${process.env.SERVER_PORT || "8080"}`,
    },
  },
}));
